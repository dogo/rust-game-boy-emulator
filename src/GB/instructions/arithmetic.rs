// Instruções de Aritmética
use crate::GB::CPU::CPU;
use super::helpers::{Instruction, FlagBits, read_r, write_r, read_rr, write_rr};

fn add_set_flags(cpu: &mut CPU, a: u8, val: u8, carry_in: u8) -> u8 {
    let sum = a as u16 + val as u16 + carry_in as u16;
    let res = (sum & 0xFF) as u8;
    cpu.registers.set_flag_z(res == 0);
    cpu.registers.set_flag_n(false);
    cpu.registers.set_flag_h(((a & 0x0F) + (val & 0x0F) + (carry_in & 0x0F)) > 0x0F);
    cpu.registers.set_flag_c(sum > 0xFF);
    res
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

pub fn add_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let res = add_set_flags(cpu, a, val, 0);
        cpu.registers.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "ADD A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn add_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let res = add_set_flags(cpu, a, imm, 0);
        cpu.registers.set_a(res);
        8
    }
    Instruction { opcode, name: "ADD A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn adc_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let carry = if cpu.registers.get_flag_c() { 1 } else { 0 };
        let res = add_set_flags(cpu, a, val, carry);
        cpu.registers.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "ADC A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn adc_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let carry = if cpu.registers.get_flag_c() { 1 } else { 0 };
        let res = add_set_flags(cpu, a, imm, carry);
        cpu.registers.set_a(res);
        8
    }
    Instruction { opcode, name: "ADC A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn sub_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let res = sub_set_flags(cpu, a, val, 0);
        cpu.registers.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "SUB A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn sub_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let res = sub_set_flags(cpu, a, imm, 0);
        cpu.registers.set_a(res);
        8
    }
    Instruction { opcode, name: "SUB A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn sbc_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let src = instr.opcode & 0x07;
        let a = cpu.registers.get_a();
        let val = read_r(cpu, src);
        let carry = if cpu.registers.get_flag_c() { 1 } else { 0 };
        let res = sub_set_flags(cpu, a, val, carry);
        cpu.registers.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "SBC A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn sbc_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        let a = cpu.registers.get_a();
        let carry = if cpu.registers.get_flag_c() { 1 } else { 0 };
        let res = sub_set_flags(cpu, a, imm, carry);
        cpu.registers.set_a(res);
        8
    }
    Instruction { opcode, name: "SBC A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn inc_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let val = read_rr(cpu, idx).wrapping_add(1);
        write_rr(cpu, idx, val);
        8
    }
    Instruction { opcode, name: "INC rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

pub fn dec_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let val = read_rr(cpu, idx).wrapping_sub(1);
        write_rr(cpu, idx, val);
        8
    }
    Instruction { opcode, name: "DEC rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

pub fn add_hl_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let hl = cpu.registers.get_hl();
        let rr = read_rr(cpu, idx);
        let res = hl.wrapping_add(rr);
        cpu.registers.set_hl(res);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(((hl & 0x0FFF) + (rr & 0x0FFF)) > 0x0FFF);
        cpu.registers.set_flag_c((hl as u32 + rr as u32) > 0xFFFF);
        8
    }
    Instruction { opcode, name: "ADD HL,rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

// INC r (r = B,C,D,E,H,L,(HL),A)
pub fn inc_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let r = (instr.opcode >> 3) & 0x07;
        let val = read_r(cpu, r);
        let res = val.wrapping_add(1);
        write_r(cpu, r, res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h((val & 0x0F) + 1 > 0x0F);
        if r == 6 { 12 } else { 4 }
    }
    Instruction { opcode, name: "INC r", cycles: 4, size: 1, flags: &[FlagBits::Z, FlagBits::N, FlagBits::H], execute: exec }
}

// DEC r (r = B, C, D, E, H, L, (HL), A)
pub fn dec_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let r = (instr.opcode >> 3) & 0x07;
        let val = read_r(cpu, r);
        let res = val.wrapping_sub(1);
        write_r(cpu, r, res);
        cpu.registers.set_flag_z(res == 0);
        cpu.registers.set_flag_n(true);
        cpu.registers.set_flag_h((val & 0x0F) == 0);
        if r == 6 { 12 } else { 4 }
    }
    Instruction { opcode, name: "DEC r", cycles: 4, size: 1, flags: &[FlagBits::Z, FlagBits::N, FlagBits::H], execute: exec }
}

// DAA - Decimal Adjust Accumulator (0x27)
// Ajusta A para BCD após ADD/ADC/SUB/SBC conforme flags N,H,C
pub fn daa(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let mut a = cpu.registers.get_a();
        let n = cpu.registers.get_flag_n();
        let mut c = cpu.registers.get_flag_c();
        let h = cpu.registers.get_flag_h();

        let mut adjust: u8 = 0;
        if !n {
            if c || a > 0x99 {
                adjust |= 0x60;
                c = true;
            }
            if h || (a & 0x0F) > 0x09 {
                adjust |= 0x06;
            }
            a = a.wrapping_add(adjust);
        } else {
            if c { adjust |= 0x60; }
            if h { adjust |= 0x06; }
            a = a.wrapping_sub(adjust);
        }

        cpu.registers.set_a(a);
        cpu.registers.set_flag_z(a == 0);
        // N permanece como está
        cpu.registers.set_flag_h(false);
        cpu.registers.set_flag_c(c);
        4
    }
    Instruction { opcode, name: "DAA", cycles: 4, size: 1, flags: &[], execute: exec }
}
