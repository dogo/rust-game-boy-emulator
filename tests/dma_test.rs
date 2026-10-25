use gb_emu::GB::CPU::CPU;

/// WRAM $C000-$C09F = índice do byte, $D000-$D09F = índice + 0x80 (com wrap)
fn cpu_with_wram_pattern() -> CPU {
    let mut cpu = CPU::new(vec![0; 32768]);
    for i in 0..0xA0u16 {
        cpu.bus.write(0xC000 + i, i as u8);
        cpu.bus.write(0xD000 + i, (i as u8).wrapping_add(0x80));
    }
    cpu
}

/// Inicia o DMA e deixa os 4 primeiros bytes serem copiados
fn start_dma(cpu: &mut CPU, page: u8) {
    cpu.bus.write(0xFF46, page);
    cpu.bus.tick(16);
}

#[test]
fn read_on_dma_bus_returns_byte_being_transferred() {
    let mut cpu = cpu_with_wram_pattern();
    start_dma(&mut cpu, 0xC0);
    assert_eq!(cpu.bus.cpu_read(0xD000), 3);
}

#[test]
fn rom_source_also_conflicts_with_wram() {
    let mut rom = vec![0; 32768];
    rom[0x1003] = 0x5A;
    let mut cpu = CPU::new(rom);
    cpu.bus.write(0xC123, 0x11);
    start_dma(&mut cpu, 0x10);
    assert_eq!(cpu.bus.cpu_read(0xC123), 0x5A);
}

#[test]
fn vram_source_leaves_main_bus_free() {
    let mut cpu = cpu_with_wram_pattern();
    cpu.bus.write(0xFF40, 0x00); // LCD desligado: VRAM acessível
    cpu.bus.write(0x8003, 0x77);
    start_dma(&mut cpu, 0x80);
    // Cada leitura da CPU também avança o DMA em um byte
    assert_eq!(cpu.bus.cpu_read(0x9000), 0x77);
    assert_eq!(cpu.bus.cpu_read(0xC005), 5);
}

#[test]
fn hram_stays_accessible_during_dma() {
    let mut cpu = cpu_with_wram_pattern();
    start_dma(&mut cpu, 0xC0);
    cpu.bus.cpu_write(0xFF80, 0x55);
    assert_eq!(cpu.bus.cpu_read(0xFF80), 0x55);
}

#[test]
fn write_during_wram_dma_is_lost_and_ands_into_oam() {
    let mut cpu = cpu_with_wram_pattern();
    cpu.bus.write(0xC003, 0x0F);
    start_dma(&mut cpu, 0xC0);
    cpu.bus.cpu_write(0xD000, 0x3C);
    cpu.bus.tick(700);
    assert_eq!(cpu.bus.read(0xD000), 0x80);
    assert_eq!(cpu.bus.ppu.oam[3], 0x0C);
}

#[test]
fn write_during_rom_dma_reaches_the_address_dma_read() {
    // MBC1, 64 KB: a escrita da CPU vai para $2003 e seleciona o banco 2
    let mut rom = vec![0; 0x10000];
    rom[0x147] = 0x01;
    rom[0x148] = 0x01;
    rom[0x8000] = 0xB2;
    let mut cpu = CPU::new(rom);
    start_dma(&mut cpu, 0x20);
    cpu.bus.cpu_write(0xC000, 0x02);
    cpu.bus.tick(640);
    assert_eq!(cpu.bus.read(0x4000), 0xB2);
}

#[test]
fn no_conflict_before_first_byte_or_after_end() {
    let mut cpu = cpu_with_wram_pattern();
    cpu.bus.write(0xFF46, 0xC0);
    assert_eq!(cpu.bus.cpu_read(0xD001), 0x81);
    cpu.bus.tick(700);
    assert_eq!(cpu.bus.cpu_read(0xD002), 0x82);
}

#[test]
fn restarted_dma_conflicts_with_new_source() {
    let mut cpu = cpu_with_wram_pattern();
    start_dma(&mut cpu, 0xC0);
    start_dma(&mut cpu, 0xD0);
    let value = cpu.bus.cpu_read(0xC050);
    assert!((0x80..0x90).contains(&value), "{value:02X}");
}

#[test]
fn cgb_mode_has_no_dmg_bus_conflicts() {
    let mut cpu = cpu_with_wram_pattern();
    cpu.bus.set_cgb_mode(true);
    start_dma(&mut cpu, 0x10);
    assert_eq!(cpu.bus.cpu_read(0xC007), 7);
}
