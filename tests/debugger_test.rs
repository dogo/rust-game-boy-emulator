use gb_emu::GB::CPU::CPU;
use gb_emu::GB::debugger::{Debugger, instruction_name};

fn cpu_with_rom(bytes: &[u8]) -> CPU {
    let mut rom = vec![0; 32768];
    rom[..bytes.len()].copy_from_slice(bytes);
    CPU::new(rom)
}

#[test]
fn every_opcode_has_a_name() {
    for opcode in 0..=0xFFu8 {
        let cpu = cpu_with_rom(&[opcode]);
        assert_ne!(instruction_name(&cpu, 0), "UNKNOWN", "{opcode:02X}");
        let cpu = cpu_with_rom(&[0xCB, opcode]);
        assert_ne!(instruction_name(&cpu, 0), "UNKNOWN", "CB {opcode:02X}");
    }
}

#[test]
fn names_follow_the_executed_instruction() {
    for (bytes, name) in [
        (&[0x3E, 0x42][..], "LD A,d8"),
        (&[0xCB, 0x37], "SWAP A"),
        (&[0xCB, 0x7E], "BIT 7,(HL)"),
        (&[0x10, 0x00], "STOP"),
        (&[0xD3], "ILLEGAL"),
    ] {
        assert_eq!(instruction_name(&cpu_with_rom(bytes), 0), name);
    }
}

#[test]
fn disassembly_shows_operands_and_advances_by_length() {
    let cpu = cpu_with_rom(&[0x3E, 0x42, 0xC3, 0x34, 0x12, 0xCB, 0x37, 0x10, 0x00, 0x00]);
    let text = Debugger::format_disassembly(&cpu, 5);
    let lines: Vec<&str> = text.lines().skip(1).collect();
    for (line, (addr, name, operand)) in lines.iter().zip([
        ("0000", "LD A,d8", Some("$42")),
        ("0002", "JP a16", Some("$1234")),
        ("0005", "SWAP A", None),
        ("0007", "STOP", None),
        ("0009", "NOP", None),
    ]) {
        assert!(line.contains(&format!("{addr}:")), "{line}");
        assert!(line.contains(name), "{line}");
        match operand {
            Some(op) => assert!(line.trim_end().ends_with(op), "{line}"),
            None => assert!(!line.contains('$'), "{line}"),
        }
    }
}
