use super::helpers::Instruction;
use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

pub fn jr_r8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let offset = bus.cpu_read(regs.get_pc()) as i8;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        regs.set_pc(regs.get_pc().wrapping_add(offset as u16));
        12
    }
    Instruction {
        opcode,
        name: "JR r8",
        cycles: 12,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn jr_cc_r8(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let offset = bus.cpu_read(regs.get_pc()) as i8;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let cond = match (instr.opcode >> 3) & 0x03 {
            0 => !regs.get_flag_z(), // NZ
            1 => regs.get_flag_z(),  // Z
            2 => !regs.get_flag_c(), // NC
            3 => regs.get_flag_c(),  // C
            _ => false,
        };
        if cond {
            let pc = regs.get_pc();
            regs.set_pc(pc.wrapping_add(offset as u16));
            12
        } else {
            8
        }
    }
    Instruction {
        opcode,
        name: "JR cc,r8",
        cycles: 8,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn jp_a16(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        regs.set_pc((hi << 8) | lo);
        16
    }
    Instruction {
        opcode,
        name: "JP a16",
        cycles: 16,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn jp_cc_a16(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let addr = (hi << 8) | lo;
        let cond = match (instr.opcode >> 3) & 0x03 {
            0 => !regs.get_flag_z(),
            1 => regs.get_flag_z(),
            2 => !regs.get_flag_c(),
            3 => regs.get_flag_c(),
            _ => false,
        };
        if cond {
            regs.set_pc(addr);
            16
        } else {
            12
        }
    }
    Instruction {
        opcode,
        name: "JP cc,a16",
        cycles: 12,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn jp_hl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        regs.set_pc(regs.get_hl());
        4
    }
    Instruction {
        opcode,
        name: "JP (HL)",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: exec,
    }
}
