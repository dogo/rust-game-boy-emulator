// Microcódigos para instruções de salto (jump)

use super::{MicroAction, MicroProgram};

/// Retorna o microprograma de jump associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // JP a16 - Jump to absolute address
        0xC3 => Some(&JP_A16_PROGRAM),

        // JP (HL) - Jump to address in HL
        0xE9 => Some(&JP_HL_PROGRAM),

        // JR r8 - Jump relative (unconditional)
        0x18 => Some(&JR_R8_PROGRAM),

        // JP cc,a16 e JR cc,r8 serão implementados depois (precisam verificar flags)

        _ => None,
    }
}

// JP a16 - Jump to 16-bit absolute address (16 ciclos: 4 fetch + 4 read lo + 4 read hi + 4 idle)
const JP_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xC3,
    "JP a16",
    &[
        MicroAction::FetchImm16AndJump,
    ],
);

// JP (HL) - Jump to address in HL (4 ciclos: apenas operação interna)
const JP_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0xE9,
    "JP (HL)",
    &[MicroAction::JumpToHl],
);

// JR r8 - Jump relative (12 ciclos: 4 fetch opcode + 4 ler offset + 4 calcular e saltar)
const JR_R8_PROGRAM: MicroProgram = MicroProgram::new(
    0x18,
    "JR r8",
    &[MicroAction::JumpRelative],
);
