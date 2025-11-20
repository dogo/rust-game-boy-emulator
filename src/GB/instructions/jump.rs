use crate::GB::CPU::CPU;
use super::helpers::Instruction;

pub fn jr_r8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let offset = cpu.fetch_next() as i8;
        let pc = cpu.registers.get_pc();
        cpu.registers.set_pc(pc.wrapping_add(offset as u16));
        12
    }
    Instruction { opcode, name: "JR r8", cycles: 12, size: 2, flags: &[], execute: exec }
}

pub fn jr_cc_r8(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let offset = cpu.fetch_next() as i8;
        let cond = match (instr.opcode >> 3) & 0x03 {
            0 => !cpu.registers.get_flag_z(), // NZ
            1 => cpu.registers.get_flag_z(),  // Z
            2 => !cpu.registers.get_flag_c(), // NC
            3 => cpu.registers.get_flag_c(),  // C
            _ => false,
        };
        if cond {
            let pc = cpu.registers.get_pc();
            cpu.registers.set_pc(pc.wrapping_add(offset as u16));
            12
        } else {
            8
        }
    }
    Instruction { opcode, name: "JR cc,r8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn jp_a16(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        cpu.registers.set_pc((hi << 8) | lo);
        16
    }
    Instruction { opcode, name: "JP a16", cycles: 16, size: 3, flags: &[], execute: exec }
}

pub fn jp_cc_a16(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        let addr = (hi << 8) | lo;
        let cond = match (instr.opcode >> 3) & 0x03 {
            0 => !cpu.registers.get_flag_z(),
            1 => cpu.registers.get_flag_z(),
            2 => !cpu.registers.get_flag_c(),
            3 => cpu.registers.get_flag_c(),
            _ => false,
        };
        if cond {
            cpu.registers.set_pc(addr);
            16
        } else {
            12
        }
    }
    Instruction { opcode, name: "JP cc,a16", cycles: 12, size: 3, flags: &[], execute: exec }
}

pub fn jp_hl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.registers.set_pc(cpu.registers.get_hl());
        4
    }
    Instruction { opcode, name: "JP (HL)", cycles: 4, size: 1, flags: &[], execute: exec }
}
