//! Acid2 DMG/CGB (Matt Currie, MIT; licenças em tests/fixtures) comparados
//! pixel a pixel com as referências. O DMG usa índices 0-3; o CGB usa RGB888.

use gb_emu::GB::CPU::{BootModel, CPU};
use gb_emu::GB::PPU::rgb555_to_rgb888;

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

#[test]
fn cgb_acid2_matches_hardware_reference() {
    // cgb-acid2 v1.1 (Matt Currie, MIT); RGB bytes from its img/reference.png.
    const CGB_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cgb-acid2");
    let rom = std::fs::read(format!("{CGB_FIXTURES}/cgb-acid2.gbc")).unwrap();
    let reference = std::fs::read(format!("{CGB_FIXTURES}/reference-rgb.bin")).unwrap();
    assert_eq!(reference.len(), 160 * 144 * 3);
    let mut cpu = CPU::new_with_model(rom, BootModel::Cgb);
    cpu.init_post_boot_model(BootModel::Cgb);
    for _ in 0..120 {
        while !cpu.bus.ppu.frame_ready {
            cpu.execute_next();
        }
        cpu.bus.ppu.frame_ready = false;
    }

    let diffs: Vec<usize> = reference
        .chunks_exact(3)
        .enumerate()
        .filter_map(|(i, expected)| {
            let actual = rgb555_to_rgb888(cpu.bus.ppu.cgb_framebuffer[i]);
            (actual != [expected[0], expected[1], expected[2]]).then_some(i)
        })
        .collect();
    assert!(
        diffs.is_empty(),
        "{} pixels diferentes; primeiro em x={} y={}",
        diffs.len(),
        diffs[0] % 160,
        diffs[0] / 160
    );
}
