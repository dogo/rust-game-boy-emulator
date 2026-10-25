//! Instruções que a CPU trata fora do microcódigo (`src/GB/microcode/`):
//! DI, EI, HALT, STOP e os opcodes ilegais. Elas gastam só o fetch e seus
//! efeitos ficam em `CPU::execute_next`.

/// Opcodes inválidos que travam a CPU (Pan Docs)
pub const ILLEGAL_OPCODES: [u8; 11] = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

/// Nome das instruções sem microcódigo; `None` para os demais opcodes
pub fn control_name(opcode: u8) -> Option<&'static str> {
    match opcode {
        0xF3 => Some("DI"),
        0xFB => Some("EI"),
        0x76 => Some("HALT"),
        0x10 => Some("STOP"),
        op if ILLEGAL_OPCODES.contains(&op) => Some("ILLEGAL"),
        _ => None,
    }
}
