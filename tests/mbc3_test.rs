use gb_emu::GB::mbc::{MBC, mbc3::MBC3};

#[test]
fn test_mbc3_rom_banking() {
    let mut rom = vec![0xFF; 256 * 1024];
    rom[0x0147] = 0x0F; // MBC3
    let mut mbc = MBC3::new(rom.clone(), 32 * 1024);
    mbc.write_register(0x2000, 0x02); // ROM bank select
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0xFF);
}

#[test]
fn test_mbc3_ram_enable_disable() {
    let mut rom = vec![0x00; 32 * 1024];
    rom[0x0147] = 0x0F; // MBC3
    let mut mbc = MBC3::new(rom, 8 * 1024);
    mbc.write_ram(0xA000, 0x55); // RAM desabilitada
    assert_eq!(mbc.read_ram(0xA000), 0xFF);
    mbc.write_register(0x0000, 0x0A); // Enable RAM
    mbc.write_ram(0xA000, 0x77);
    assert_eq!(mbc.read_ram(0xA000), 0x77);
}
