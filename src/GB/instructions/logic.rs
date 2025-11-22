// Instruções lógicas
use crate::GB::registers::Registers;
use crate::GB::bus::MemoryBus;
use super::helpers::{Instruction, read_r};

pub fn and_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = read_r(regs, bus, src);
        let res = a & val;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(true);
        regs.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "AND A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn and_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let res = a & imm;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(true);
        regs.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "AND A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn or_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = read_r(regs, bus, src);
        let res = a | val;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "OR A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn or_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let res = a | imm;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "OR A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn xor_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = read_r(regs, bus, src);
        let res = a ^ val;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(false);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "XOR A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn xor_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let res = a ^ imm;
        regs.set_a(res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(false);
        8
    }
    Instruction { opcode, name: "XOR A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

fn sub_set_flags(regs: &mut Registers, a: u8, val: u8, carry_in: u8) -> u8 {
    let diff = a as i16 - val as i16 - carry_in as i16;
    let res = (diff & 0xFF) as u8;
    regs.set_flag_z(res == 0);
    regs.set_flag_n(true);
    regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16 - (carry_in & 0x0F) as i16) < 0);
    regs.set_flag_c(diff < 0);
    res
}

pub fn cp_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = read_r(regs, bus, src);
        sub_set_flags(regs, a, val, 0);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "CP A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn cp_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        sub_set_flags(regs, a, imm, 0);
        8
    }
    Instruction { opcode, name: "CP A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

// Rotations (sem prefixo CB) — 1 byte, 4 ciclos, Z sempre 0, N=0, H=0
pub fn rlca(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let a = regs.get_a();
        let carry = (a & 0x80) != 0;
        let res = (a << 1) | (if carry { 1 } else { 0 });
        regs.set_a(res);
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(carry);
        4
    }
    Instruction { opcode, name: "RLCA", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn rrca(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let a = regs.get_a();
        let carry = (a & 0x01) != 0;
        let res = (a >> 1) | (if carry { 0x80 } else { 0 });
        regs.set_a(res);
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(carry);
        4
    }
    Instruction { opcode, name: "RRCA", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn rla(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let a = regs.get_a();
        let old_c = regs.get_flag_c();
        let carry = (a & 0x80) != 0;
        let res = (a << 1) | (if old_c { 1 } else { 0 });
        regs.set_a(res);
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(carry);
        4
    }
    Instruction { opcode, name: "RLA", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn rra(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let a = regs.get_a();
        let old_c = regs.get_flag_c();
        let carry = (a & 0x01) != 0;
        let res = ((if old_c { 1 } else { 0 }) << 7) | (a >> 1);
        regs.set_a(res);
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(carry);
        4
    }
    Instruction { opcode, name: "RRA", cycles: 4, size: 1, flags: &[], execute: exec }
}

// CPL - Complement A (0x2F)
pub fn cpl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let a = regs.get_a();
        regs.set_a(!a);
        regs.set_flag_n(true);
        regs.set_flag_h(true);
        4
    }
    Instruction { opcode, name: "CPL", cycles: 4, size: 1, flags: &[], execute: exec }
}

// SCF - Set Carry Flag (0x37)
pub fn scf(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(true);
        4
    }
    Instruction { opcode, name: "SCF", cycles: 4, size: 1, flags: &[], execute: exec }
}

// CCF - Complement Carry Flag (0x3F)
pub fn ccf(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let c = regs.get_flag_c();
        regs.set_flag_n(false);
        regs.set_flag_h(false);
        regs.set_flag_c(!c);
        4
    }
    Instruction { opcode, name: "CCF", cycles: 4, size: 1, flags: &[], execute: exec }
}
