// Microcódigos para instruções de stack

use super::{JumpCondition, MicroAction, MicroProgram};

/// Retorna o microprograma de stack associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // PUSH rr (0xC5, 0xD5, 0xE5, 0xF5)
        0xC5 => Some(&PUSH_BC_PROGRAM),
        0xD5 => Some(&PUSH_DE_PROGRAM),
        0xE5 => Some(&PUSH_HL_PROGRAM),
        0xF5 => Some(&PUSH_AF_PROGRAM),

        // POP rr (0xC1, 0xD1, 0xE1, 0xF1)
        0xC1 => Some(&POP_BC_PROGRAM),
        0xD1 => Some(&POP_DE_PROGRAM),
        0xE1 => Some(&POP_HL_PROGRAM),
        0xF1 => Some(&POP_AF_PROGRAM),

        // CALL a16
        0xCD => Some(&CALL_A16_PROGRAM),

        // CALL cc,a16
        0xC4 => Some(&CALL_NZ_A16_PROGRAM),
        0xCC => Some(&CALL_Z_A16_PROGRAM),
        0xD4 => Some(&CALL_NC_A16_PROGRAM),
        0xDC => Some(&CALL_C_A16_PROGRAM),

        // RET
        0xC9 => Some(&RET_PROGRAM),

        // RET cc
        0xC0 => Some(&RET_NZ_PROGRAM),
        0xC8 => Some(&RET_Z_PROGRAM),
        0xD0 => Some(&RET_NC_PROGRAM),
        0xD8 => Some(&RET_C_PROGRAM),

        // RETI - será tratado no CPU.rs (habilita IME)
        0xD9 => Some(&RETI_PROGRAM),

        // RST (0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF)
        0xC7 => Some(&RST_00_PROGRAM),
        0xCF => Some(&RST_08_PROGRAM),
        0xD7 => Some(&RST_10_PROGRAM),
        0xDF => Some(&RST_18_PROGRAM),
        0xE7 => Some(&RST_20_PROGRAM),
        0xEF => Some(&RST_28_PROGRAM),
        0xF7 => Some(&RST_30_PROGRAM),
        0xFF => Some(&RST_38_PROGRAM),

        _ => None,
    }
}

// === PUSH rr ===
// idx: 0=BC, 1=DE, 2=HL, 3=AF
const PUSH_BC_PROGRAM: MicroProgram = MicroProgram::new(0xC5, "PUSH BC", &[MicroAction::PushReg16 { idx: 0 }]);
const PUSH_DE_PROGRAM: MicroProgram = MicroProgram::new(0xD5, "PUSH DE", &[MicroAction::PushReg16 { idx: 1 }]);
const PUSH_HL_PROGRAM: MicroProgram = MicroProgram::new(0xE5, "PUSH HL", &[MicroAction::PushReg16 { idx: 2 }]);
const PUSH_AF_PROGRAM: MicroProgram = MicroProgram::new(0xF5, "PUSH AF", &[MicroAction::PushReg16 { idx: 3 }]);

// === POP rr ===
const POP_BC_PROGRAM: MicroProgram = MicroProgram::new(0xC1, "POP BC", &[MicroAction::PopReg16 { idx: 0 }]);
const POP_DE_PROGRAM: MicroProgram = MicroProgram::new(0xD1, "POP DE", &[MicroAction::PopReg16 { idx: 1 }]);
const POP_HL_PROGRAM: MicroProgram = MicroProgram::new(0xE1, "POP HL", &[MicroAction::PopReg16 { idx: 2 }]);
const POP_AF_PROGRAM: MicroProgram = MicroProgram::new(0xF1, "POP AF", &[MicroAction::PopReg16 { idx: 3 }]);

// === CALL ===
const CALL_A16_PROGRAM: MicroProgram = MicroProgram::new(0xCD, "CALL a16", &[MicroAction::CallAbsolute]);

const CALL_NZ_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xC4,
    "CALL NZ,a16",
    &[MicroAction::CallAbsoluteConditional { cond: JumpCondition::NZ }],
);

const CALL_Z_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xCC,
    "CALL Z,a16",
    &[MicroAction::CallAbsoluteConditional { cond: JumpCondition::Z }],
);

const CALL_NC_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xD4,
    "CALL NC,a16",
    &[MicroAction::CallAbsoluteConditional { cond: JumpCondition::NC }],
);

const CALL_C_A16_PROGRAM: MicroProgram = MicroProgram::new(
    0xDC,
    "CALL C,a16",
    &[MicroAction::CallAbsoluteConditional { cond: JumpCondition::C }],
);

// === RET ===
const RET_PROGRAM: MicroProgram = MicroProgram::new(0xC9, "RET", &[MicroAction::Return]);

const RET_NZ_PROGRAM: MicroProgram = MicroProgram::new(
    0xC0,
    "RET NZ",
    &[MicroAction::ReturnConditional { cond: JumpCondition::NZ }],
);

const RET_Z_PROGRAM: MicroProgram = MicroProgram::new(
    0xC8,
    "RET Z",
    &[MicroAction::ReturnConditional { cond: JumpCondition::Z }],
);

const RET_NC_PROGRAM: MicroProgram = MicroProgram::new(
    0xD0,
    "RET NC",
    &[MicroAction::ReturnConditional { cond: JumpCondition::NC }],
);

const RET_C_PROGRAM: MicroProgram = MicroProgram::new(
    0xD8,
    "RET C",
    &[MicroAction::ReturnConditional { cond: JumpCondition::C }],
);

const RETI_PROGRAM: MicroProgram = MicroProgram::new(0xD9, "RETI", &[MicroAction::Return]); // IME será habilitado no CPU.rs

// === RST ===
const RST_00_PROGRAM: MicroProgram = MicroProgram::new(0xC7, "RST 00H", &[MicroAction::Reset { addr: 0x00 }]);
const RST_08_PROGRAM: MicroProgram = MicroProgram::new(0xCF, "RST 08H", &[MicroAction::Reset { addr: 0x08 }]);
const RST_10_PROGRAM: MicroProgram = MicroProgram::new(0xD7, "RST 10H", &[MicroAction::Reset { addr: 0x10 }]);
const RST_18_PROGRAM: MicroProgram = MicroProgram::new(0xDF, "RST 18H", &[MicroAction::Reset { addr: 0x18 }]);
const RST_20_PROGRAM: MicroProgram = MicroProgram::new(0xE7, "RST 20H", &[MicroAction::Reset { addr: 0x20 }]);
const RST_28_PROGRAM: MicroProgram = MicroProgram::new(0xEF, "RST 28H", &[MicroAction::Reset { addr: 0x28 }]);
const RST_30_PROGRAM: MicroProgram = MicroProgram::new(0xF7, "RST 30H", &[MicroAction::Reset { addr: 0x30 }]);
const RST_38_PROGRAM: MicroProgram = MicroProgram::new(0xFF, "RST 38H", &[MicroAction::Reset { addr: 0x38 }]);
