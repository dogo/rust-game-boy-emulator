// Microcódigos para instruções lógicas e operações em A

use super::{MicroAction, MicroProgram, Reg8};

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

        // AND A,r (0xA0-0xA7)
        0xA0 => Some(&AND_A_B_PROGRAM),
        0xA1 => Some(&AND_A_C_PROGRAM),
        0xA2 => Some(&AND_A_D_PROGRAM),
        0xA3 => Some(&AND_A_E_PROGRAM),
        0xA4 => Some(&AND_A_H_PROGRAM),
        0xA5 => Some(&AND_A_L_PROGRAM),
        0xA7 => Some(&AND_A_A_PROGRAM),
        0xA6 => Some(&AND_A_HL_PROGRAM), // AND A,(HL)
        0xE6 => Some(&AND_A_D8_PROGRAM), // AND A,d8

        // OR A,r (0xB0-0xB7)
        0xB0 => Some(&OR_A_B_PROGRAM),
        0xB1 => Some(&OR_A_C_PROGRAM),
        0xB2 => Some(&OR_A_D_PROGRAM),
        0xB3 => Some(&OR_A_E_PROGRAM),
        0xB4 => Some(&OR_A_H_PROGRAM),
        0xB5 => Some(&OR_A_L_PROGRAM),
        0xB7 => Some(&OR_A_A_PROGRAM),
        0xB6 => Some(&OR_A_HL_PROGRAM), // OR A,(HL)
        0xF6 => Some(&OR_A_D8_PROGRAM), // OR A,d8

        // XOR A,r (0xA8-0xAF)
        0xA8 => Some(&XOR_A_B_PROGRAM),
        0xA9 => Some(&XOR_A_C_PROGRAM),
        0xAA => Some(&XOR_A_D_PROGRAM),
        0xAB => Some(&XOR_A_E_PROGRAM),
        0xAC => Some(&XOR_A_H_PROGRAM),
        0xAD => Some(&XOR_A_L_PROGRAM),
        0xAF => Some(&XOR_A_A_PROGRAM),
        0xAE => Some(&XOR_A_HL_PROGRAM), // XOR A,(HL)
        0xEE => Some(&XOR_A_D8_PROGRAM), // XOR A,d8

        // CP A,r (0xB8-0xBF)
        0xB8 => Some(&CP_A_B_PROGRAM),
        0xB9 => Some(&CP_A_C_PROGRAM),
        0xBA => Some(&CP_A_D_PROGRAM),
        0xBB => Some(&CP_A_E_PROGRAM),
        0xBC => Some(&CP_A_H_PROGRAM),
        0xBD => Some(&CP_A_L_PROGRAM),
        0xBF => Some(&CP_A_A_PROGRAM),
        0xBE => Some(&CP_A_HL_PROGRAM), // CP A,(HL)
        0xFE => Some(&CP_A_D8_PROGRAM), // CP A,d8

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

// === AND A,r ===
const AND_A_B_PROGRAM: MicroProgram = MicroProgram::new(0xA0, "AND A,B", &[MicroAction::AndAToReg { src: Reg8::B }]);
const AND_A_C_PROGRAM: MicroProgram = MicroProgram::new(0xA1, "AND A,C", &[MicroAction::AndAToReg { src: Reg8::C }]);
const AND_A_D_PROGRAM: MicroProgram = MicroProgram::new(0xA2, "AND A,D", &[MicroAction::AndAToReg { src: Reg8::D }]);
const AND_A_E_PROGRAM: MicroProgram = MicroProgram::new(0xA3, "AND A,E", &[MicroAction::AndAToReg { src: Reg8::E }]);
const AND_A_H_PROGRAM: MicroProgram = MicroProgram::new(0xA4, "AND A,H", &[MicroAction::AndAToReg { src: Reg8::H }]);
const AND_A_L_PROGRAM: MicroProgram = MicroProgram::new(0xA5, "AND A,L", &[MicroAction::AndAToReg { src: Reg8::L }]);
const AND_A_A_PROGRAM: MicroProgram = MicroProgram::new(0xA7, "AND A,A", &[MicroAction::AndAToReg { src: Reg8::A }]);

const AND_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0xA6,
    "AND A,(HL)",
    &[MicroAction::AndAToHlValue],
);

const AND_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xE6, "AND A,d8", &[MicroAction::AndAToImm8]);

