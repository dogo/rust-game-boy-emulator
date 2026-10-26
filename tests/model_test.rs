use gb_emu::GB::CPU::{BootModel, CPU};

#[test]
fn cgb_game_mode_depends_on_hardware_and_cartridge() {
    let mut rom = vec![0; 0x8000];
    rom[0x0143] = 0xc0;
    assert!(
        !CPU::new_with_model(rom.clone(), BootModel::DmgAbc)
            .bus
            .cgb_mode
    );
    assert!(
        CPU::new_with_model(rom.clone(), BootModel::Cgb)
            .bus
            .cgb_mode
    );
    assert!(CPU::new(rom).bus.cgb_mode);

    let mut compatible = vec![0; 0x8000];
    compatible[0x0143] = 0x80;
    assert!(!CPU::new(compatible.clone()).bus.cgb_mode);
    assert!(CPU::new_with_model(compatible, BootModel::Cgb).bus.cgb_mode);
}
