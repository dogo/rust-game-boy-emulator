use crate::GB::registers::Registers;
use crate::GB::bus::MemoryBus;
use super::helpers::Instruction;

pub fn di(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        // IME disable must be handled by CPU struct after instruction execution
        4
    }
    Instruction { opcode, name: "DI", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn ei(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        // IME enable must be handled by CPU struct after instruction execution
        4
    }
    Instruction { opcode, name: "EI", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn halt(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        // HALT state must be handled by CPU struct after instruction execution
        4
    }
    Instruction { opcode, name: "HALT", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn stop(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, _regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        // STOP state must be handled by CPU struct after instruction execution
        4
    }
    Instruction { opcode, name: "STOP", cycles: 4, size: 2, flags: &[], execute: exec }
}
