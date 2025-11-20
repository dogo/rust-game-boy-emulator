use crate::GB::CPU::CPU;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(&Instruction, &mut CPU) -> u64,
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
        fn exec_nop(_instr: &Instruction, _cpu: &mut CPU) -> u64 { 4 }
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
        fn exec_nop(_instr: &Instruction, _cpu: &mut CPU) -> u64 { 0 }
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
pub fn read_r(cpu: &CPU, idx: u8) -> u8 {
    match idx {
        0 => cpu.registers.get_b(),
        1 => cpu.registers.get_c(),
        2 => cpu.registers.get_d(),
        3 => cpu.registers.get_e(),
        4 => cpu.registers.get_h(),
        5 => cpu.registers.get_l(),
        6 => cpu.ram.read(cpu.registers.get_hl()),
        7 => cpu.registers.get_a(),
        _ => 0,
    }
}

pub fn write_r(cpu: &mut CPU, idx: u8, val: u8) {
    match idx {
        0 => cpu.registers.set_b(val),
        1 => cpu.registers.set_c(val),
        2 => cpu.registers.set_d(val),
        3 => cpu.registers.set_e(val),
        4 => cpu.registers.set_h(val),
        5 => cpu.registers.set_l(val),
        6 => cpu.ram.write(cpu.registers.get_hl(), val),
        7 => cpu.registers.set_a(val),
        _ => {}
    }
}

// Helpers para pares de registradores 16-bit (0-3: BC,DE,HL,SP)
pub fn read_rr(cpu: &CPU, idx: u8) -> u16 {
    match idx {
        0 => cpu.registers.get_bc(),
        1 => cpu.registers.get_de(),
        2 => cpu.registers.get_hl(),
        3 => cpu.registers.get_sp(),
        _ => 0,
    }
}

pub fn write_rr(cpu: &mut CPU, idx: u8, val: u16) {
    match idx {
        0 => cpu.registers.set_bc(val),
        1 => cpu.registers.set_de(val),
        2 => cpu.registers.set_hl(val),
        3 => cpu.registers.set_sp(val),
        _ => {}
    }
}

// Stack helpers
pub fn push_u16(cpu: &mut CPU, val: u16) {
    let sp = cpu.registers.get_sp();
    let sp = sp.wrapping_sub(1);
    cpu.ram.write(sp, (val >> 8) as u8);
    let sp = sp.wrapping_sub(1);
    cpu.ram.write(sp, (val & 0xFF) as u8);
    cpu.registers.set_sp(sp);
}

pub fn pop_u16(cpu: &mut CPU) -> u16 {
    let sp = cpu.registers.get_sp();
    let lo = cpu.ram.read(sp) as u16;
    let sp = sp.wrapping_add(1);
    let hi = cpu.ram.read(sp) as u16;
    let sp = sp.wrapping_add(1);
    cpu.registers.set_sp(sp);
    (hi << 8) | lo
}
