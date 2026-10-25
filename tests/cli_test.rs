use std::process::Command;

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
    assert!(String::from_utf8_lossy(&output.stderr).contains("ROM muito pequena"));
}
