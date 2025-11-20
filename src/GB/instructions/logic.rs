// Instruções lógicas
use crate::GB::CPU::CPU;
use super::helpers::{Instruction, read_r};

pub fn and_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let res = a & val;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(true);
        cpu.registers.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "AND A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn and_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let res = a & imm;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(true);
        cpu.registers.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "AND A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn or_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let res = a | val;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(false);
        cpu.registers.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "OR A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn or_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let res = a | imm;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(false);
        cpu.registers.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "OR A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn xor_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let res = a ^ val;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(false);
        cpu.registers.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "XOR A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn xor_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let res = a ^ imm;
        cpu.registers.set_a(res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(false);
        cpu.registers.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "XOR A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

fn sub_set_flags(cpu: &mut CPU, a: u8, val: u8, carry_in: u8) -> u8 {
    let diff = a as i16 - val as i16 - carry_in as i16;
    let res = (diff & 0xFF) as u8;
    cpu.registers.set_flag_z(res == 0);
    cpu.registers.set_flag_n(true);
    cpu.registers.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16 - (carry_in & 0x0F) as i16) < 0);
    cpu.registers.set_flag_c(diff < 0);
    res
}

pub fn cp_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        sub_set_flags(cpu, a, val, 0);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "CP A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn cp_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        sub_set_flags(cpu, a, imm, 0);
        8
    }
    Instruction { opcode, name: "CP A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}
