// Microcódigos para instruções lógicas e operações em A

use super::{MicroAction, MicroProgram};

/// Retorna o microprograma de lógica associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // Rotações simples (4 ciclos, apenas operação interna)
        0x07 => Some(&RLCA_PROGRAM),
        0x0F => Some(&RRCA_PROGRAM),
        0x17 => Some(&RLA_PROGRAM),
        0x1F => Some(&RRA_PROGRAM),

        // Operações de flags
        0x2F => Some(&CPL_PROGRAM),
        0x37 => Some(&SCF_PROGRAM),
        0x3F => Some(&CCF_PROGRAM),

        _ => None,
    }
}

// === Rotações ===
// RLCA - Rotate Left through Carry (bit 7 → Carry, Carry → bit 0)
const RLCA_PROGRAM: MicroProgram = MicroProgram::new(
    0x07,
    "RLCA",
    &[MicroAction::ExecuteRLCA],
);

// RRCA - Rotate Right through Carry (bit 0 → Carry, Carry → bit 7)
const RRCA_PROGRAM: MicroProgram = MicroProgram::new(
    0x0F,
    "RRCA",
    &[MicroAction::ExecuteRRCA],
);

// RLA - Rotate Left A through Carry
const RLA_PROGRAM: MicroProgram = MicroProgram::new(
    0x17,
    "RLA",
    &[MicroAction::ExecuteRLA],
);

// RRA - Rotate Right A through Carry
const RRA_PROGRAM: MicroProgram = MicroProgram::new(
    0x1F,
    "RRA",
    &[MicroAction::ExecuteRRA],
);

// === Operações de flags ===
// CPL - Complement A (A = ~A, set N=1, H=1)
const CPL_PROGRAM: MicroProgram = MicroProgram::new(
    0x2F,
    "CPL",
    &[MicroAction::ExecuteCPL],
);

// SCF - Set Carry Flag (C=1, N=0, H=0)
const SCF_PROGRAM: MicroProgram = MicroProgram::new(
    0x37,
    "SCF",
    &[MicroAction::ExecuteSCF],
);

// CCF - Complement Carry Flag (C=~C, N=0, H=0)
const CCF_PROGRAM: MicroProgram = MicroProgram::new(
    0x3F,
    "CCF",
    &[MicroAction::ExecuteCCF],
);
