use gb_emu::GB::CPU::{BootModel, CPU};

fn cgb() -> CPU {
    let mut rom = vec![0; 0x8000];
    rom[0x143] = 0xC0;
    CPU::new_with_model(rom, BootModel::Cgb)
}

#[test]
fn cgb_post_boot_clears_vram_used_by_blank_tiles() {
    let mut cpu = cgb();
    cpu.bus.ppu.vram[0x17FF] = 0x49;
    cpu.bus.ppu.vram_bank1[0x17FF] = 0x49;
    cpu.init_post_boot();
    assert!(cpu.bus.ppu.vram.iter().all(|&byte| byte == 0));
    assert!(cpu.bus.ppu.vram_bank1.iter().all(|&byte| byte == 0));
}

#[test]
fn cgb_vram_banks_and_palette_ports() {
    let mut cpu = cgb();
    let bus = &mut cpu.bus;
    bus.write(0xFF40, 0);
    bus.write(0x8000, 0x12);
    bus.write(0xFF4F, 1);
    bus.write(0x8000, 0x34);
    assert_eq!(bus.read(0x8000), 0x34);
    bus.write(0xFF4F, 0);
    assert_eq!(bus.read(0x8000), 0x12);

    bus.write(0xFF68, 0xBF); // índice 63, auto incremento
    bus.write(0xFF69, 0xA5);
    assert_eq!(bus.read(0xFF68), 0xC0);
    bus.write(0xFF68, 0x3F);
    assert_eq!(bus.read(0xFF69), 0xA5);
    bus.write(0xFF6A, 0x00);
    bus.write(0xFF6B, 0x5A);
    assert_eq!(bus.read(0xFF6B), 0x5A);
    assert_eq!(bus.read(0xFF69), 0xA5); // bancos de paleta separados

    bus.write(0xFF40, 0x91);
    bus.ppu.mode = 3;
    bus.ppu.mode_clock = 100;
    bus.write(0xFF68, 0xBF);
    bus.write(0xFF69, 0x99); // escrita bloqueada, índice ainda avança
    assert_eq!(bus.read(0xFF68), 0xC0);
    assert_eq!(bus.read(0xFF69), 0xFF);
    bus.ppu.mode = 0;
    bus.ppu.mode_clock = 260;
    bus.write(0xFF68, 0x3F);
    assert_eq!(bus.read(0xFF69), 0xA5);
}

#[test]
fn cgb_wram_bank_zero_aliases_bank_one_and_echo_tracks_selection() {
    let mut cpu = cgb();
    let bus = &mut cpu.bus;
    bus.write(0xC000, 0x11);
    bus.write(0xD000, 0x22);
    bus.write(0xFF70, 2);
    bus.write(0xD000, 0x33);
    assert_eq!(bus.read(0xF000), 0x33);
    assert_eq!(bus.read(0xE000), 0x11);
    bus.write(0xFF70, 0);
    assert_eq!(bus.read(0xD000), 0x22);
    assert_eq!(bus.read(0xFF70), 0xF8);
    bus.write(0xF000, 0x44);
    bus.write(0xFF70, 2);
    assert_eq!(bus.read(0xD000), 0x33);
}

