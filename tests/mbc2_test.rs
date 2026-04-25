use gb_emu::GB::mbc::{MBC, mbc2::MBC2};

#[test]
fn test_mbc2_rom_banking() {
    let mut rom = vec![0xFF; 64 * 1024];
    rom[0x0147] = 0x05; // MBC2
    rom[2 * 0x4000] = 0x42;
    let mut mbc = MBC2::new(rom);
    mbc.write_register(0x2100, 0x02); // ROM bank select
    let val = mbc.read_rom(0x4000);
    assert_eq!(val, 0x42);
}

#[test]
fn test_mbc2_rom_banking_wraps_to_rom_size() {
    let mut rom = vec![0xFF; 64 * 1024];
    rom[0x0147] = 0x05; // MBC2
    rom[0] = 0x40;
    rom[1 * 0x4000] = 0x41;

    let mut mbc = MBC2::new(rom);
    mbc.write_register(0x2100, 0x00); // Bank 0 maps to bank 1
    assert_eq!(mbc.read_rom(0x4000), 0x41);

    mbc.write_register(0x2100, 0x04); // 4 % 4 wraps to bank 0
    assert_eq!(mbc.read_rom(0x4000), 0x40);
}

#[test]
fn test_mbc2_ignores_writes_to_upper_rom_area() {
    let mut rom = vec![0xFF; 64 * 1024];
    rom[0x0147] = 0x05; // MBC2
    rom[1 * 0x4000] = 0x41;
    rom[2 * 0x4000] = 0x42;

    let mut mbc = MBC2::new(rom);
    mbc.write_register(0x2100, 0x02);
    mbc.write_register(0x4100, 0x01);
    assert_eq!(mbc.read_rom(0x4000), 0x42);
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

#[test]
fn test_mbc2_ram_mirrors_across_external_ram_area() {
    let mut rom = vec![0x00; 32 * 1024];
    rom[0x0147] = 0x05; // MBC2
    let mut mbc = MBC2::new(rom);

    mbc.write_register(0x0000, 0x0A);
    mbc.write_ram(0xA000, 0x05);
    assert_eq!(mbc.read_ram(0xA200), 0xF5);

    mbc.write_ram(0xBFFF, 0x0C);
    assert_eq!(mbc.read_ram(0xA1FF), 0xFC);
}
