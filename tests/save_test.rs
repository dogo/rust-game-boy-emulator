use gb_emu::GB::bus::MemoryBus;
use gb_emu::GB::mbc::mbc1::MBC1;
use gb_emu::GB::mbc::mbc3::MBC3;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gb-emu-save-test-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    dir
}

/// MBC1+RAM+BATTERY ($03): só cartuchos com bateria têm save
fn bus_with_ram() -> MemoryBus {
    let mut rom = vec![0; 0x8000];
    rom[0x0147] = 0x03;
    MemoryBus::new(Box::new(MBC1::new(rom, 0x2000)))
}

#[test]
fn save_creates_replaces_and_loads_ram() {
    let dir = test_dir();
    let save = dir.join("game.sav");
    let mut bus = bus_with_ram();
    bus.write(0x0000, 0x0A);
    bus.write(0xA000, 0x42);
    bus.save_cart_ram(save.to_str().unwrap()).unwrap();
    assert_eq!(fs::read(&save).unwrap()[0], 0x42);

    #[cfg(unix)]
    fs::set_permissions(&save, fs::Permissions::from_mode(0o600)).unwrap();
    bus.write(0xA000, 0x99);
    bus.save_cart_ram(save.to_str().unwrap()).unwrap();
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&save).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let mut restored = bus_with_ram();
    restored.load_cart_ram(save.to_str().unwrap()).unwrap();
    restored.write(0x0000, 0x0A);
    assert_eq!(restored.read(0xA000), 0x99);
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

    fs::remove_file(save).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn failed_save_preserves_previous_file() {
    let dir = test_dir();
    let save = dir.join("game.sav");
    fs::write(&save, b"previous save").unwrap();
    fs::set_permissions(&save, fs::Permissions::from_mode(0o400)).unwrap();

    let mut bus = bus_with_ram();
    bus.write(0x0000, 0x0A);
    bus.write(0xA000, 0x42);
    // Root pode escrever em arquivos 0400; nesses ambientes a falha não é reproduzível.
    if fs::OpenOptions::new().write(true).open(&save).is_err() {
        assert!(bus.save_cart_ram(save.to_str().unwrap()).is_err());
        assert_eq!(fs::read(&save).unwrap(), b"previous save");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    }

    fs::remove_file(save).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn save_keeps_existing_symlink() {
    let dir = test_dir();
    let target = dir.join("target.sav");
    let link = dir.join("game.sav");
    fs::write(&target, b"previous save").unwrap();
    symlink(&target, &link).unwrap();

    let mut bus = bus_with_ram();
    bus.write(0x0000, 0x0A);
    bus.write(0xA000, 0x42);
    bus.save_cart_ram(link.to_str().unwrap()).unwrap();
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&target).unwrap()[0], 0x42);

    fs::remove_file(link).unwrap();
    fs::remove_file(target).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn missing_save_symlink_target_is_an_error() {
    let dir = test_dir();
    let link = dir.join("game.sav");
    symlink(dir.join("missing.sav"), &link).unwrap();

    let error = bus_with_ram()
        .load_cart_ram(link.to_str().unwrap())
        .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    fs::remove_file(link).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[test]
fn ram_carts_accept_only_saves_of_their_ram_size() {
    // (tipo, código de RAM no header, bytes de RAM)
    for (cart_type, ram_code, ram_len) in [
        (0x03, 0x02, 8 * 1024),  // MBC1
        (0x06, 0x00, 512),       // MBC2
        (0x1B, 0x03, 32 * 1024), // MBC5
        (0x09, 0x02, 8 * 1024),  // ROM+RAM+bateria
    ] {
        let mut rom = vec![0; 32 * 1024];
        rom[0x147] = cart_type;
        rom[0x149] = ram_code;
        let mut mbc = gb_emu::GB::mbc::create_mbc(rom);
        for len in [0, ram_len - 1, ram_len + 1, 4 * ram_len] {
            assert!(
                mbc.load_ram(&vec![0x42; len]).is_err(),
                "{cart_type:02X}: {len}"
            );
        }
        mbc.load_ram(&vec![0x0A; ram_len]).unwrap();
        assert_eq!(mbc.save_ram().unwrap().len(), ram_len, "{cart_type:02X}");
    }
}

#[test]
fn stray_save_for_cart_without_ram_is_ignored() {
    let mut mbc = gb_emu::GB::mbc::create_mbc(vec![0; 32 * 1024]);
    assert!(mbc.load_ram(&[1, 2, 3]).is_ok());
    assert!(mbc.save_ram().is_none());
}

#[test]
fn cart_without_ram_saves_nothing_and_succeeds() {
    let dir = test_dir();
    let save = dir.join("game.sav");
    let bus = MemoryBus::new(gb_emu::GB::mbc::create_mbc(vec![0; 32 * 1024]));
    bus.save_cart_ram(save.to_str().unwrap()).unwrap();
    assert!(!save.exists());
    fs::remove_dir(dir).unwrap();
}

#[test]
fn only_battery_backed_carts_persist_ram() {
    // (tipo sem bateria, tipo equivalente com bateria, código de RAM)
    for (plain, battery, ram_code) in [
        (0x02, 0x03, 0x02), // MBC1+RAM
        (0x05, 0x06, 0x00), // MBC2
        (0x12, 0x13, 0x02), // MBC3+RAM
        (0x1A, 0x1B, 0x02), // MBC5+RAM
        (0x08, 0x09, 0x02), // ROM+RAM
    ] {
        let cart = |cart_type: u8| {
            let mut rom = vec![0; 32 * 1024];
            rom[0x147] = cart_type;
            rom[0x149] = ram_code;
            let mut mbc = gb_emu::GB::mbc::create_mbc(rom);
            mbc.write_register(0x0000, 0x0A);
            mbc.write_ram(0xA000, 0x05);
            mbc
        };

        let mut mbc = cart(plain);
        assert_eq!(
            mbc.read_ram(0xA000) & 0x0F,
            0x05,
            "{plain:02X}: RAM na sessão"
        );
        assert!(mbc.save_ram().is_none(), "{plain:02X}: não deve gerar .sav");
        let stray = cart(battery).save_ram().unwrap();
        let mut other = stray.clone();
        other[0] = 0x0C;
        mbc.load_ram(&other).unwrap();
        assert_eq!(
            mbc.read_ram(0xA000) & 0x0F,
            0x05,
            "{plain:02X}: .sav ignorado"
        );

        let saved = cart(battery).save_ram().unwrap();
        assert_eq!(saved[0] & 0x0F, 0x05, "{battery:02X}: com bateria grava");
    }
}

#[test]
fn periodic_save_writes_only_when_ram_changes() {
    let dir = test_dir();
    let save = dir.join("game.sav");
    let path = save.to_str().unwrap();
    let mut bus = bus_with_ram();
    let mut last = None;

    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    assert!(save.exists());

    fs::remove_file(&save).unwrap();
    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    assert!(!save.exists(), "RAM igual: nada a gravar");

    bus.write(0x0000, 0x0A);
    bus.write(0xA000, 0x42);
    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    assert_eq!(fs::read(&save).unwrap()[0], 0x42);

    fs::remove_file(save).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[test]
fn periodic_save_ignores_the_mbc3_clock() {
    let dir = test_dir();
    let save = dir.join("game.sav");
    let path = save.to_str().unwrap();
    let mut rom = vec![0; 0x8000];
    rom[0x0147] = 0x10; // MBC3+TIMER+RAM+BATTERY
    let mut bus = MemoryBus::new(Box::new(MBC3::new(rom, 0x2000)));
    let mut last = None;

    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    fs::remove_file(&save).unwrap();
    // O horário salvo muda a cada segundo; a RAM não
    std::thread::sleep(std::time::Duration::from_millis(1100));
    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    assert!(!save.exists(), "só o relógio mudou");

    bus.write(0x0000, 0x0A);
    bus.write(0xA000, 0x42);
    bus.save_cart_ram_if_changed(path, &mut last).unwrap();
    assert_eq!(fs::read(&save).unwrap().len(), 0x2000 + 13, "RAM e relógio");

    fs::remove_file(save).unwrap();
    fs::remove_dir(dir).unwrap();
}
