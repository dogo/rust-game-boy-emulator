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

/// Passa (protocolo Mooneye) se A=$01 no pós-boot, como no DMG; o MGB começa com A=$FF
fn rom_passing_only_on_dmg() -> Vec<u8> {
    let mut rom = rom_with_cart_type(0x00);
    rom.resize(0x8000, 0);
    rom[0x0100..0x0104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]); // jp $0150
    let fib = [
        0x06, 3, 0x0E, 5, 0x16, 8, 0x1E, 13, 0x26, 21, 0x2E, 34, 0x40,
    ];
    let fail = [
        0x06, 0x42, 0x0E, 0x42, 0x16, 0x42, 0x1E, 0x42, 0x26, 0x42, 0x2E, 0x42, 0x40,
    ];
    let mut code = vec![0xFE, 0x01, 0x20, fib.len() as u8]; // cp $01; jr nz, fail
    code.extend(fib);
    code.extend(fail);
    rom[0x0150..0x0150 + code.len()].copy_from_slice(&code);
    rom
}

fn run_headless(dir: &std::path::Path, rom_name: &str, extra: &[&str]) -> std::process::Output {
    fs::write(dir.join(rom_name), rom_passing_only_on_dmg()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_gb_emu"))
        .arg("--headless")
        .args(extra)
        .arg(dir.join(rom_name))
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn rom_name_does_not_choose_the_hardware_but_model_flag_does() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-boot-model-{:016x}", rand::random::<u64>()));
    fs::create_dir(&dir).unwrap();

    assert!(run_headless(&dir, "game-mgb.gb", &[]).status.success());
    assert!(
        !run_headless(&dir, "game.gb", &["--model", "mgb"])
            .status
            .success()
    );
    let invalid = run_headless(&dir, "game.gb", &["--model", "nes"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("Modelo inválido"));

    fs::remove_dir_all(dir).unwrap();
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