#[test]
fn cgb_vram_dma_uses_selected_bank_and_hblank_blocks() {
    let mut cpu = cgb();
    let bus = &mut cpu.bus;
    bus.write(0xFF40, 0);
    for i in 0..48 {
        bus.write(0xC000 + i, (i + 1) as u8);
    }
    bus.ppu.write_vram(0x8000, 0);
    bus.write(0xFF4F, 1);
    bus.write(0xFF51, 0xC0);
    bus.write(0xFF52, 0);
    bus.write(0xFF53, 0);
    bus.write(0xFF54, 0);
    bus.write(0xFF55, 1); // GDMA, dois blocos
    assert_eq!(bus.read(0xFF55), 0xFF);
    assert_eq!(bus.ppu.read_vram_bank1(0x8000), 1);
    assert_eq!(bus.ppu.read_vram_bank1(0x801F), 32);
    assert_eq!(bus.ppu.read_vram(0x8000), 0);

    bus.write(0xFF51, 0xC0);
    bus.write(0xFF52, 0x20);
    bus.write(0xFF54, 0x20);
    bus.write(0xFF40, 0x91);
    bus.write(0xFF55, 0x81); // HDMA, dois HBlanks
    assert_eq!(bus.read(0xFF55), 1);
    for _ in 0..70 {
        bus.tick(4);
    }
    assert_eq!(bus.read(0xFF55), 0);
    assert_eq!(bus.ppu.read_vram_bank1(0x8020), 33);
    for _ in 0..120 {
        bus.tick(4);
    }
    assert_eq!(bus.read(0xFF55), 0xFF);
    assert_eq!(bus.ppu.read_vram_bank1(0x802F), 48);
}

#[test]
fn cgb_double_speed_keeps_ppu_at_normal_clock() {
    let mut cpu = cgb();
    cpu.bus.ppu.mode_clock = 0;
    cpu.bus.cgb_speed = true;
    cpu.bus.tick(8);
    assert_eq!(cpu.bus.ppu.mode_clock, 4);
    cpu.bus.cgb_speed = false;
    cpu.bus.tick(8);
    assert_eq!(cpu.bus.ppu.mode_clock, 12);
}

#[test]
fn cgb_gdma_stalls_cpu_for_each_block() {
    let mut cpu = cgb();
    cpu.bus.write(0xFF40, 0);
    cpu.bus.write(0xFF51, 0xC0);
    cpu.bus.write(0xFF52, 0);
    cpu.bus.write(0xFF53, 0);
    cpu.bus.write(0xFF54, 0);
    cpu.bus.reset_cpu_cycle_log();
    cpu.bus.cpu_write(0xFF55, 1); // dois blocos de 32 T-cycles
    assert_eq!(cpu.bus.take_cpu_cycle_log(), 4 + 64);
}

#[test]
fn cgb_gdma_stall_advances_complete_scanlines() {
    let mut cpu = cgb();
    cpu.bus.ppu.mode_clock = 0;
    cpu.bus.ppu.ly = 0;
    cpu.bus.write(0xFF51, 0xC0);
    cpu.bus.write(0xFF52, 0);
    cpu.bus.write(0xFF53, 0);
    cpu.bus.write(0xFF54, 0);
    cpu.bus.cpu_write(0xFF55, 0x7F); // 128 blocos, 4096 ciclos de CPU parados
    assert_eq!(cpu.bus.ppu.ly, 8);
    assert_eq!(cpu.bus.ppu.mode_clock, 452);
}

#[test]
fn cgb_bg_uses_bank_palette_and_flips() {
    let mut cpu = cgb();
    let ppu = &mut cpu.bus.ppu;
    ppu.vram.fill(0);
    ppu.vram_bank1.fill(0);
    ppu.vram_bank1[0x1800] = 0x2B; // banco 1, flip X, paleta 3
    ppu.vram_bank1[0] = 0x01; // bit à direita vira pixel à esquerda
    ppu.write_cgb_palette(false, 3 * 8 + 2, 0x1F);
    ppu.write_cgb_palette(false, 3 * 8 + 3, 0x00);
    ppu.render_bg_scanline();
    assert_eq!(ppu.cgb_framebuffer[0], 0x001F);
    assert_eq!(ppu.cgb_framebuffer[1], 0);

    ppu.vram_bank1[0x1800] = 0x4B; // flip Y: linha de baixo do tile
    ppu.vram_bank1[0] = 0;
    ppu.vram_bank1[14] = 0x80;
    ppu.render_bg_scanline();
    assert_eq!(ppu.cgb_framebuffer[0], 0x001F);
}

