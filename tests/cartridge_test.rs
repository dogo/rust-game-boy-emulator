use gb_emu::GB::cartridge::{is_cgb_only_rom, is_cgb_rom};

#[test]
fn test_cgb_header_flags() {
    let mut rom = vec![0; 0x150];

    rom[0x0143] = 0x00;
    assert!(!is_cgb_rom(&rom));
    assert!(!is_cgb_only_rom(&rom));

    rom[0x0143] = 0x80;
    assert!(is_cgb_rom(&rom));
    assert!(!is_cgb_only_rom(&rom));

    rom[0x0143] = 0xC0;
    assert!(is_cgb_rom(&rom));
    assert!(is_cgb_only_rom(&rom));
}
