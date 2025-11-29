// Microcódigos para instruções de salto (jump)

use super::{JumpCondition, MicroAction, MicroProgram};

/// Retorna o microprograma de jump associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // JP a16 - Jump to absolute address
        0xC3 => Some(&JP_A16_PROGRAM),

        // JP (HL) - Jump to address in HL
        0xE9 => Some(&JP_HL_PROGRAM),

        // JR r8 - Jump relative (unconditional)
        0x18 => Some(&JR_R8_PROGRAM),

        // JP cc,a16 (condicional)
        0xC2 => Some(&JP_NZ_A16_PROGRAM),
        0xCA => Some(&JP_Z_A16_PROGRAM),
        0xD2 => Some(&JP_NC_A16_PROGRAM),
        0xDA => Some(&JP_C_A16_PROGRAM),

        // JR cc,r8 (condicional)
        0x20 => Some(&JR_NZ_R8_PROGRAM),
        0x28 => Some(&JR_Z_R8_PROGRAM),
        0x30 => Some(&JR_NC_R8_PROGRAM),
        0x38 => Some(&JR_C_R8_PROGRAM),

        _ => None,
    }
}

// JP a16 - Jump to 16-bit absolute address (16 ciclos: 4 fetch + 4 read lo + 4 read hi + 4 idle)
const JP_A16_PROGRAM: MicroProgram =
    MicroProgram::new(0xC3, "JP a16", &[MicroAction::FetchImm16AndJump]);

// JP (HL) - Jump to address in HL (4 ciclos: apenas operação interna)
const JP_HL_PROGRAM: MicroProgram = MicroProgram::new(0xE9, "JP (HL)", &[MicroAction::JumpToHl]);

// JR r8 - Jump relative (12 ciclos: 4 fetch opcode + 4 ler offset + 4 calcular e saltar)
const JR_R8_PROGRAM: MicroProgram = MicroProgram::new(0x18, "JR r8", &[MicroAction::JumpRelative]);

// === JP cc,a16 ===
const JP_NZ_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xC2,
    "JP NZ,a16",
    &[MicroAction::JumpAbsoluteConditional {
        cond: JumpCondition::NZ,
    }],
);

const JP_Z_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xCA,
    "JP Z,a16",
    &[MicroAction::JumpAbsoluteConditional {
        cond: JumpCondition::Z,
    }],
);

const JP_NC_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xD2,
    "JP NC,a16",
    &[MicroAction::JumpAbsoluteConditional {
        cond: JumpCondition::NC,
    }],
);

const JP_C_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xDA,
    "JP C,a16",
    &[MicroAction::JumpAbsoluteConditional {
        cond: JumpCondition::C,
    }],
);

// === JR cc,r8 ===
const JR_NZ_R8_PROGRAM: MicroProgram = MicroProgram::new(
    0x20,
    "JR NZ,r8",
    &[MicroAction::JumpRelativeConditional {
        cond: JumpCondition::NZ,
    }],
);

const JR_Z_R8_PROGRAM: MicroProgram = MicroProgram::new(
    0x28,
    "JR Z,r8",
    &[MicroAction::JumpRelativeConditional {
        cond: JumpCondition::Z,
    }],
);

const JR_NC_R8_PROGRAM: MicroProgram = MicroProgram::new(
    0x30,
    "JR NC,r8",
    &[MicroAction::JumpRelativeConditional {
        cond: JumpCondition::NC,
    }],
);

const JR_C_R8_PROGRAM: MicroProgram = MicroProgram::new(
    0x38,
    "JR C,r8",
    &[MicroAction::JumpRelativeConditional {
        cond: JumpCondition::C,
    }],
);