#[test]
fn cgb_window_draws_with_lcdc_priority_bit_clear() {
    let mut cpu = cgb();
    let ppu = &mut cpu.bus.ppu;
    ppu.vram.fill(0);
    ppu.vram_bank1.fill(0);
    ppu.lcdc = 0xF0; // window ligada, BG master priority desligada
    ppu.wy = 0;
    ppu.wx = 7;
    ppu.vram[0x1C00] = 1;
    ppu.vram_bank1[0x1C00] = 0x09; // banco 1, paleta 1
    ppu.vram_bank1[16] = 0x80;
    ppu.write_cgb_palette(false, 8 + 2, 0x00);
    ppu.write_cgb_palette(false, 8 + 3, 0x7C);
    ppu.render_bg_scanline();
    ppu.render_window_scanline();
    assert_eq!(ppu.cgb_framebuffer[0], 0x7C00);
}

#[test]
fn cgb_bg_and_obj_priority_obey_lcdc_and_oam_order() {
    let mut cpu = cgb();
    let ppu = &mut cpu.bus.ppu;
    ppu.vram.fill(0);
    ppu.vram_bank1.fill(0);
    ppu.lcdc = 0x93;
    ppu.vram[0] = 0x80; // BG pixel 1
    ppu.vram_bank1[0x1800] = 0x80; // BG acima de OBJ
    ppu.vram[16] = 0x80; // sprite 1, pixel 1
    ppu.vram[32] = 0xC0; // sprite 2 também cobre x=0
    ppu.write_cgb_palette(false, 2, 0x1F); // BG vermelho
    ppu.write_cgb_palette(true, 2, 0xE0); // OBJ verde
    ppu.write_cgb_palette(true, 3, 0x03);
    ppu.write_cgb_palette(true, 10, 0x00); // OBJ paleta 1 azul
    ppu.write_cgb_palette(true, 11, 0x7C);
    ppu.oam.fill(0);
    ppu.oam[0..4].copy_from_slice(&[16, 8, 1, 0]);
    ppu.oam[4..8].copy_from_slice(&[16, 7, 2, 1]);

    ppu.render_bg_scanline();
    ppu.render_sprites_scanline(0);
    assert_eq!(ppu.cgb_framebuffer[0], 0x001F); // BG attr bit 7

    ppu.lcdc &= !0x01; // CGB: limpa prioridade, mantém BG visível
    ppu.render_bg_scanline();
    ppu.render_sprites_scanline(0);
    assert_eq!(ppu.cgb_framebuffer[0], 0x03E0); // primeiro sprite no OAM
}

#[test]
fn hblank_dma_during_halt_does_not_delay_the_wake_up() {
    let mut rom = vec![0; 0x8000];
    rom[0x143] = 0xC0;
    #[rustfmt::skip]
    let code = [
        0x3E, 0x00, 0xE0, 0x51, 0xE0, 0x52, 0xE0, 0x53, 0xE0, 0x54, // origem $0000, destino $8000
        0x3E, 0x01, 0xE0, 0xFF, // IE = VBlank
        0x3E, 0xFF, 0xE0, 0x55, // HDMA de 128 blocos
        0xAF, 0xE0, 0x0F, // IF = 0
        0x76, 0x00, 0x18, 0xFE, // halt; nop; jr -2
    ];
    rom[0x100..0x100 + code.len()].copy_from_slice(&code);
    let mut cpu = CPU::new_with_model(rom, BootModel::Cgb);
    cpu.init_post_boot();
    while !cpu.halted {
        cpu.execute_next();
    }
    let mut waking_call = 0;
    while cpu.halted {
        waking_call = cpu.execute_next();
    }
    assert_eq!(cpu.bus.read(0xFF55), 0xFF, "HDMA terminou durante o HALT");
    assert_eq!(waking_call, 4, "só o NOP depois do HALT");
    assert_eq!(cpu.bus.ppu.ly, 144);
}
