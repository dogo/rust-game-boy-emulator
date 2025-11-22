use gb_emu::GB::mbc::{MBC, mbc2::MBC2};

#[test]
fn test_mbc2_rom_banking() {
    let mut rom = vec![0xFF; 64 * 1024];
    rom[0x0147] = 0x05; // MBC2
    let mut mbc = MBC2::new(rom.clone());
    mbc.write_register(0x2100, 0x02); // ROM bank select
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0xFF);
}

#[test]
fn test_mbc2_ram_enable_disable() {
    let mut rom = vec![0x00; 32 * 1024];
    rom[0x0147] = 0x05; // MBC2
    let mut mbc = MBC2::new(rom);
    mbc.write_ram(0xA000, 0x55); // RAM desabilitada
    assert_eq!(mbc.read_ram(0xA000), 0xFF);
    mbc.write_register(0x0000, 0x0A); // Enable RAM
    mbc.write_ram(0xA000, 0x0F);
    assert_eq!(mbc.read_ram(0xA000), 0x0F | 0xF0);
}
