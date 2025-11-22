use super::helpers::Instruction;
use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

pub fn call_a16(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let addr = (hi << 8) | lo;
        let pc = regs.get_pc();
        push_u16(regs, bus, pc);
        regs.set_pc(addr);
        24
    }
    Instruction {
        opcode,
        name: "CALL a16",
        cycles: 24,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn call_cc_a16(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let lo = bus.read(regs.get_pc()) as u16;
        regs.set_pc(regs.get_pc().wrapping_add(1));
        let hi = bus.read(regs.get_pc()) as u16;
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
            let pc = regs.get_pc();
            push_u16(regs, bus, pc);
            regs.set_pc(addr);
            24
        } else {
            12
        }
    }
    Instruction {
        opcode,
        name: "CALL cc,a16",
        cycles: 12,
        size: 3,
        flags: &[],
        execute: exec,
    }
}

pub fn ret(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let addr = pop_u16(regs, bus);
        regs.set_pc(addr);
        16
    }
    Instruction {
        opcode,
        name: "RET",
        cycles: 16,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn ret_cc(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let cond = match (instr.opcode >> 3) & 0x03 {
            0 => !regs.get_flag_z(),
            1 => regs.get_flag_z(),
            2 => !regs.get_flag_c(),
            3 => regs.get_flag_c(),
            _ => false,
        };
        if cond {
            let addr = pop_u16(regs, bus);
            regs.set_pc(addr);
            20
        } else {
            8
        }
    }
    Instruction {
        opcode,
        name: "RET cc",
        cycles: 8,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn reti(opcode: u8) -> Instruction {
    fn exec(_instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let addr = pop_u16(regs, bus);
        regs.set_pc(addr);
        // IME enable must be handled by CPU struct after instruction execution
        16
    }
    Instruction {
        opcode,
        name: "RETI",
        cycles: 16,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn push(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let idx = (instr.opcode >> 4) & 0x03;
        let val = match idx {
            0 => regs.get_bc(),
            1 => regs.get_de(),
            2 => regs.get_hl(),
            3 => regs.get_af(),
            _ => 0,
        };
        push_u16(regs, bus, val);
        16
    }
    Instruction {
        opcode,
        name: "PUSH rr",
        cycles: 16,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn pop(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let val = pop_u16(regs, bus);
        let idx = (instr.opcode >> 4) & 0x03;
        match idx {
            0 => regs.set_bc(val),
            1 => regs.set_de(val),
            2 => regs.set_hl(val),
            3 => regs.set_af(val & 0xFFF0), // Lower 4 bits of F always 0
            _ => {}
        }
        12
    }
    Instruction {
        opcode,
        name: "POP rr",
        cycles: 12,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

pub fn rst(opcode: u8) -> Instruction {
    fn exec(instr: &Instruction, regs: &mut Registers, bus: &mut MemoryBus) -> u64 {
        let addr = (instr.opcode & 0x38) as u16;
        let pc = regs.get_pc();
        push_u16(regs, bus, pc);
        regs.set_pc(addr);
        16
    }
    Instruction {
        opcode,
        name: "RST",
        cycles: 16,
        size: 1,
        flags: &[],
        execute: exec,
    }
}

// Stack helpers migrated to take &mut Registers and &mut MemoryBus
#[inline]
pub fn push_u16(regs: &mut Registers, bus: &mut MemoryBus, val: u16) {
    let mut sp = regs.get_sp();
    sp = sp.wrapping_sub(1);
    bus.write(sp, (val >> 8) as u8);
    sp = sp.wrapping_sub(1);
    bus.write(sp, (val & 0xFF) as u8);
    regs.set_sp(sp);
}

#[inline]
pub fn pop_u16(regs: &mut Registers, bus: &mut MemoryBus) -> u16 {
    let mut sp = regs.get_sp();
    let lo = bus.read(sp) as u16;
    sp = sp.wrapping_add(1);
    let hi = bus.read(sp) as u16;
    sp = sp.wrapping_add(1);
    regs.set_sp(sp);
    (hi << 8) | lo
}
