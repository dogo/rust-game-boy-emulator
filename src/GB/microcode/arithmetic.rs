// Microcódigos para instruções aritméticas

use super::{MicroAction, MicroProgram, Reg8};

/// Retorna o microprograma aritmético associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // ADD A,r (0x80-0x87)
        0x80 => Some(&ADD_A_B_PROGRAM),
        0x81 => Some(&ADD_A_C_PROGRAM),
        0x82 => Some(&ADD_A_D_PROGRAM),
        0x83 => Some(&ADD_A_E_PROGRAM),
        0x84 => Some(&ADD_A_H_PROGRAM),
        0x85 => Some(&ADD_A_L_PROGRAM),
        0x87 => Some(&ADD_A_A_PROGRAM),

        // ADD A,(HL) - precisa ler da memória primeiro
        0x86 => Some(&ADD_A_HL_PROGRAM),

        // ADD A,d8
        0xC6 => Some(&ADD_A_D8_PROGRAM),

        // ADC A,r (0x88-0x8F)
        0x88 => Some(&ADC_A_B_PROGRAM),
        0x89 => Some(&ADC_A_C_PROGRAM),
        0x8A => Some(&ADC_A_D_PROGRAM),
        0x8B => Some(&ADC_A_E_PROGRAM),
        0x8C => Some(&ADC_A_H_PROGRAM),
        0x8D => Some(&ADC_A_L_PROGRAM),
        0x8F => Some(&ADC_A_A_PROGRAM),

        // ADC A,(HL)
        0x8E => Some(&ADC_A_HL_PROGRAM),

        // ADC A,d8
        0xCE => Some(&ADC_A_D8_PROGRAM),

        // SUB A,r (0x90-0x97)
        0x90 => Some(&SUB_A_B_PROGRAM),
        0x91 => Some(&SUB_A_C_PROGRAM),
        0x92 => Some(&SUB_A_D_PROGRAM),
        0x93 => Some(&SUB_A_E_PROGRAM),
        0x94 => Some(&SUB_A_H_PROGRAM),
        0x95 => Some(&SUB_A_L_PROGRAM),
        0x97 => Some(&SUB_A_A_PROGRAM),

        // SUB A,(HL)
        0x96 => Some(&SUB_A_HL_PROGRAM),

        // SUB A,d8
        0xD6 => Some(&SUB_A_D8_PROGRAM),

        // SBC A,r (0x98-0x9F)
        0x98 => Some(&SBC_A_B_PROGRAM),
        0x99 => Some(&SBC_A_C_PROGRAM),
        0x9A => Some(&SBC_A_D_PROGRAM),
        0x9B => Some(&SBC_A_E_PROGRAM),
        0x9C => Some(&SBC_A_H_PROGRAM),
        0x9D => Some(&SBC_A_L_PROGRAM),
        0x9F => Some(&SBC_A_A_PROGRAM),

        // SBC A,(HL)
        0x9E => Some(&SBC_A_HL_PROGRAM),

        // SBC A,d8
        0xDE => Some(&SBC_A_D8_PROGRAM),

        _ => None,
    }
}

// === ADD A,r ===
const ADD_A_B_PROGRAM: MicroProgram = MicroProgram::new(0x80, "ADD A,B", &[MicroAction::AddAToReg { src: Reg8::B }]);
const ADD_A_C_PROGRAM: MicroProgram = MicroProgram::new(0x81, "ADD A,C", &[MicroAction::AddAToReg { src: Reg8::C }]);
const ADD_A_D_PROGRAM: MicroProgram = MicroProgram::new(0x82, "ADD A,D", &[MicroAction::AddAToReg { src: Reg8::D }]);
const ADD_A_E_PROGRAM: MicroProgram = MicroProgram::new(0x83, "ADD A,E", &[MicroAction::AddAToReg { src: Reg8::E }]);
const ADD_A_H_PROGRAM: MicroProgram = MicroProgram::new(0x84, "ADD A,H", &[MicroAction::AddAToReg { src: Reg8::H }]);
const ADD_A_L_PROGRAM: MicroProgram = MicroProgram::new(0x85, "ADD A,L", &[MicroAction::AddAToReg { src: Reg8::L }]);
const ADD_A_A_PROGRAM: MicroProgram = MicroProgram::new(0x87, "ADD A,A", &[MicroAction::AddAToReg { src: Reg8::A }]);

// ADD A,(HL)
const ADD_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0x86,
    "ADD A,(HL)",
    &[MicroAction::AddAToHlValue],
);

