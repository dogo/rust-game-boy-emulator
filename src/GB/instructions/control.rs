use crate::GB::CPU::CPU;
use super::helpers::Instruction;

pub fn di(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.ime = false;
        4
    }
    Instruction { opcode, name: "DI", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn ei(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.ime_enable_next = true; // EI habilita IME após a próxima instrução
        4
    }
    Instruction { opcode, name: "EI", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn halt(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.halted = true;
        eprintln!("💤 HALT executed! IME={} IF={:02X} IE={:02X}", cpu.ime, cpu.ram.read(0xFF0F), cpu.ram.read(0xFFFF));
        4
    }
    Instruction { opcode, name: "HALT", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn stop(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _cpu: &mut CPU) -> u64 {
        4
    }
    Instruction { opcode, name: "STOP", cycles: 4, size: 2, flags: &[], execute: exec }
}
