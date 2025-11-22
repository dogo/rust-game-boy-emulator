use gb_emu::GB::mbc::{MBC, none::NoMBC};

#[test]
fn test_nombc_rom_read() {
    let mut rom = vec![0xAA; 32 * 1024];
    let mbc = NoMBC::new(rom.clone());
    assert_eq!(mbc.read_rom(0x0000), 0xAA);
    assert_eq!(mbc.read_rom(0x7FFF), 0xAA);
}

#[test]
fn test_nombc_ram_read_write() {
    let mut rom = vec![0xBB; 32 * 1024];
    let mut mbc = NoMBC::new(rom);
    mbc.write_ram(0xA000, 0x55); // Should do nothing
    assert_eq!(mbc.read_ram(0xA000), 0xFF);
}
