use gb_emu::GB::mbc::{MBC, create_mbc, is_supported_cart_type, none::NoMBC};

#[test]
fn test_nombc_rom_read() {
    let rom = vec![0xAA; 32 * 1024];
    let mbc = NoMBC::new(rom.clone());
    assert_eq!(mbc.read_rom(0x0000), 0xAA);
    assert_eq!(mbc.read_rom(0x7FFF), 0xAA);
}

#[test]
fn test_nombc_ram_read_write() {
    let rom = vec![0xBB; 32 * 1024];
    let mut mbc = NoMBC::new(rom);
    mbc.write_ram(0xA000, 0x55); // Should do nothing
    assert_eq!(mbc.read_ram(0xA000), 0xFF);
}

/// ROM sem controlador com o tipo e o código de tamanho de RAM dados
fn rom_only(cart_type: u8, ram_size_code: u8) -> Vec<u8> {
    let mut rom = vec![0; 32 * 1024];
    rom[0x147] = cart_type;
    rom[0x149] = ram_size_code;
    rom
}

#[test]
fn rom_ram_without_battery_keeps_ram_but_does_not_save() {
    let mut mbc = create_mbc(rom_only(0x08, 0x02));
    mbc.write_ram(0xBFFF, 0x5A);
    assert_eq!(mbc.read_ram(0xBFFF), 0x5A);
    assert!(mbc.save_ram().is_none());
}

#[test]
fn rom_ram_battery_round_trips_save() {
    let mut mbc = create_mbc(rom_only(0x09, 0x02));
    mbc.write_ram(0xA123, 0x42);
    let save = mbc.save_ram().unwrap();
    assert_eq!(save.len(), 8 * 1024);

    let mut restored = create_mbc(rom_only(0x09, 0x02));
    restored.load_ram(&save).unwrap();
    assert_eq!(restored.read_ram(0xA123), 0x42);
}

#[test]
fn rom_ram_smaller_than_window_reads_open_bus_past_end() {
    let mut mbc = create_mbc(rom_only(0x09, 0x01));
    mbc.write_ram(0xA800, 0x42);
    assert_eq!(mbc.read_ram(0xA800), 0xFF);
    assert_eq!(mbc.save_ram().unwrap().len(), 2 * 1024);
}

#[test]
fn only_emulated_cartridge_types_are_supported() {
    for t in [
        0x00, 0x01, 0x03, 0x05, 0x06, 0x08, 0x09, 0x0F, 0x13, 0x19, 0x1E,
    ] {
        assert!(is_supported_cart_type(t), "{t:02X}");
    }
    for t in [0x04, 0x07, 0x0B, 0x14, 0x20, 0x22, 0xFC, 0xFF] {
        assert!(!is_supported_cart_type(t), "{t:02X}");
    }
}
