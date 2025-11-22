use crate::GB::CPU::CPU;
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(&Instruction, &mut crate::GB::registers::Registers, &mut crate::GB::bus::MemoryBus) -> u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FlagBits {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

impl Instruction {
    pub fn nop() -> Self {
        fn exec_nop(_instr: &Instruction, _regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 { 4 }
        Instruction {
            opcode: 0x00,
            name: "NOP",
            cycles: 4,
            size: 1,
            flags: &[],
            execute: exec_nop,
        }
    }

    pub fn unknown(opcode: u8) -> Self {
        fn exec_nop(_instr: &Instruction, _regs: &mut crate::GB::registers::Registers, _bus: &mut crate::GB::bus::MemoryBus) -> u64 { 0 }
        Instruction {
            opcode,
            name: "UNKNOWN",
            cycles: 0,
            size: 1,
            flags: &[],
            execute: exec_nop,
        }
    }
}

// Helpers para ler/escrever registradores por índice (0-7: B,C,D,E,H,L,(HL),A)
use crate::GB::bus::MemoryBus;

pub fn read_r(regs: &crate::GB::registers::Registers, bus: &MemoryBus, idx: u8) -> u8 {
    match idx {
        0 => regs.get_b(),
        1 => regs.get_c(),
        2 => regs.get_d(),
        3 => regs.get_e(),
        4 => regs.get_h(),
        5 => regs.get_l(),
        6 => bus.read(regs.get_hl()),
        7 => regs.get_a(),
        _ => 0,
    }
}

pub fn write_r(regs: &mut crate::GB::registers::Registers, bus: &mut MemoryBus, idx: u8, val: u8) {
    match idx {
        0 => regs.set_b(val),
        1 => regs.set_c(val),
        2 => regs.set_d(val),
        3 => regs.set_e(val),
        4 => regs.set_h(val),
        5 => regs.set_l(val),
        6 => bus.write(regs.get_hl(), val),
        7 => regs.set_a(val),
        _ => {}
    }
}

// Helpers para pares de registradores 16-bit (0-3: BC,DE,HL,SP)
pub fn read_rr(regs: &crate::GB::registers::Registers, idx: u8) -> u16 {
    match idx {
        0 => regs.get_bc(),
        1 => regs.get_de(),
        2 => regs.get_hl(),
        3 => regs.get_sp(),
        _ => 0,
    }
}

pub fn write_rr(regs: &mut crate::GB::registers::Registers, idx: u8, val: u16) {
    match idx {
        0 => regs.set_bc(val),
        1 => regs.set_de(val),
        2 => regs.set_hl(val),
        3 => regs.set_sp(val),
        _ => {}
    }
}

// Stack helpers
#[inline]
pub fn push_u16(cpu: &mut CPU, val: u16) {
    cpu.push_u16(val);
}

#[inline]
pub fn pop_u16(cpu: &mut CPU) -> u16 {
    cpu.pop_u16()
}
