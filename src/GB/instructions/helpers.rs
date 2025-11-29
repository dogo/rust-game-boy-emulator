pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub cycles: u8,
    pub size: u8,
    pub flags: &'static [FlagBits],
    pub execute: fn(
        &Instruction,
        &mut crate::GB::registers::Registers,
        &mut crate::GB::bus::MemoryBus,
    ) -> u64,
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
        fn exec_nop(
            _instr: &Instruction,
            _regs: &mut crate::GB::registers::Registers,
            _bus: &mut crate::GB::bus::MemoryBus,
        ) -> u64 {
            4
        }
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
        fn exec_nop(
            _instr: &Instruction,
            _regs: &mut crate::GB::registers::Registers,
            _bus: &mut crate::GB::bus::MemoryBus,
        ) -> u64 {
            0
        }
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
