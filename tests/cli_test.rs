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
    fix_header_checksum(&mut rom);
    rom
}

fn fix_header_checksum(rom: &mut [u8]) {
    rom[0x014D] = rom[0x0134..=0x014C]
        .iter()
        .fold(0u8, |sum, &byte| sum.wrapping_sub(byte).wrapping_sub(1));
}

/// Passa direto pelo protocolo Mooneye, com a flag CGB ($0143) dada
fn passing_rom_with_cgb_flag(flag: u8) -> Vec<u8> {
    let mut rom = rom_with_cart_type(0x00);
    rom.resize(0x8000, 0);
    rom[0x0100..0x0104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]); // jp $0150
    let fib = [
        0x06, 3, 0x0E, 5, 0x16, 8, 0x1E, 13, 0x26, 21, 0x2E, 34, 0x40,
    ];
    rom[0x0150..0x0150 + fib.len()].copy_from_slice(&fib);
    rom[0x0143] = flag;
    fix_header_checksum(&mut rom);
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

/// Passa (protocolo Mooneye) se A tiver o valor dado ao entrar em $0100
fn rom_passing_if_a(a: u8) -> Vec<u8> {
    let mut rom = rom_with_cart_type(0x00);
    rom.resize(0x8000, 0);
    rom[0x0100..0x0104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]); // jp $0150
    let fib = [
        0x06, 3, 0x0E, 5, 0x16, 8, 0x1E, 13, 0x26, 21, 0x2E, 34, 0x40,
    ];
    let fail = [
        0x06, 0x42, 0x0E, 0x42, 0x16, 0x42, 0x1E, 0x42, 0x26, 0x42, 0x2E, 0x42, 0x40,
    ];
    let mut code = vec![0xFE, a, 0x20, fib.len() as u8]; // cp a; jr nz, fail
    code.extend(fib);
    code.extend(fail);
    rom[0x0150..0x0150 + code.len()].copy_from_slice(&code);
    rom
}

fn run_headless(dir: &std::path::Path, rom_name: &str, extra: &[&str]) -> std::process::Output {
    // O DMG começa com A=$01; o MGB, com A=$FF
    fs::write(dir.join(rom_name), rom_passing_if_a(0x01)).unwrap();
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
        std::env::temp_dir().join(format!("gb-emu-boot-model-{:016x}", gb_emu::random_u64()));
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
fn dmg_boot_rom_runs_only_on_the_dmg_model() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-boot-rom-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    let mut boot = vec![0; 0x100];
    boot[..5].copy_from_slice(&[0x3E, 0x42, 0xC3, 0xFE, 0x00]); // ld a,$42; jp $00FE
    boot[0xFE..].copy_from_slice(&[0xE0, 0x50]); // desliga o boot ROM e segue em $0100
    fs::write(dir.join("dmg_boot.bin"), boot).unwrap();
    let run = |a: u8, extra: &[&str]| {
        fs::write(dir.join("game.gb"), rom_passing_if_a(a)).unwrap();
        Command::new(env!("CARGO_BIN_EXE_gb_emu"))
            .arg("--headless")
            .args(extra)
            .arg(dir.join("game.gb"))
            .current_dir(&dir)
            .output()
            .unwrap()
            .status
            .success()
    };

    assert!(run(0x42, &[]), "o boot ROM roda no DMG");
    assert!(run(0x11, &["--model", "cgb"]), "o CGB parte do pós-boot");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cgb_only_rom_needs_an_explicit_model() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-cgb-only-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    let run = |flag: u8, extra: &[&str]| {
        let rom_path = dir.join(format!("game-{flag:02x}.gb"));
        fs::write(&rom_path, passing_rom_with_cgb_flag(flag)).unwrap();
        Command::new(env!("CARGO_BIN_EXE_gb_emu"))
            .arg("--headless")
            .args(extra)
            .arg(&rom_path)
            .current_dir(&dir)
            .output()
            .unwrap()
    };

    let refused = run(0xC0, &[]);
    assert!(!refused.status.success());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("exclusiva de Game Boy Color"),
        "stderr: {stderr}"
    );
    assert!(run(0xC0, &["--model", "cgb"]).status.success());
    assert!(!run(0xC0, &["--model", "dmg"]).status.success());
    // CGB-compatível ($80) continua no caminho DMG sem --model
    assert!(run(0x80, &[]).status.success());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unknown_cartridge_type_is_rejected() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-unknown-cart-{:016x}", gb_emu::random_u64()));
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

/// O emulador deve sair com erro ao carregar `save` e deixar o arquivo intacto
fn assert_save_rejected(rom: Vec<u8>, save: &[u8]) {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-invalid-save-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    let rom_path = dir.join("game.gb");
    let save_path = dir.join("game.sav");
    fs::write(&rom_path, rom).unwrap();
    fs::write(&save_path, save).unwrap();

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

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn invalid_save_exits_without_replacing_file() {
    let mut save = vec![0; 8 * 1024];
    save.extend_from_slice(&[61, 0, 0, 0, 0]);
    assert_save_rejected(mbc3_rom(), &save);
}

#[test]
fn cart_without_ram_exits_cleanly_without_save() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("gb-emu-no-ram-{:016x}", gb_emu::random_u64()));
    fs::create_dir(&dir).unwrap();
    // Opcode ilegal no entry point: o --trace para quando a CPU trava e o main tenta salvar
    let mut rom = rom_with_cart_type(0x00);
    rom.resize(0x8000, 0);
    rom[0x0100] = 0xD3;
    let rom_path = dir.join("game.gb");
    fs::write(&rom_path, rom).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_gb_emu"))
        .arg("--trace")
        .arg(&rom_path)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.join("game.sav").exists());

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn oversized_save_for_mbc1_is_rejected_and_kept() {
    // Save de 32 KB (outro jogo) num MBC1 com 8 KB de RAM
    assert_save_rejected(rom_with_cart_type(0x03), &vec![0x5A; 32 * 1024]);
}
