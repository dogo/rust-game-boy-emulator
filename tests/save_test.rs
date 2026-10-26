use gb_emu::GB::bus::MemoryBus;
use gb_emu::GB::mbc::mbc1::MBC1;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gb-emu-save-test-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    dir
}

fn bus_with_ram() -> MemoryBus {
    MemoryBus::new(Box::new(MBC1::new(vec![0; 0x8000], 0x2000)))
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