// === OR A,r ===
const OR_A_B_PROGRAM: MicroProgram = MicroProgram::new(0xB0, "OR A,B", &[MicroAction::OrAToReg { src: Reg8::B }]);
const OR_A_C_PROGRAM: MicroProgram = MicroProgram::new(0xB1, "OR A,C", &[MicroAction::OrAToReg { src: Reg8::C }]);
const OR_A_D_PROGRAM: MicroProgram = MicroProgram::new(0xB2, "OR A,D", &[MicroAction::OrAToReg { src: Reg8::D }]);
const OR_A_E_PROGRAM: MicroProgram = MicroProgram::new(0xB3, "OR A,E", &[MicroAction::OrAToReg { src: Reg8::E }]);
const OR_A_H_PROGRAM: MicroProgram = MicroProgram::new(0xB4, "OR A,H", &[MicroAction::OrAToReg { src: Reg8::H }]);
const OR_A_L_PROGRAM: MicroProgram = MicroProgram::new(0xB5, "OR A,L", &[MicroAction::OrAToReg { src: Reg8::L }]);
const OR_A_A_PROGRAM: MicroProgram = MicroProgram::new(0xB7, "OR A,A", &[MicroAction::OrAToReg { src: Reg8::A }]);

const OR_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0xB6,
    "OR A,(HL)",
    &[MicroAction::OrAToHlValue],
);

const OR_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xF6, "OR A,d8", &[MicroAction::OrAToImm8]);

// === XOR A,r ===
const XOR_A_B_PROGRAM: MicroProgram = MicroProgram::new(0xA8, "XOR A,B", &[MicroAction::XorAToReg { src: Reg8::B }]);
const XOR_A_C_PROGRAM: MicroProgram = MicroProgram::new(0xA9, "XOR A,C", &[MicroAction::XorAToReg { src: Reg8::C }]);
const XOR_A_D_PROGRAM: MicroProgram = MicroProgram::new(0xAA, "XOR A,D", &[MicroAction::XorAToReg { src: Reg8::D }]);
const XOR_A_E_PROGRAM: MicroProgram = MicroProgram::new(0xAB, "XOR A,E", &[MicroAction::XorAToReg { src: Reg8::E }]);
const XOR_A_H_PROGRAM: MicroProgram = MicroProgram::new(0xAC, "XOR A,H", &[MicroAction::XorAToReg { src: Reg8::H }]);
const XOR_A_L_PROGRAM: MicroProgram = MicroProgram::new(0xAD, "XOR A,L", &[MicroAction::XorAToReg { src: Reg8::L }]);
const XOR_A_A_PROGRAM: MicroProgram = MicroProgram::new(0xAF, "XOR A,A", &[MicroAction::XorAToReg { src: Reg8::A }]);

const XOR_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0xAE,
    "XOR A,(HL)",
    &[MicroAction::XorAToHlValue],
);

const XOR_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xEE, "XOR A,d8", &[MicroAction::XorAToImm8]);

// === CP A,r ===
const CP_A_B_PROGRAM: MicroProgram = MicroProgram::new(0xB8, "CP A,B", &[MicroAction::CompareAToReg { src: Reg8::B }]);
const CP_A_C_PROGRAM: MicroProgram = MicroProgram::new(0xB9, "CP A,C", &[MicroAction::CompareAToReg { src: Reg8::C }]);
const CP_A_D_PROGRAM: MicroProgram = MicroProgram::new(0xBA, "CP A,D", &[MicroAction::CompareAToReg { src: Reg8::D }]);
const CP_A_E_PROGRAM: MicroProgram = MicroProgram::new(0xBB, "CP A,E", &[MicroAction::CompareAToReg { src: Reg8::E }]);
const CP_A_H_PROGRAM: MicroProgram = MicroProgram::new(0xBC, "CP A,H", &[MicroAction::CompareAToReg { src: Reg8::H }]);
const CP_A_L_PROGRAM: MicroProgram = MicroProgram::new(0xBD, "CP A,L", &[MicroAction::CompareAToReg { src: Reg8::L }]);
const CP_A_A_PROGRAM: MicroProgram = MicroProgram::new(0xBF, "CP A,A", &[MicroAction::CompareAToReg { src: Reg8::A }]);

const CP_A_HL_PROGRAM: MicroProgram = MicroProgram::new(
    0xBE,
    "CP A,(HL)",
    &[MicroAction::CompareAToHlValue],
);

const CP_A_D8_PROGRAM: MicroProgram = MicroProgram::new(0xFE, "CP A,d8", &[MicroAction::CompareAToImm8]);
