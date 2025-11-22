// Instruções de Aritmética
use super::helpers::{Instruction, FlagBits};

fn add_set_flags(regs: &mut crate::GB::registers::Registers, a: u8, val: u8, carry_in: u8) -> u8 {
    let sum = a as u16 + val as u16 + carry_in as u16;
    let res = (sum & 0xFF) as u8;
    regs.set_flag_z(res == 0);
    regs.set_flag_n(false);
    regs.set_flag_h(((a & 0x0F) + (val & 0x0F) + (carry_in & 0x0F)) > 0x0F);
    regs.set_flag_c(sum > 0xFF);
    res
}

fn sub_set_flags(regs: &mut crate::GB::registers::Registers, a: u8, val: u8, carry_in: u8) -> u8 {
    let diff = a as i16 - val as i16 - carry_in as i16;
    let res = (diff & 0xFF) as u8;
    regs.set_flag_z(res == 0);
    regs.set_flag_n(true);
    regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16 - (carry_in & 0x0F) as i16) < 0);
    regs.set_flag_c(diff < 0);
    res
}

pub fn add_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = crate::GB::instructions::helpers::read_r(regs, bus, src);
        let res = add_set_flags(regs, a, val, 0);
        regs.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "ADD A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn add_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let res = add_set_flags(regs, a, imm, 0);
        regs.set_a(res);
        8
    }
    Instruction { opcode, name: "ADD A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn adc_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = crate::GB::instructions::helpers::read_r(regs, bus, src);
        let carry = if regs.get_flag_c() { 1 } else { 0 };
        let res = add_set_flags(regs, a, val, carry);
        regs.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "ADC A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn adc_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let carry = if regs.get_flag_c() { 1 } else { 0 };
        let res = add_set_flags(regs, a, imm, carry);
        regs.set_a(res);
        8
    }
    Instruction { opcode, name: "ADC A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn sub_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = crate::GB::instructions::helpers::read_r(regs, bus, src);
        let res = sub_set_flags(regs, a, val, 0);
        regs.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "SUB A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn sub_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let res = sub_set_flags(regs, a, imm, 0);
        regs.set_a(res);
        8
    }
    Instruction { opcode, name: "SUB A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn sbc_a_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let src = instr.opcode & 0x07;
        let a = regs.get_a();
        let val = crate::GB::instructions::helpers::read_r(regs, bus, src);
        let carry = if regs.get_flag_c() { 1 } else { 0 };
        let res = sub_set_flags(regs, a, val, carry);
        regs.set_a(res);
        if src == 6 { 8 } else { 4 }
    }
    Instruction { opcode, name: "SBC A,r", cycles: 4, size: 1, flags: &[], execute: exec }
}

pub fn sbc_a_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let imm = bus.read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let a = regs.get_a();
        let carry = if regs.get_flag_c() { 1 } else { 0 };
        let res = sub_set_flags(regs, a, imm, carry);
        regs.set_a(res);
        8
    }
    Instruction { opcode, name: "SBC A,d8", cycles: 8, size: 2, flags: &[], execute: exec }
}

pub fn inc_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let val = crate::GB::instructions::helpers::read_rr(regs, idx).wrapping_add(1);
        crate::GB::instructions::helpers::write_rr(regs, idx, val);
        8
    }
    Instruction { opcode, name: "INC rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

pub fn dec_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let val = crate::GB::instructions::helpers::read_rr(regs, idx).wrapping_sub(1);
        crate::GB::instructions::helpers::write_rr(regs, idx, val);
        8
    }
    Instruction { opcode, name: "DEC rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

pub fn add_hl_rr(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let hl = regs.get_hl();
        let rr = crate::GB::instructions::helpers::read_rr(regs, idx);
        let res = hl.wrapping_add(rr);
        regs.set_hl(res);
        regs.set_flag_n(false);
        regs.set_flag_h(((hl & 0x0FFF) + (rr & 0x0FFF)) > 0x0FFF);
        regs.set_flag_c((hl as u32 + rr as u32) > 0xFFFF);
        8
    }
    Instruction { opcode, name: "ADD HL,rr", cycles: 8, size: 1, flags: &[], execute: exec }
}

// INC r (r = B,C,D,E,H,L,(HL),A)
pub fn inc_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let r = (instr.opcode >> 3) & 0x07;
        let val = crate::GB::instructions::helpers::read_r(regs, bus, r);
        let res = val.wrapping_add(1);
        crate::GB::instructions::helpers::write_r(regs, bus, r, res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(false);
        regs.set_flag_h((val & 0x0F) + 1 > 0x0F);
        if r == 6 { 12 } else { 4 }
    }
    Instruction { opcode, name: "INC r", cycles: 4, size: 1, flags: &[FlagBits::Z, FlagBits::N, FlagBits::H], execute: exec }
}

// DEC r (r = B, C, D, E, H, L, (HL), A)
pub fn dec_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let r = (instr.opcode >> 3) & 0x07;
        let val = crate::GB::instructions::helpers::read_r(regs, bus, r);
        let res = val.wrapping_sub(1);
        crate::GB::instructions::helpers::write_r(regs, bus, r, res);
        regs.set_flag_z(res == 0);
        regs.set_flag_n(true);
        regs.set_flag_h((val & 0x0F) == 0);
        if r == 6 { 12 } else { 4 }
    }
    Instruction { opcode, name: "DEC r", cycles: 4, size: 1, flags: &[FlagBits::Z, FlagBits::N, FlagBits::H], execute: exec }
}

// DAA - Decimal Adjust Accumulator (0x27)
// Ajusta A para BCD após ADD/ADC/SUB/SBC conforme flags N,H,C
pub fn daa(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let mut a = regs.get_a();
        let n = regs.get_flag_n();
        let mut c = regs.get_flag_c();
        let h = regs.get_flag_h();

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

        regs.set_a(a);
        regs.set_flag_z(a == 0);
        // N permanece como está
        regs.set_flag_h(false);
        regs.set_flag_c(c);
        4
    }
    Instruction { opcode, name: "DAA", cycles: 4, size: 1, flags: &[], execute: exec }
}

// ADD SP,r8 (0xE8) - adiciona signed byte a SP
pub fn add_sp_r8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut crate::GB::registers::Registers, bus: &mut crate::GB::bus::MemoryBus) -> u64 {
        let offset = bus.read(regs.get_pc()) as i8;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let sp = regs.get_sp();
        let result = sp.wrapping_add(offset as i16 as u16);

        // Flags: Z=0, N=0, H e C baseados nos 8 bits inferiores
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(((sp & 0x0F) + ((offset as u8 as u16) & 0x0F)) > 0x0F);
        regs.set_flag_c(((sp & 0xFF) + (offset as u8 as u16)) > 0xFF);

        regs.set_sp(result);
        16
    }
    Instruction { opcode, name: "ADD SP,r8", cycles: 16, size: 2, flags: &[], execute: exec }
}
