// Instruções de LOAD
use super::helpers::{Instruction, read_r, write_r, write_rr};
use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

pub fn ld_r_r(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let dest = (instr.opcode >> 3) & 0x07;
        let src = instr.opcode & 0x07;
        let val = read_r(regs, bus, src);
        write_r(regs, bus, dest, val);
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
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let dest = (instr.opcode >> 3) & 0x07;
        let imm = bus.cpu_read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        write_r(regs, bus, dest, imm);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let imm = bus.cpu_read(regs.get_pc());
        regs.set_pc(regs.get_pc().wrapping_add(1));
        bus.cpu_write(regs.get_hl(), imm);
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
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        write_rr(regs, idx, (hi << 8) | lo);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let val = bus.cpu_read(regs.get_bc());
        regs.set_a(val);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let val = bus.cpu_read(regs.get_de());
        regs.set_a(val);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        bus.cpu_write(regs.get_bc(), regs.get_a());
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        bus.cpu_write(regs.get_de(), regs.get_a());
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let val = bus.cpu_read((hi << 8) | lo);
        regs.set_a(val);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        bus.cpu_write((hi << 8) | lo, regs.get_a());
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let offset = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        bus.cpu_write(0xFF00 + offset, regs.get_a());
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let offset = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let val = bus.cpu_read(0xFF00 + offset);
        regs.set_a(val);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let c = regs.get_c() as u16;
        bus.cpu_write(0xFF00 + c, regs.get_a());
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let c = regs.get_c() as u16;
        let val = bus.cpu_read(0xFF00 + c);
        regs.set_a(val);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let hl = regs.get_hl();
        bus.cpu_write(hl, regs.get_a());
        regs.set_hl(hl.wrapping_add(1));
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let hl = regs.get_hl();
        let val = bus.cpu_read(hl);
        regs.set_a(val);
        regs.set_hl(hl.wrapping_add(1));
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let hl = regs.get_hl();
        bus.cpu_write(hl, regs.get_a());
        regs.set_hl(hl.wrapping_sub(1));
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let hl = regs.get_hl();
        let val = bus.cpu_read(hl);
        regs.set_a(val);
        regs.set_hl(hl.wrapping_sub(1));
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.cpu_read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let addr = (hi << 8) | lo;
        let sp = regs.get_sp();
        bus.cpu_write(addr, (sp & 0xFF) as u8);
        bus.cpu_write(addr.wrapping_add(1), (sp >> 8) as u8);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, _bus: &mut MemoryBus) -> u64 {
        let hl = regs.get_hl();
        regs.set_sp(hl);
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
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let offset = bus.cpu_read(regs.get_pc()) as i8;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let sp = regs.get_sp();
        let result = sp.wrapping_add(offset as i16 as u16);

        // Flags: Z=0, N=0, H e C baseados nos 8 bits inferiores
        regs.set_flag_z(false);
        regs.set_flag_n(false);
        regs.set_flag_h(((sp & 0x0F) + ((offset as u8 as u16) & 0x0F)) > 0x0F);
        regs.set_flag_c(((sp & 0xFF) + (offset as u8 as u16)) > 0xFF);

        regs.set_hl(result);
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
