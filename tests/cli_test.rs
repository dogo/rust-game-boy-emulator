use std::process::Command;
use std::{fs, path::PathBuf};

fn mbc3_rom() -> Vec<u8> {
    rom_with_cart_type(0x10)
}

fn rom_with_cart_type(cart_type: u8) -> Vec<u8> {
    let logo = [
        0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00,
        0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD,
        0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB,
        0xB9, 0x33, 0x3E,
    ];
    let mut rom = vec![0; 0x150];
    rom[0x0104..0x0134].copy_from_slice(&logo);
    rom[0x0147] = cart_type;
    rom[0x0149] = 0x02;
    rom[0x014D] = rom[0x0134..=0x014C]
        .iter()
        .fold(0u8, |sum, &byte| sum.wrapping_sub(byte).wrapping_sub(1));
    rom
}

#[test]
fn invalid_rom_exits_with_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_gb_emu"))
        .arg("--headless")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/invalid_header.bin"
        ))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ROM muito pequena"), "stderr: {stderr}");
}

#[test]
fn unknown_cartridge_type_is_rejected() {
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "gb-emu-unknown-cart-{:016x}",
        rand::random::<u64>()
    ));
    fs::create_dir(&dir).unwrap();
    let rom_path = dir.join("game.gb");
    fs::write(&rom_path, rom_with_cart_type(0x22)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_gb_emu"))
        .arg("--headless")
        .arg(&rom_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("não suportado: 22"), "stderr: {stderr}");

    fs::remove_file(rom_path).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[test]
fn invalid_save_exits_without_replacing_file() {
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "gb-emu-invalid-save-{:016x}",
        rand::random::<u64>()
    ));
    fs::create_dir(&dir).unwrap();
    let rom_path = dir.join("game.gb");
    let save_path = dir.join("game.sav");
    let mut save = vec![0; 8 * 1024];
    save.extend_from_slice(&[61, 0, 0, 0, 0]);
    fs::write(&rom_path, mbc3_rom()).unwrap();
    fs::write(&save_path, &save).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_gb_emu"))
        .arg("--headless")
        .arg(&rom_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Erro ao carregar save"), "stderr: {stderr}");
    assert_eq!(fs::read(&save_path).unwrap(), save);

    fs::remove_file(rom_path).unwrap();
    fs::remove_file(save_path).unwrap();
    fs::remove_dir(dir).unwrap();
}
