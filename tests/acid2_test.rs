//! dmg-acid2 v1.0 (Matt Currie, MIT; ver tests/fixtures/dmg-acid2/LICENSE) comparado
//! pixel a pixel com a referência do hardware. `reference-dmg.bin` é o
//! `img/reference-dmg.png` do projeto convertido para os índices de cor 0-3 do PPU.

use gb_emu::GB::CPU::{BootModel, CPU};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/dmg-acid2");

#[test]
fn dmg_acid2_matches_hardware_reference() {
    let rom = std::fs::read(format!("{FIXTURES}/dmg-acid2.gb")).unwrap();
    let reference = std::fs::read(format!("{FIXTURES}/reference-dmg.bin")).unwrap();
    let mut cpu = CPU::new(rom);
    cpu.init_post_boot_model(BootModel::DmgAbc);
    for _ in 0..120 {
        while !cpu.bus.ppu.frame_ready {
            cpu.execute_next();
        }
        cpu.bus.ppu.frame_ready = false;
    }

    let diffs: Vec<usize> = (0..reference.len())
        .filter(|&i| cpu.bus.ppu.framebuffer[i] != reference[i])
        .collect();
    assert!(
        diffs.is_empty(),
        "{} pixels diferentes; primeiro em x={} y={}",
        diffs.len(),
        diffs[0] % 160,
        diffs[0] / 160
    );
}
