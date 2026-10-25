//! Comportamentos reproduzidos na auditoria. Ative cada teste ao corrigir a causa.

use gb_emu::GB::{
    CPU::CPU,
    PPU::PPU,
    joypad::Joypad,
    mbc::{MBC, mbc3::MBC3},
};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn sprites_keep_lower_oam_priority() {
    let mut p = PPU::new();
    p.vram.fill(0);
    p.oam.fill(0);
    p.lcdc = 0x93;
    p.obp0 = 0xE4;
    p.vram[0] = 0xFF;
    p.vram[17] = 0xFF;
    p.oam[..8].copy_from_slice(&[16, 8, 0, 0, 16, 8, 1, 0]);
    p.render_sprites_scanline(0);
    assert_eq!(p.framebuffer[0], 1);
}

#[test]
fn bg_disabled_clears_old_priority() {
    let mut p = PPU::new();
    p.vram.fill(0);
    p.oam.fill(0);
    p.bg_priority.fill(true);
    p.lcdc = 0x92;
    p.obp0 = 0xE4;
    p.vram[0] = 0xFF;
    p.oam[..4].copy_from_slice(&[16, 8, 0, 0x80]);
    p.render_bg_scanline();
    p.render_sprites_scanline(0);
    assert_eq!(p.framebuffer[0], 1);
}

#[test]
fn window_wx_6_is_partially_visible() {
    let mut p = PPU::new();
    p.vram.fill(0);
    p.lcdc = 0xB1;
    p.bgp = 0xE4;
    p.wx = 6;
    p.wy = 0;
    p.ly = 0;
    p.vram[0] = 0xFF;
    p.render_window_scanline();
    assert_eq!(p.framebuffer[0], 1);
}

#[test]
fn line_zero_blocks_vram_after_first_frame() {
    let mut p = PPU::new();
    p.headless = true;
    let mut flags = 0;
    for _ in 0..(70224 + 100) / 4 {
        p.step(4, &mut flags);
    }
    assert_eq!((p.ly, p.mode), (0, 3));
    assert!(p.cpu_vram_blocked());
}

#[test]
fn timer_disable_still_completes_overflow() {
    let mut cpu = CPU::new(vec![0; 32768]);
    cpu.bus.set_div_counter(4);
    cpu.bus.write(0xFF06, 0x42);
    cpu.bus.write(0xFF05, 0xFF);
    cpu.bus.write(0xFF07, 5);
    cpu.bus.tick(12);
    assert_eq!(cpu.bus.read(0xFF05), 0);
    cpu.bus.write(0xFF07, 0);
    cpu.bus.tick(4);
    assert_eq!(cpu.bus.read(0xFF05), 0x42);
    assert_ne!(cpu.bus.get_if() & 4, 0);
}

#[test]
#[ignore = "D09: o divisor deve parar durante STOP"]
fn stopped_cpu_freezes_divider() {
    let mut cpu = CPU::new(vec![0; 32768]);
    cpu.stopped = true;
    cpu.bus.set_div_counter(0);
    for _ in 0..64 {
        cpu.execute_next();
    }
    assert_eq!(cpu.bus.read(0xFF04), 0);
}

#[test]
fn joypad_reads_both_selected_groups() {
    let mut j = Joypad::new();
    j.press("A");
    j.write(0);
    assert_eq!(j.read() & 0xF, 0xE);
}

#[test]
fn rtc_halt_is_preserved_when_loading() {
    let mut m = MBC3::new(vec![0; 32768], 8192);
    let mut save = vec![0; 8192];
    save.extend_from_slice(&[10, 0, 0, 0, 0x40]);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    save.extend_from_slice(&(now - 60).to_le_bytes());
    m.load_ram(&save).unwrap();
    m.write_register(0, 0xA);
    m.write_register(0x4000, 9);
    assert_eq!(m.read_ram(0xA000), 0);
}

#[test]
fn rtc_only_cart_can_be_saved() {
    let mut rom = vec![0; 32768];
    rom[0x147] = 0x0F;
    let m = gb_emu::GB::mbc::create_mbc(rom);
    assert!(m.save_ram().is_some());
}

#[test]
fn malformed_rtc_seconds_does_not_crash() {
    let mut m = MBC3::new(vec![0; 32768], 8192);
    let mut save = vec![0; 8192];
    save.extend_from_slice(&[61, 0, 0, 0, 0]);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    save.extend_from_slice(&(now - 1).to_le_bytes());
    assert!(m.load_ram(&save).is_err());
}

#[test]
#[ignore = "L01: debugger deve reconhecer instruções executadas pelo microcódigo"]
fn disassembler_identifies_load_instruction() {
    let mut rom = vec![0; 32768];
    rom[..2].copy_from_slice(&[0x3E, 0x42]);
    let cpu = CPU::new(rom);
    let text = gb_emu::GB::debugger::Debugger::format_disassembly(&cpu, 1);
    assert!(!text.contains("UNKNOWN"), "{text}");
}

#[test]
fn dma_blocks_cpu_writes_to_wram() {
    let mut cpu = CPU::new(vec![0; 32768]);
    cpu.bus.write(0xC000, 0x42);
    cpu.bus.write(0xFF46, 0xC0);
    cpu.bus.tick(16);
    cpu.bus.cpu_write(0xC000, 0x99);
    cpu.bus.tick(700);
    assert_eq!(cpu.bus.read(0xC000), 0x42);
}

#[test]
#[ignore = "D08: cartucho 0x09 mantém RAM externa"]
fn cartridge_type_09_keeps_external_ram() {
    let mut rom = vec![0; 32768];
    rom[0x147] = 9;
    rom[0x149] = 2;
    let mut m = gb_emu::GB::mbc::create_mbc(rom);
    m.write_ram(0xA000, 0x42);
    assert_eq!(m.read_ram(0xA000), 0x42);
}

#[test]
fn window_resumes_from_last_rendered_row() {
    let mut p = PPU::new();
    p.vram.fill(0);
    p.lcdc = 0xB1;
    p.bgp = 0xE4;
    p.wx = 7;
    p.wy = 0;
    p.vram[0] = 0xFF;
    p.vram[2] = 0xFF;
    p.ly = 0;
    p.render_window_scanline();
    p.wx = 167;
    p.ly = 1;
    p.render_window_scanline();
    p.wx = 7;
    p.ly = 2;
    p.render_window_scanline();
    assert_eq!(p.framebuffer[320], 1);
}