// ADD A,d8
const ADD_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xC6, "ADD A,d8", &[MicroAction::AddAToImm8]);

// === ADC A,r ===
const ADC_A_B_PROGRAM: MicroProgram = MicroProgram::new(0x88, "ADC A,B", &[MicroAction::AddAWithCarryToReg { src: Reg8::B }]);
const ADC_A_C_PROGRAM: MicroProgram = MicroProgram::new(0x89, "ADC A,C", &[MicroAction::AddAWithCarryToReg { src: Reg8::C }]);
const ADC_A_D_PROGRAM: MicroProgram = MicroProgram::new(0x8A, "ADC A,D", &[MicroAction::AddAWithCarryToReg { src: Reg8::D }]);
const ADC_A_E_PROGRAM: MicroProgram = MicroProgram::new(0x8B, "ADC A,E", &[MicroAction::AddAWithCarryToReg { src: Reg8::E }]);
const ADC_A_H_PROGRAM: MicroProgram = MicroProgram::new(0x8C, "ADC A,H", &[MicroAction::AddAWithCarryToReg { src: Reg8::H }]);
const ADC_A_L_PROGRAM: MicroProgram = MicroProgram::new(0x8D, "ADC A,L", &[MicroAction::AddAWithCarryToReg { src: Reg8::L }]);
const ADC_A_A_PROGRAM: MicroProgram = MicroProgram::new(0x8F, "ADC A,A", &[MicroAction::AddAWithCarryToReg { src: Reg8::A }]);

const ADC_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0x8E,
    "ADC A,(HL)",
    &[MicroAction::AddAWithCarryToHlValue],
);

const ADC_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xCE, "ADC A,d8", &[MicroAction::AddAWithCarryToImm8]);

// === SUB A,r ===
const SUB_A_B_PROGRAM: MicroProgram = MicroProgram::new(0x90, "SUB A,B", &[MicroAction::SubAFromReg { src: Reg8::B }]);
const SUB_A_C_PROGRAM: MicroProgram = MicroProgram::new(0x91, "SUB A,C", &[MicroAction::SubAFromReg { src: Reg8::C }]);
const SUB_A_D_PROGRAM: MicroProgram = MicroProgram::new(0x92, "SUB A,D", &[MicroAction::SubAFromReg { src: Reg8::D }]);
const SUB_A_E_PROGRAM: MicroProgram = MicroProgram::new(0x93, "SUB A,E", &[MicroAction::SubAFromReg { src: Reg8::E }]);
const SUB_A_H_PROGRAM: MicroProgram = MicroProgram::new(0x94, "SUB A,H", &[MicroAction::SubAFromReg { src: Reg8::H }]);
const SUB_A_L_PROGRAM: MicroProgram = MicroProgram::new(0x95, "SUB A,L", &[MicroAction::SubAFromReg { src: Reg8::L }]);
const SUB_A_A_PROGRAM: MicroProgram = MicroProgram::new(0x97, "SUB A,A", &[MicroAction::SubAFromReg { src: Reg8::A }]);

const SUB_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0x96,
    "SUB A,(HL)",
    &[MicroAction::SubAFromHlValue],
);

const SUB_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xD6, "SUB A,d8", &[MicroAction::SubAFromImm8]);

// === SBC A,r ===
const SBC_A_B_PROGRAM: MicroProgram = MicroProgram::new(0x98, "SBC A,B", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::B }]);
const SBC_A_C_PROGRAM: MicroProgram = MicroProgram::new(0x99, "SBC A,C", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::C }]);
const SBC_A_D_PROGRAM: MicroProgram = MicroProgram::new(0x9A, "SBC A,D", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::D }]);
const SBC_A_E_PROGRAM: MicroProgram = MicroProgram::new(0x9B, "SBC A,E", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::E }]);
const SBC_A_H_PROGRAM: MicroProgram = MicroProgram::new(0x9C, "SBC A,H", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::H }]);
const SBC_A_L_PROGRAM: MicroProgram = MicroProgram::new(0x9D, "SBC A,L", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::L }]);
const SBC_A_A_PROGRAM: MicroProgram = MicroProgram::new(0x9F, "SBC A,A", &[MicroAction::SubAWithBorrowFromReg { src: Reg8::A }]);

const SBC_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0x9E,
    "SBC A,(HL)",
    &[MicroAction::SubAWithBorrowFromHlValue],
);

const SBC_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xDE, "SBC A,d8", &[MicroAction::SubAWithBorrowFromImm8]);
