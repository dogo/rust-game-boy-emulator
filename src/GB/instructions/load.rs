// Instruções de LOAD
use crate::GB::CPU::CPU;
use super::helpers::{Instruction, read_r, write_r, write_rr};

pub fn ld_r_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let dest = (instr.opcode >> 3) & 0x07;
        let src = instr.opcode & 0x07;
        let val = read_r(cpu, src);
        write_r(cpu, dest, val);
        if dest == 6 || src == 6 { 8 } else { 4 }
    }
    Instruction {
        opcode,
        name: "LD r,r",
        cycles: 4,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_r_d8(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let dest = (instr.opcode >> 3) & 0x07;
        let imm = cpu.fetch_next();
        write_r(cpu, dest, imm);
        if dest == 6 { 12 } else { 8 }
    }
    Instruction {
        opcode,
        name: "LD r,d8",
        cycles: 8,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_hl_d8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let imm = cpu.fetch_next();
        cpu.ram.write(cpu.registers.get_hl(), imm);
        12
    }
    Instruction {
        opcode,
        name: "LD (HL),d8",
        cycles: 12,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_rr_d16(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, cpu: &mut CPU) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        write_rr(cpu, idx, (hi << 8) | lo);
        12
    }
    Instruction {
        opcode,
        name: "LD rr,d16",
        cycles: 12,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a_bc(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let val = cpu.ram.read(cpu.registers.get_bc());
        cpu.registers.set_a(val);
        8
    }
    Instruction {
        opcode,
        name: "LD A,(BC)",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a_de(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let val = cpu.ram.read(cpu.registers.get_de());
        cpu.registers.set_a(val);
        8
    }
    Instruction {
        opcode,
        name: "LD A,(DE)",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_bc_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.ram.write(cpu.registers.get_bc(), cpu.registers.get_a());
        8
    }
    Instruction {
        opcode,
        name: "LD (BC),A",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_de_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        cpu.ram.write(cpu.registers.get_de(), cpu.registers.get_a());
        8
    }
    Instruction {
        opcode,
        name: "LD (DE),A",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a_a16(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        let val = cpu.ram.read((hi << 8) | lo);
        cpu.registers.set_a(val);
        16
    }
    Instruction {
        opcode,
        name: "LD A,(a16)",
        cycles: 16,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a16_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        cpu.ram.write((hi << 8) | lo, cpu.registers.get_a());
        16
    }
    Instruction {
        opcode,
        name: "LD (a16),A",
        cycles: 16,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn ldh_n_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let offset = cpu.fetch_next() as u16;
        cpu.ram.write(0xFF00 + offset, cpu.registers.get_a());
        12
    }
    Instruction {
        opcode,
        name: "LDH (n),A",
        cycles: 12,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn ldh_a_n(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let offset = cpu.fetch_next() as u16;
        let val = cpu.ram.read(0xFF00 + offset);
        cpu.registers.set_a(val);
        12
    }
    Instruction {
        opcode,
        name: "LDH A,(n)",
        cycles: 12,
        size: 2,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_c_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let c = cpu.registers.get_c() as u16;
        cpu.ram.write(0xFF00 + c, cpu.registers.get_a());
        8
    }
    Instruction {
        opcode,
        name: "LD (C),A",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a_c(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let c = cpu.registers.get_c() as u16;
        let val = cpu.ram.read(0xFF00 + c);
        cpu.registers.set_a(val);
        8
    }
    Instruction {
        opcode,
        name: "LD A,(C)",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ldi_hl_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let hl = cpu.registers.get_hl();
        cpu.ram.write(hl, cpu.registers.get_a());
        cpu.registers.set_hl(hl.wrapping_add(1));
        8
    }
    Instruction {
        opcode,
        name: "LDI (HL),A",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ldi_a_hl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let hl = cpu.registers.get_hl();
        let val = cpu.ram.read(hl);
        cpu.registers.set_a(val);
        cpu.registers.set_hl(hl.wrapping_add(1));
        8
    }
    Instruction {
        opcode,
        name: "LDI A,(HL)",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ldd_hl_a(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let hl = cpu.registers.get_hl();
        cpu.ram.write(hl, cpu.registers.get_a());
        cpu.registers.set_hl(hl.wrapping_sub(1));
        8
    }
    Instruction {
        opcode,
        name: "LDD (HL),A",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ldd_a_hl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let hl = cpu.registers.get_hl();
        let val = cpu.ram.read(hl);
        cpu.registers.set_a(val);
        cpu.registers.set_hl(hl.wrapping_sub(1));
        8
    }
    Instruction {
        opcode,
        name: "LDD A,(HL)",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_a16_sp(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let lo = cpu.fetch_next() as u16;
        let hi = cpu.fetch_next() as u16;
        let addr = (hi << 8) | lo;
        let sp = cpu.registers.get_sp();
        cpu.ram.write(addr, (sp & 0xFF) as u8);
        cpu.ram.write(addr.wrapping_add(1), (sp >> 8) as u8);
        20
    }
    Instruction {
        opcode,
        name: "LD (a16),SP",
        cycles: 20,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn ld_sp_hl(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let hl = cpu.registers.get_hl();
        cpu.registers.set_sp(hl);
        8
    }
    Instruction {
        opcode,
        name: "LD SP,HL",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

// LD HL,SP+r8 (0xF8) - carrega HL com SP + signed byte
pub fn ld_hl_sp_r8(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, cpu: &mut CPU) -> u64 {
        let offset = cpu.fetch_next() as i8;
        let sp = cpu.registers.get_sp();
        let result = sp.wrapping_add(offset as i16 as u16);

        // Flags: Z=0, N=0, H e C baseados nos 8 bits inferiores
        cpu.registers.set_flag_z(false);
        cpu.registers.set_flag_n(false);
        cpu.registers.set_flag_h(((sp & 0x0F) + ((offset as u8 as u16) & 0x0F)) > 0x0F);
        cpu.registers.set_flag_c(((sp & 0xFF) + (offset as u8 as u16)) > 0xFF);

        cpu.registers.set_hl(result);
        12
    }
    Instruction {
        opcode,
        name: "LD HL,SP+r8",
        cycles: 12,
        size: 2,
        flags: &[],
        execute: exec,
    }
}
