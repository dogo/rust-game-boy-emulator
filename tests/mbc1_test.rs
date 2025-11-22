use gb_emu::GB::mbc::{MBC, mbc1::MBC1};

#[test]
fn test_mbc1_rom_banking() {
    // ROM de 128KB (4 bancos de 32KB)
    let mut rom = vec![0xFF; 128 * 1024];
    rom[0x0147] = 0x01; // MBC1
    let mut mbc = MBC1::new(rom.clone(), 0);

    // Banco 1
    mbc.write_register(0x2000, 0x01);
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0xFF);

    // Banco 2
    mbc.write_register(0x2000, 0x02);
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0xFF);
}

#[test]
fn test_mbc1_ram_enable_disable() {
    let mut rom = vec![0x00; 32 * 1024];
    rom[0x0147] = 0x01; // MBC1
    let mut mbc = MBC1::new(rom, 8 * 1024);

    // RAM desabilitada
    mbc.write_ram(0xA000, 0x55);
    assert_eq!(mbc.read_ram(0xA000), 0xFF);

    // RAM habilitada
    mbc.write_register(0x0000, 0x0A);
    mbc.write_ram(0xA000, 0x55);
    assert_eq!(mbc.read_ram(0xA000), 0x55);
}
