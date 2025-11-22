use gb_emu::GB::mbc::{MBC, mbc5::MBC5};

#[test]
fn test_mbc5_rom_banking() {
    let mut rom = vec![0xFF; 512 * 1024];
    rom[0x0147] = 0x19; // MBC5
    let mut mbc = MBC5::new(rom.clone(), 64 * 1024);
    mbc.write_register(0x2000, 0x02); // ROM bank low
    mbc.write_register(0x3000, 0x01); // ROM bank high
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0xFF);
}

#[test]
fn test_mbc5_ram_enable_disable() {
    let mut rom = vec![0x00; 32 * 1024];
    rom[0x0147] = 0x19; // MBC5
    let mut mbc = MBC5::new(rom, 8 * 1024);
    mbc.write_ram(0xA000, 0x55); // RAM desabilitada
    assert_eq!(mbc.read_ram(0xA000), 0xFF);
    mbc.write_register(0x0000, 0x0A); // Enable RAM
    mbc.write_ram(0xA000, 0x99);
    assert_eq!(mbc.read_ram(0xA000), 0x99);
}
