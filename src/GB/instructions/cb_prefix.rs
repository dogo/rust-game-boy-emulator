use super::helpers::{Instruction, read_r, write_r};
use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

pub fn cb(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let cb_op = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        match cb_op {
            0x00..=0x07 => exec_rlc(cb_op, regs, bus),
            0x08..=0x0F => exec_rrc(cb_op, regs, bus),
            0x10..=0x17 => exec_rl(cb_op, regs, bus),
            0x18..=0x1F => exec_rr(cb_op, regs, bus),
            0x20..=0x27 => exec_sla(cb_op, regs, bus),
            0x28..=0x2F => exec_sra(cb_op, regs, bus),
            0x30..=0x37 => exec_swap(cb_op, regs, bus),
            0x38..=0x3F => exec_srl(cb_op, regs, bus),
            0x40..=0x7F => exec_bit(cb_op, regs, bus),
            0x80..=0xBF => exec_res(cb_op, regs, bus),
            0xC0..=0xFF => exec_set(cb_op, regs, bus),
        }
    }
    Instruction {
        opcode,
        name: "CB",
        cycles: 4,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

fn exec_rlc(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit7 = (val >> 7) & 1;
    let result = (val << 1) | bit7;
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit7 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_rrc(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit0 = val & 1;
    let result = (val >> 1) | (bit0 << 7);
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit0 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_rl(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let old_carry = if regs.get_flag_c() { 1 } else { 0 };
    let bit7 = (val >> 7) & 1;
    let result = (val << 1) | old_carry;
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit7 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_rr(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let old_carry = if regs.get_flag_c() { 1 } else { 0 };
    let bit0 = val & 1;
    let result = (val >> 1) | (old_carry << 7);
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit0 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_sla(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit7 = (val >> 7) & 1;
    let result = val << 1;
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit7 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_sra(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit0 = val & 1;
    let bit7 = val & 0x80;
    let result = (val >> 1) | bit7;
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit0 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_swap(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(false);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_srl(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit0 = val & 1;
    let result = val >> 1;
    write_r(regs, bus, r_idx, result);
    regs.set_flag_z(result == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(false);
    regs.set_flag_c(bit0 == 1);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_bit(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let bit_idx = (cb_op >> 3) & 0x07;
    let val = read_r(regs, bus, r_idx);
    let bit_set = (val & (1 << bit_idx)) != 0;
    regs.set_flag_z(!bit_set);
    regs.set_flag_n(false);
    regs.set_flag_h(true);
    if r_idx == 6 { 12 } else { 8 }
}

fn exec_res(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let bit_idx = (cb_op >> 3) & 0x07;
    let val = read_r(regs, bus, r_idx);
    let result = val & !(1 << bit_idx);
    write_r(regs, bus, r_idx, result);
    if r_idx == 6 { 16 } else { 8 }
}

fn exec_set(cb_op: u8, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
    let r_idx = cb_op & 0x07;
    let bit_idx = (cb_op >> 3) & 0x07;
    let val = read_r(regs, bus, r_idx);
    let result = val | (1 << bit_idx);
    write_r(regs, bus, r_idx, result);
    if r_idx == 6 { 16 } else { 8 }
}
