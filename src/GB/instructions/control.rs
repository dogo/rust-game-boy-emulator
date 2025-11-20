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
        cpu.ime = true;
        4
    }
    Instruction { opcode, name: "EI", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn halt(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _cpu: &mut CPU) -> u64 {
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
