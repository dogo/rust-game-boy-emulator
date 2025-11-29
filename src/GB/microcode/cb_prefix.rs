// Microcódigos para instruções CB-prefix
// CB prefix: opcode 0xCB seguido de outro byte que define a operação

use super::{MicroAction, MicroProgram, Reg8};

/// Retorna o microprograma CB associado ao segundo byte (cb_opcode), se existir.
/// Este lookup é usado quando o opcode 0xCB é encontrado.
pub fn lookup(cb_opcode: u8) -> Option<&'static MicroProgram> {
    match cb_opcode {
        // RLC r (0x00-0x07)
        0x00 => Some(&RLC_B),
        0x01 => Some(&RLC_C),
        0x02 => Some(&RLC_D),
        0x03 => Some(&RLC_E),
        0x04 => Some(&RLC_H),
        0x05 => Some(&RLC_L),
        0x06 => Some(&RLC_HL),
        0x07 => Some(&RLC_A),

        // RRC r (0x08-0x0F)
        0x08 => Some(&RRC_B),
        0x09 => Some(&RRC_C),
        0x0A => Some(&RRC_D),
        0x0B => Some(&RRC_E),
        0x0C => Some(&RRC_H),
        0x0D => Some(&RRC_L),
        0x0E => Some(&RRC_HL),
        0x0F => Some(&RRC_A),

        // RL r (0x10-0x17)
        0x10 => Some(&RL_B),
        0x11 => Some(&RL_C),
        0x12 => Some(&RL_D),
        0x13 => Some(&RL_E),
        0x14 => Some(&RL_H),
        0x15 => Some(&RL_L),
        0x16 => Some(&RL_HL),
        0x17 => Some(&RL_A),

        // RR r (0x18-0x1F)
        0x18 => Some(&RR_B),
        0x19 => Some(&RR_C),
        0x1A => Some(&RR_D),
        0x1B => Some(&RR_E),
        0x1C => Some(&RR_H),
        0x1D => Some(&RR_L),
        0x1E => Some(&RR_HL),
        0x1F => Some(&RR_A),

        // SLA r (0x20-0x27)
        0x20 => Some(&SLA_B),
        0x21 => Some(&SLA_C),
        0x22 => Some(&SLA_D),
        0x23 => Some(&SLA_E),
        0x24 => Some(&SLA_H),
        0x25 => Some(&SLA_L),
        0x26 => Some(&SLA_HL),
        0x27 => Some(&SLA_A),

        // SRA r (0x28-0x2F)
        0x28 => Some(&SRA_B),
        0x29 => Some(&SRA_C),
        0x2A => Some(&SRA_D),
        0x2B => Some(&SRA_E),
        0x2C => Some(&SRA_H),
        0x2D => Some(&SRA_L),
        0x2E => Some(&SRA_HL),
        0x2F => Some(&SRA_A),

        // SWAP r (0x30-0x37)
        0x30 => Some(&SWAP_B),
        0x31 => Some(&SWAP_C),
        0x32 => Some(&SWAP_D),
        0x33 => Some(&SWAP_E),
        0x34 => Some(&SWAP_H),
        0x35 => Some(&SWAP_L),
        0x36 => Some(&SWAP_HL),
        0x37 => Some(&SWAP_A),

        // SRL r (0x38-0x3F)
        0x38 => Some(&SRL_B),
        0x39 => Some(&SRL_C),
        0x3A => Some(&SRL_D),
        0x3B => Some(&SRL_E),
        0x3C => Some(&SRL_H),
        0x3D => Some(&SRL_L),
        0x3E => Some(&SRL_HL),
        0x3F => Some(&SRL_A),

        // BIT b,r (0x40-0x7F)
        0x40 => Some(&BIT_0_B),
        0x41 => Some(&BIT_0_C),
        0x42 => Some(&BIT_0_D),
        0x43 => Some(&BIT_0_E),
        0x44 => Some(&BIT_0_H),
        0x45 => Some(&BIT_0_L),
        0x46 => Some(&BIT_0_HL),
        0x47 => Some(&BIT_0_A),
        0x48 => Some(&BIT_1_B),
        0x49 => Some(&BIT_1_C),
        0x4A => Some(&BIT_1_D),
        0x4B => Some(&BIT_1_E),
        0x4C => Some(&BIT_1_H),
        0x4D => Some(&BIT_1_L),
        0x4E => Some(&BIT_1_HL),
        0x4F => Some(&BIT_1_A),
        0x50 => Some(&BIT_2_B),
        0x51 => Some(&BIT_2_C),
        0x52 => Some(&BIT_2_D),
        0x53 => Some(&BIT_2_E),
        0x54 => Some(&BIT_2_H),
        0x55 => Some(&BIT_2_L),
        0x56 => Some(&BIT_2_HL),
        0x57 => Some(&BIT_2_A),
        0x58 => Some(&BIT_3_B),
        0x59 => Some(&BIT_3_C),
        0x5A => Some(&BIT_3_D),
        0x5B => Some(&BIT_3_E),
        0x5C => Some(&BIT_3_H),
        0x5D => Some(&BIT_3_L),
        0x5E => Some(&BIT_3_HL),
        0x5F => Some(&BIT_3_A),
        0x60 => Some(&BIT_4_B),
        0x61 => Some(&BIT_4_C),
        0x62 => Some(&BIT_4_D),
        0x63 => Some(&BIT_4_E),
        0x64 => Some(&BIT_4_H),
        0x65 => Some(&BIT_4_L),
        0x66 => Some(&BIT_4_HL),
        0x67 => Some(&BIT_4_A),
        0x68 => Some(&BIT_5_B),
        0x69 => Some(&BIT_5_C),
        0x6A => Some(&BIT_5_D),
        0x6B => Some(&BIT_5_E),
        0x6C => Some(&BIT_5_H),
        0x6D => Some(&BIT_5_L),
        0x6E => Some(&BIT_5_HL),
        0x6F => Some(&BIT_5_A),
        0x70 => Some(&BIT_6_B),
        0x71 => Some(&BIT_6_C),
        0x72 => Some(&BIT_6_D),
        0x73 => Some(&BIT_6_E),
        0x74 => Some(&BIT_6_H),
        0x75 => Some(&BIT_6_L),
        0x76 => Some(&BIT_6_HL),
        0x77 => Some(&BIT_6_A),
        0x78 => Some(&BIT_7_B),
        0x79 => Some(&BIT_7_C),
        0x7A => Some(&BIT_7_D),
        0x7B => Some(&BIT_7_E),
        0x7C => Some(&BIT_7_H),
        0x7D => Some(&BIT_7_L),
        0x7E => Some(&BIT_7_HL),
        0x7F => Some(&BIT_7_A),

        // RES b,r (0x80-0xBF)
        0x80 => Some(&RES_0_B),
        0x81 => Some(&RES_0_C),
        0x82 => Some(&RES_0_D),
        0x83 => Some(&RES_0_E),
        0x84 => Some(&RES_0_H),
        0x85 => Some(&RES_0_L),
        0x86 => Some(&RES_0_HL),
        0x87 => Some(&RES_0_A),
        0x88 => Some(&RES_1_B),
        0x89 => Some(&RES_1_C),
        0x8A => Some(&RES_1_D),
        0x8B => Some(&RES_1_E),
        0x8C => Some(&RES_1_H),
        0x8D => Some(&RES_1_L),
        0x8E => Some(&RES_1_HL),
        0x8F => Some(&RES_1_A),
        0x90 => Some(&RES_2_B),
        0x91 => Some(&RES_2_C),
        0x92 => Some(&RES_2_D),
        0x93 => Some(&RES_2_E),
        0x94 => Some(&RES_2_H),
        0x95 => Some(&RES_2_L),
        0x96 => Some(&RES_2_HL),
        0x97 => Some(&RES_2_A),
        0x98 => Some(&RES_3_B),
        0x99 => Some(&RES_3_C),
        0x9A => Some(&RES_3_D),
        0x9B => Some(&RES_3_E),
        0x9C => Some(&RES_3_H),
        0x9D => Some(&RES_3_L),
        0x9E => Some(&RES_3_HL),
        0x9F => Some(&RES_3_A),
        0xA0 => Some(&RES_4_B),
        0xA1 => Some(&RES_4_C),
        0xA2 => Some(&RES_4_D),
        0xA3 => Some(&RES_4_E),
        0xA4 => Some(&RES_4_H),
        0xA5 => Some(&RES_4_L),
        0xA6 => Some(&RES_4_HL),
        0xA7 => Some(&RES_4_A),
        0xA8 => Some(&RES_5_B),
        0xA9 => Some(&RES_5_C),
        0xAA => Some(&RES_5_D),
        0xAB => Some(&RES_5_E),
        0xAC => Some(&RES_5_H),
        0xAD => Some(&RES_5_L),
        0xAE => Some(&RES_5_HL),
        0xAF => Some(&RES_5_A),
        0xB0 => Some(&RES_6_B),
        0xB1 => Some(&RES_6_C),
        0xB2 => Some(&RES_6_D),
        0xB3 => Some(&RES_6_E),
        0xB4 => Some(&RES_6_H),
        0xB5 => Some(&RES_6_L),
        0xB6 => Some(&RES_6_HL),
        0xB7 => Some(&RES_6_A),
        0xB8 => Some(&RES_7_B),
        0xB9 => Some(&RES_7_C),
        0xBA => Some(&RES_7_D),
        0xBB => Some(&RES_7_E),
        0xBC => Some(&RES_7_H),
        0xBD => Some(&RES_7_L),
        0xBE => Some(&RES_7_HL),
        0xBF => Some(&RES_7_A),

        // SET b,r (0xC0-0xFF)
        0xC0 => Some(&SET_0_B),
        0xC1 => Some(&SET_0_C),
        0xC2 => Some(&SET_0_D),
        0xC3 => Some(&SET_0_E),
        0xC4 => Some(&SET_0_H),
        0xC5 => Some(&SET_0_L),
        0xC6 => Some(&SET_0_HL),
        0xC7 => Some(&SET_0_A),
        0xC8 => Some(&SET_1_B),
        0xC9 => Some(&SET_1_C),
        0xCA => Some(&SET_1_D),
        0xCB => Some(&SET_1_E),
        0xCC => Some(&SET_1_H),
        0xCD => Some(&SET_1_L),
        0xCE => Some(&SET_1_HL),
        0xCF => Some(&SET_1_A),
        0xD0 => Some(&SET_2_B),
        0xD1 => Some(&SET_2_C),
        0xD2 => Some(&SET_2_D),
        0xD3 => Some(&SET_2_E),
        0xD4 => Some(&SET_2_H),
        0xD5 => Some(&SET_2_L),
        0xD6 => Some(&SET_2_HL),
        0xD7 => Some(&SET_2_A),
        0xD8 => Some(&SET_3_B),
        0xD9 => Some(&SET_3_C),
        0xDA => Some(&SET_3_D),
        0xDB => Some(&SET_3_E),
        0xDC => Some(&SET_3_H),
        0xDD => Some(&SET_3_L),
        0xDE => Some(&SET_3_HL),
        0xDF => Some(&SET_3_A),
        0xE0 => Some(&SET_4_B),
        0xE1 => Some(&SET_4_C),
        0xE2 => Some(&SET_4_D),
        0xE3 => Some(&SET_4_E),
        0xE4 => Some(&SET_4_H),
        0xE5 => Some(&SET_4_L),
        0xE6 => Some(&SET_4_HL),
        0xE7 => Some(&SET_4_A),
        0xE8 => Some(&SET_5_B),
        0xE9 => Some(&SET_5_C),
        0xEA => Some(&SET_5_D),
        0xEB => Some(&SET_5_E),
        0xEC => Some(&SET_5_H),
        0xED => Some(&SET_5_L),
        0xEE => Some(&SET_5_HL),
        0xEF => Some(&SET_5_A),
        0xF0 => Some(&SET_6_B),
        0xF1 => Some(&SET_6_C),
        0xF2 => Some(&SET_6_D),
        0xF3 => Some(&SET_6_E),
        0xF4 => Some(&SET_6_H),
        0xF5 => Some(&SET_6_L),
        0xF6 => Some(&SET_6_HL),
        0xF7 => Some(&SET_6_A),
        0xF8 => Some(&SET_7_B),
        0xF9 => Some(&SET_7_C),
        0xFA => Some(&SET_7_D),
        0xFB => Some(&SET_7_E),
        0xFC => Some(&SET_7_H),
        0xFD => Some(&SET_7_L),
        0xFE => Some(&SET_7_HL),
        0xFF => Some(&SET_7_A),
    }
}

// === RLC (Rotate Left Circular) ===
const RLC_B: MicroProgram =
    MicroProgram::new(0x00, "RLC B", &[MicroAction::ExecuteRLC { reg: Reg8::B }]);
const RLC_C: MicroProgram =
    MicroProgram::new(0x01, "RLC C", &[MicroAction::ExecuteRLC { reg: Reg8::C }]);
const RLC_D: MicroProgram =
    MicroProgram::new(0x02, "RLC D", &[MicroAction::ExecuteRLC { reg: Reg8::D }]);
const RLC_E: MicroProgram =
    MicroProgram::new(0x03, "RLC E", &[MicroAction::ExecuteRLC { reg: Reg8::E }]);
const RLC_H: MicroProgram =
    MicroProgram::new(0x04, "RLC H", &[MicroAction::ExecuteRLC { reg: Reg8::H }]);
const RLC_L: MicroProgram =
    MicroProgram::new(0x05, "RLC L", &[MicroAction::ExecuteRLC { reg: Reg8::L }]);
const RLC_HL: MicroProgram = MicroProgram::new(0x06, "RLC (HL)", &[MicroAction::ExecuteRLCHl]);
const RLC_A: MicroProgram =
    MicroProgram::new(0x07, "RLC A", &[MicroAction::ExecuteRLC { reg: Reg8::A }]);

// === RRC (Rotate Right Circular) ===
const RRC_B: MicroProgram =
    MicroProgram::new(0x08, "RRC B", &[MicroAction::ExecuteRRC { reg: Reg8::B }]);
const RRC_C: MicroProgram =
    MicroProgram::new(0x09, "RRC C", &[MicroAction::ExecuteRRC { reg: Reg8::C }]);
const RRC_D: MicroProgram =
    MicroProgram::new(0x0A, "RRC D", &[MicroAction::ExecuteRRC { reg: Reg8::D }]);
const RRC_E: MicroProgram =
    MicroProgram::new(0x0B, "RRC E", &[MicroAction::ExecuteRRC { reg: Reg8::E }]);
const RRC_H: MicroProgram =
    MicroProgram::new(0x0C, "RRC H", &[MicroAction::ExecuteRRC { reg: Reg8::H }]);
const RRC_L: MicroProgram =
    MicroProgram::new(0x0D, "RRC L", &[MicroAction::ExecuteRRC { reg: Reg8::L }]);
const RRC_HL: MicroProgram = MicroProgram::new(0x0E, "RRC (HL)", &[MicroAction::ExecuteRRCHl]);
const RRC_A: MicroProgram =
    MicroProgram::new(0x0F, "RRC A", &[MicroAction::ExecuteRRC { reg: Reg8::A }]);

// === RL (Rotate Left through Carry) ===
const RL_B: MicroProgram =
    MicroProgram::new(0x10, "RL B", &[MicroAction::ExecuteRL { reg: Reg8::B }]);
const RL_C: MicroProgram =
    MicroProgram::new(0x11, "RL C", &[MicroAction::ExecuteRL { reg: Reg8::C }]);
const RL_D: MicroProgram =
    MicroProgram::new(0x12, "RL D", &[MicroAction::ExecuteRL { reg: Reg8::D }]);
const RL_E: MicroProgram =
    MicroProgram::new(0x13, "RL E", &[MicroAction::ExecuteRL { reg: Reg8::E }]);
const RL_H: MicroProgram =
    MicroProgram::new(0x14, "RL H", &[MicroAction::ExecuteRL { reg: Reg8::H }]);
const RL_L: MicroProgram =
    MicroProgram::new(0x15, "RL L", &[MicroAction::ExecuteRL { reg: Reg8::L }]);
const RL_HL: MicroProgram = MicroProgram::new(0x16, "RL (HL)", &[MicroAction::ExecuteRLHl]);
const RL_A: MicroProgram =
    MicroProgram::new(0x17, "RL A", &[MicroAction::ExecuteRL { reg: Reg8::A }]);

// === RR (Rotate Right through Carry) ===
const RR_B: MicroProgram =
    MicroProgram::new(0x18, "RR B", &[MicroAction::ExecuteRR { reg: Reg8::B }]);
const RR_C: MicroProgram =
    MicroProgram::new(0x19, "RR C", &[MicroAction::ExecuteRR { reg: Reg8::C }]);
const RR_D: MicroProgram =
    MicroProgram::new(0x1A, "RR D", &[MicroAction::ExecuteRR { reg: Reg8::D }]);
const RR_E: MicroProgram =
    MicroProgram::new(0x1B, "RR E", &[MicroAction::ExecuteRR { reg: Reg8::E }]);
const RR_H: MicroProgram =
    MicroProgram::new(0x1C, "RR H", &[MicroAction::ExecuteRR { reg: Reg8::H }]);
const RR_L: MicroProgram =
    MicroProgram::new(0x1D, "RR L", &[MicroAction::ExecuteRR { reg: Reg8::L }]);
const RR_HL: MicroProgram = MicroProgram::new(0x1E, "RR (HL)", &[MicroAction::ExecuteRRHl]);
const RR_A: MicroProgram =
    MicroProgram::new(0x1F, "RR A", &[MicroAction::ExecuteRR { reg: Reg8::A }]);

// === SLA (Shift Left Arithmetic) ===
const SLA_B: MicroProgram =
    MicroProgram::new(0x20, "SLA B", &[MicroAction::ExecuteSLA { reg: Reg8::B }]);
const SLA_C: MicroProgram =
    MicroProgram::new(0x21, "SLA C", &[MicroAction::ExecuteSLA { reg: Reg8::C }]);
const SLA_D: MicroProgram =
    MicroProgram::new(0x22, "SLA D", &[MicroAction::ExecuteSLA { reg: Reg8::D }]);
const SLA_E: MicroProgram =
    MicroProgram::new(0x23, "SLA E", &[MicroAction::ExecuteSLA { reg: Reg8::E }]);
const SLA_H: MicroProgram =
    MicroProgram::new(0x24, "SLA H", &[MicroAction::ExecuteSLA { reg: Reg8::H }]);
const SLA_L: MicroProgram =
    MicroProgram::new(0x25, "SLA L", &[MicroAction::ExecuteSLA { reg: Reg8::L }]);
const SLA_HL: MicroProgram = MicroProgram::new(0x26, "SLA (HL)", &[MicroAction::ExecuteSLAHl]);
const SLA_A: MicroProgram =
    MicroProgram::new(0x27, "SLA A", &[MicroAction::ExecuteSLA { reg: Reg8::A }]);

// === SRA (Shift Right Arithmetic - preserva MSB) ===
const SRA_B: MicroProgram =
    MicroProgram::new(0x28, "SRA B", &[MicroAction::ExecuteSRA { reg: Reg8::B }]);
const SRA_C: MicroProgram =
    MicroProgram::new(0x29, "SRA C", &[MicroAction::ExecuteSRA { reg: Reg8::C }]);
const SRA_D: MicroProgram =
    MicroProgram::new(0x2A, "SRA D", &[MicroAction::ExecuteSRA { reg: Reg8::D }]);
const SRA_E: MicroProgram =
    MicroProgram::new(0x2B, "SRA E", &[MicroAction::ExecuteSRA { reg: Reg8::E }]);
const SRA_H: MicroProgram =
    MicroProgram::new(0x2C, "SRA H", &[MicroAction::ExecuteSRA { reg: Reg8::H }]);
const SRA_L: MicroProgram =
    MicroProgram::new(0x2D, "SRA L", &[MicroAction::ExecuteSRA { reg: Reg8::L }]);
const SRA_HL: MicroProgram = MicroProgram::new(0x2E, "SRA (HL)", &[MicroAction::ExecuteSRAHl]);
const SRA_A: MicroProgram =
    MicroProgram::new(0x2F, "SRA A", &[MicroAction::ExecuteSRA { reg: Reg8::A }]);

// === SWAP (troca nibbles) ===
const SWAP_B: MicroProgram =
    MicroProgram::new(0x30, "SWAP B", &[MicroAction::ExecuteSWAP { reg: Reg8::B }]);
const SWAP_C: MicroProgram =
    MicroProgram::new(0x31, "SWAP C", &[MicroAction::ExecuteSWAP { reg: Reg8::C }]);
const SWAP_D: MicroProgram =
    MicroProgram::new(0x32, "SWAP D", &[MicroAction::ExecuteSWAP { reg: Reg8::D }]);
const SWAP_E: MicroProgram =
    MicroProgram::new(0x33, "SWAP E", &[MicroAction::ExecuteSWAP { reg: Reg8::E }]);
const SWAP_H: MicroProgram =
    MicroProgram::new(0x34, "SWAP H", &[MicroAction::ExecuteSWAP { reg: Reg8::H }]);
const SWAP_L: MicroProgram =
    MicroProgram::new(0x35, "SWAP L", &[MicroAction::ExecuteSWAP { reg: Reg8::L }]);
const SWAP_HL: MicroProgram = MicroProgram::new(0x36, "SWAP (HL)", &[MicroAction::ExecuteSWAPHl]);
const SWAP_A: MicroProgram =
    MicroProgram::new(0x37, "SWAP A", &[MicroAction::ExecuteSWAP { reg: Reg8::A }]);

// === SRL (Shift Right Logical - zero fill) ===
const SRL_B: MicroProgram =
    MicroProgram::new(0x38, "SRL B", &[MicroAction::ExecuteSRL { reg: Reg8::B }]);
const SRL_C: MicroProgram =
    MicroProgram::new(0x39, "SRL C", &[MicroAction::ExecuteSRL { reg: Reg8::C }]);
const SRL_D: MicroProgram =
    MicroProgram::new(0x3A, "SRL D", &[MicroAction::ExecuteSRL { reg: Reg8::D }]);
const SRL_E: MicroProgram =
    MicroProgram::new(0x3B, "SRL E", &[MicroAction::ExecuteSRL { reg: Reg8::E }]);
const SRL_H: MicroProgram =
    MicroProgram::new(0x3C, "SRL H", &[MicroAction::ExecuteSRL { reg: Reg8::H }]);
const SRL_L: MicroProgram =
    MicroProgram::new(0x3D, "SRL L", &[MicroAction::ExecuteSRL { reg: Reg8::L }]);
const SRL_HL: MicroProgram = MicroProgram::new(0x3E, "SRL (HL)", &[MicroAction::ExecuteSRLHl]);
const SRL_A: MicroProgram =
    MicroProgram::new(0x3F, "SRL A", &[MicroAction::ExecuteSRL { reg: Reg8::A }]);

// === BIT b,r (testa bit) ===
// Bit 0
const BIT_0_B: MicroProgram = MicroProgram::new(
    0x40,
    "BIT 0,B",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::B,
    }],
);
const BIT_0_C: MicroProgram = MicroProgram::new(
    0x41,
    "BIT 0,C",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::C,
    }],
);
const BIT_0_D: MicroProgram = MicroProgram::new(
    0x42,
    "BIT 0,D",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::D,
    }],
);
const BIT_0_E: MicroProgram = MicroProgram::new(
    0x43,
    "BIT 0,E",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::E,
    }],
);
const BIT_0_H: MicroProgram = MicroProgram::new(
    0x44,
    "BIT 0,H",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::H,
    }],
);
const BIT_0_L: MicroProgram = MicroProgram::new(
    0x45,
    "BIT 0,L",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::L,
    }],
);
const BIT_0_HL: MicroProgram =
    MicroProgram::new(0x46, "BIT 0,(HL)", &[MicroAction::TestBitHl { bit: 0 }]);
const BIT_0_A: MicroProgram = MicroProgram::new(
    0x47,
    "BIT 0,A",
    &[MicroAction::TestBit {
        bit: 0,
        reg: Reg8::A,
    }],
);

// Bit 1
const BIT_1_B: MicroProgram = MicroProgram::new(
    0x48,
    "BIT 1,B",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::B,
    }],
);
const BIT_1_C: MicroProgram = MicroProgram::new(
    0x49,
    "BIT 1,C",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::C,
    }],
);
const BIT_1_D: MicroProgram = MicroProgram::new(
    0x4A,
    "BIT 1,D",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::D,
    }],
);
const BIT_1_E: MicroProgram = MicroProgram::new(
    0x4B,
    "BIT 1,E",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::E,
    }],
);
const BIT_1_H: MicroProgram = MicroProgram::new(
    0x4C,
    "BIT 1,H",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::H,
    }],
);
const BIT_1_L: MicroProgram = MicroProgram::new(
    0x4D,
    "BIT 1,L",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::L,
    }],
);
const BIT_1_HL: MicroProgram =
    MicroProgram::new(0x4E, "BIT 1,(HL)", &[MicroAction::TestBitHl { bit: 1 }]);
const BIT_1_A: MicroProgram = MicroProgram::new(
    0x4F,
    "BIT 1,A",
    &[MicroAction::TestBit {
        bit: 1,
        reg: Reg8::A,
    }],
);

// Bit 2
const BIT_2_B: MicroProgram = MicroProgram::new(
    0x50,
    "BIT 2,B",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::B,
    }],
);
const BIT_2_C: MicroProgram = MicroProgram::new(
    0x51,
    "BIT 2,C",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::C,
    }],
);
const BIT_2_D: MicroProgram = MicroProgram::new(
    0x52,
    "BIT 2,D",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::D,
    }],
);
const BIT_2_E: MicroProgram = MicroProgram::new(
    0x53,
    "BIT 2,E",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::E,
    }],
);
const BIT_2_H: MicroProgram = MicroProgram::new(
    0x54,
    "BIT 2,H",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::H,
    }],
);
const BIT_2_L: MicroProgram = MicroProgram::new(
    0x55,
    "BIT 2,L",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::L,
    }],
);
const BIT_2_HL: MicroProgram =
    MicroProgram::new(0x56, "BIT 2,(HL)", &[MicroAction::TestBitHl { bit: 2 }]);
const BIT_2_A: MicroProgram = MicroProgram::new(
    0x57,
    "BIT 2,A",
    &[MicroAction::TestBit {
        bit: 2,
        reg: Reg8::A,
    }],
);

// Bit 3
const BIT_3_B: MicroProgram = MicroProgram::new(
    0x58,
    "BIT 3,B",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::B,
    }],
);
const BIT_3_C: MicroProgram = MicroProgram::new(
    0x59,
    "BIT 3,C",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::C,
    }],
);
const BIT_3_D: MicroProgram = MicroProgram::new(
    0x5A,
    "BIT 3,D",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::D,
    }],
);
const BIT_3_E: MicroProgram = MicroProgram::new(
    0x5B,
    "BIT 3,E",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::E,
    }],
);
const BIT_3_H: MicroProgram = MicroProgram::new(
    0x5C,
    "BIT 3,H",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::H,
    }],
);
const BIT_3_L: MicroProgram = MicroProgram::new(
    0x5D,
    "BIT 3,L",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::L,
    }],
);
const BIT_3_HL: MicroProgram =
    MicroProgram::new(0x5E, "BIT 3,(HL)", &[MicroAction::TestBitHl { bit: 3 }]);
const BIT_3_A: MicroProgram = MicroProgram::new(
    0x5F,
    "BIT 3,A",
    &[MicroAction::TestBit {
        bit: 3,
        reg: Reg8::A,
    }],
);

// Bit 4
const BIT_4_B: MicroProgram = MicroProgram::new(
    0x60,
    "BIT 4,B",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::B,
    }],
);
const BIT_4_C: MicroProgram = MicroProgram::new(
    0x61,
    "BIT 4,C",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::C,
    }],
);
const BIT_4_D: MicroProgram = MicroProgram::new(
    0x62,
    "BIT 4,D",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::D,
    }],
);
const BIT_4_E: MicroProgram = MicroProgram::new(
    0x63,
    "BIT 4,E",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::E,
    }],
);
const BIT_4_H: MicroProgram = MicroProgram::new(
    0x64,
    "BIT 4,H",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::H,
    }],
);
const BIT_4_L: MicroProgram = MicroProgram::new(
    0x65,
    "BIT 4,L",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::L,
    }],
);
const BIT_4_HL: MicroProgram =
    MicroProgram::new(0x66, "BIT 4,(HL)", &[MicroAction::TestBitHl { bit: 4 }]);
const BIT_4_A: MicroProgram = MicroProgram::new(
    0x67,
    "BIT 4,A",
    &[MicroAction::TestBit {
        bit: 4,
        reg: Reg8::A,
    }],
);

// Bit 5
const BIT_5_B: MicroProgram = MicroProgram::new(
    0x68,
    "BIT 5,B",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::B,
    }],
);
const BIT_5_C: MicroProgram = MicroProgram::new(
    0x69,
    "BIT 5,C",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::C,
    }],
);
const BIT_5_D: MicroProgram = MicroProgram::new(
    0x6A,
    "BIT 5,D",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::D,
    }],
);
const BIT_5_E: MicroProgram = MicroProgram::new(
    0x6B,
    "BIT 5,E",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::E,
    }],
);
const BIT_5_H: MicroProgram = MicroProgram::new(
    0x6C,
    "BIT 5,H",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::H,
    }],
);
const BIT_5_L: MicroProgram = MicroProgram::new(
    0x6D,
    "BIT 5,L",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::L,
    }],
);
const BIT_5_HL: MicroProgram =
    MicroProgram::new(0x6E, "BIT 5,(HL)", &[MicroAction::TestBitHl { bit: 5 }]);
const BIT_5_A: MicroProgram = MicroProgram::new(
    0x6F,
    "BIT 5,A",
    &[MicroAction::TestBit {
        bit: 5,
        reg: Reg8::A,
    }],
);

// Bit 6
const BIT_6_B: MicroProgram = MicroProgram::new(
    0x70,
    "BIT 6,B",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::B,
    }],
);
const BIT_6_C: MicroProgram = MicroProgram::new(
    0x71,
    "BIT 6,C",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::C,
    }],
);
const BIT_6_D: MicroProgram = MicroProgram::new(
    0x72,
    "BIT 6,D",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::D,
    }],
);
const BIT_6_E: MicroProgram = MicroProgram::new(
    0x73,
    "BIT 6,E",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::E,
    }],
);
const BIT_6_H: MicroProgram = MicroProgram::new(
    0x74,
    "BIT 6,H",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::H,
    }],
);
const BIT_6_L: MicroProgram = MicroProgram::new(
    0x75,
    "BIT 6,L",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::L,
    }],
);
const BIT_6_HL: MicroProgram =
    MicroProgram::new(0x76, "BIT 6,(HL)", &[MicroAction::TestBitHl { bit: 6 }]);
const BIT_6_A: MicroProgram = MicroProgram::new(
    0x77,
    "BIT 6,A",
    &[MicroAction::TestBit {
        bit: 6,
        reg: Reg8::A,
    }],
);

// Bit 7
const BIT_7_B: MicroProgram = MicroProgram::new(
    0x78,
    "BIT 7,B",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::B,
    }],
);
const BIT_7_C: MicroProgram = MicroProgram::new(
    0x79,
    "BIT 7,C",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::C,
    }],
);
const BIT_7_D: MicroProgram = MicroProgram::new(
    0x7A,
    "BIT 7,D",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::D,
    }],
);
const BIT_7_E: MicroProgram = MicroProgram::new(
    0x7B,
    "BIT 7,E",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::E,
    }],
);
const BIT_7_H: MicroProgram = MicroProgram::new(
    0x7C,
    "BIT 7,H",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::H,
    }],
);
const BIT_7_L: MicroProgram = MicroProgram::new(
    0x7D,
    "BIT 7,L",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::L,
    }],
);
const BIT_7_HL: MicroProgram =
    MicroProgram::new(0x7E, "BIT 7,(HL)", &[MicroAction::TestBitHl { bit: 7 }]);
const BIT_7_A: MicroProgram = MicroProgram::new(
    0x7F,
    "BIT 7,A",
    &[MicroAction::TestBit {
        bit: 7,
        reg: Reg8::A,
    }],
);

// === RES b,r (reseta bit) ===
// Bit 0
const RES_0_B: MicroProgram = MicroProgram::new(
    0x80,
    "RES 0,B",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::B,
    }],
);
const RES_0_C: MicroProgram = MicroProgram::new(
    0x81,
    "RES 0,C",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::C,
    }],
);
const RES_0_D: MicroProgram = MicroProgram::new(
    0x82,
    "RES 0,D",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::D,
    }],
);
const RES_0_E: MicroProgram = MicroProgram::new(
    0x83,
    "RES 0,E",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::E,
    }],
);
const RES_0_H: MicroProgram = MicroProgram::new(
    0x84,
    "RES 0,H",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::H,
    }],
);
const RES_0_L: MicroProgram = MicroProgram::new(
    0x85,
    "RES 0,L",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::L,
    }],
);
const RES_0_HL: MicroProgram =
    MicroProgram::new(0x86, "RES 0,(HL)", &[MicroAction::ResetBitHl { bit: 0 }]);
const RES_0_A: MicroProgram = MicroProgram::new(
    0x87,
    "RES 0,A",
    &[MicroAction::ResetBit {
        bit: 0,
        reg: Reg8::A,
    }],
);

// Bit 1
const RES_1_B: MicroProgram = MicroProgram::new(
    0x88,
    "RES 1,B",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::B,
    }],
);
const RES_1_C: MicroProgram = MicroProgram::new(
    0x89,
    "RES 1,C",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::C,
    }],
);
const RES_1_D: MicroProgram = MicroProgram::new(
    0x8A,
    "RES 1,D",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::D,
    }],
);
const RES_1_E: MicroProgram = MicroProgram::new(
    0x8B,
    "RES 1,E",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::E,
    }],
);
const RES_1_H: MicroProgram = MicroProgram::new(
    0x8C,
    "RES 1,H",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::H,
    }],
);
const RES_1_L: MicroProgram = MicroProgram::new(
    0x8D,
    "RES 1,L",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::L,
    }],
);
const RES_1_HL: MicroProgram =
    MicroProgram::new(0x8E, "RES 1,(HL)", &[MicroAction::ResetBitHl { bit: 1 }]);
const RES_1_A: MicroProgram = MicroProgram::new(
    0x8F,
    "RES 1,A",
    &[MicroAction::ResetBit {
        bit: 1,
        reg: Reg8::A,
    }],
);

// Bit 2
const RES_2_B: MicroProgram = MicroProgram::new(
    0x90,
    "RES 2,B",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::B,
    }],
);
const RES_2_C: MicroProgram = MicroProgram::new(
    0x91,
    "RES 2,C",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::C,
    }],
);
const RES_2_D: MicroProgram = MicroProgram::new(
    0x92,
    "RES 2,D",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::D,
    }],
);
const RES_2_E: MicroProgram = MicroProgram::new(
    0x93,
    "RES 2,E",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::E,
    }],
);
const RES_2_H: MicroProgram = MicroProgram::new(
    0x94,
    "RES 2,H",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::H,
    }],
);
const RES_2_L: MicroProgram = MicroProgram::new(
    0x95,
    "RES 2,L",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::L,
    }],
);
const RES_2_HL: MicroProgram =
    MicroProgram::new(0x96, "RES 2,(HL)", &[MicroAction::ResetBitHl { bit: 2 }]);
const RES_2_A: MicroProgram = MicroProgram::new(
    0x97,
    "RES 2,A",
    &[MicroAction::ResetBit {
        bit: 2,
        reg: Reg8::A,
    }],
);

// Bit 3
const RES_3_B: MicroProgram = MicroProgram::new(
    0x98,
    "RES 3,B",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::B,
    }],
);
const RES_3_C: MicroProgram = MicroProgram::new(
    0x99,
    "RES 3,C",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::C,
    }],
);
const RES_3_D: MicroProgram = MicroProgram::new(
    0x9A,
    "RES 3,D",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::D,
    }],
);
const RES_3_E: MicroProgram = MicroProgram::new(
    0x9B,
    "RES 3,E",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::E,
    }],
);
const RES_3_H: MicroProgram = MicroProgram::new(
    0x9C,
    "RES 3,H",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::H,
    }],
);
const RES_3_L: MicroProgram = MicroProgram::new(
    0x9D,
    "RES 3,L",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::L,
    }],
);
const RES_3_HL: MicroProgram =
    MicroProgram::new(0x9E, "RES 3,(HL)", &[MicroAction::ResetBitHl { bit: 3 }]);
const RES_3_A: MicroProgram = MicroProgram::new(
    0x9F,
    "RES 3,A",
    &[MicroAction::ResetBit {
        bit: 3,
        reg: Reg8::A,
    }],
);

// Bit 4
const RES_4_B: MicroProgram = MicroProgram::new(
    0xA0,
    "RES 4,B",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::B,
    }],
);
const RES_4_C: MicroProgram = MicroProgram::new(
    0xA1,
    "RES 4,C",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::C,
    }],
);
const RES_4_D: MicroProgram = MicroProgram::new(
    0xA2,
    "RES 4,D",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::D,
    }],
);
const RES_4_E: MicroProgram = MicroProgram::new(
    0xA3,
    "RES 4,E",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::E,
    }],
);
const RES_4_H: MicroProgram = MicroProgram::new(
    0xA4,
    "RES 4,H",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::H,
    }],
);
const RES_4_L: MicroProgram = MicroProgram::new(
    0xA5,
    "RES 4,L",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::L,
    }],
);
const RES_4_HL: MicroProgram =
    MicroProgram::new(0xA6, "RES 4,(HL)", &[MicroAction::ResetBitHl { bit: 4 }]);
const RES_4_A: MicroProgram = MicroProgram::new(
    0xA7,
    "RES 4,A",
    &[MicroAction::ResetBit {
        bit: 4,
        reg: Reg8::A,
    }],
);

// Bit 5
const RES_5_B: MicroProgram = MicroProgram::new(
    0xA8,
    "RES 5,B",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::B,
    }],
);
const RES_5_C: MicroProgram = MicroProgram::new(
    0xA9,
    "RES 5,C",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::C,
    }],
);
const RES_5_D: MicroProgram = MicroProgram::new(
    0xAA,
    "RES 5,D",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::D,
    }],
);
const RES_5_E: MicroProgram = MicroProgram::new(
    0xAB,
    "RES 5,E",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::E,
    }],
);
const RES_5_H: MicroProgram = MicroProgram::new(
    0xAC,
    "RES 5,H",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::H,
    }],
);
const RES_5_L: MicroProgram = MicroProgram::new(
    0xAD,
    "RES 5,L",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::L,
    }],
);
const RES_5_HL: MicroProgram =
    MicroProgram::new(0xAE, "RES 5,(HL)", &[MicroAction::ResetBitHl { bit: 5 }]);
const RES_5_A: MicroProgram = MicroProgram::new(
    0xAF,
    "RES 5,A",
    &[MicroAction::ResetBit {
        bit: 5,
        reg: Reg8::A,
    }],
);

// Bit 6
const RES_6_B: MicroProgram = MicroProgram::new(
    0xB0,
    "RES 6,B",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::B,
    }],
);
const RES_6_C: MicroProgram = MicroProgram::new(
    0xB1,
    "RES 6,C",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::C,
    }],
);
const RES_6_D: MicroProgram = MicroProgram::new(
    0xB2,
    "RES 6,D",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::D,
    }],
);
const RES_6_E: MicroProgram = MicroProgram::new(
    0xB3,
    "RES 6,E",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::E,
    }],
);
const RES_6_H: MicroProgram = MicroProgram::new(
    0xB4,
    "RES 6,H",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::H,
    }],
);
const RES_6_L: MicroProgram = MicroProgram::new(
    0xB5,
    "RES 6,L",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::L,
    }],
);
const RES_6_HL: MicroProgram =
    MicroProgram::new(0xB6, "RES 6,(HL)", &[MicroAction::ResetBitHl { bit: 6 }]);
const RES_6_A: MicroProgram = MicroProgram::new(
    0xB7,
    "RES 6,A",
    &[MicroAction::ResetBit {
        bit: 6,
        reg: Reg8::A,
    }],
);

// Bit 7
const RES_7_B: MicroProgram = MicroProgram::new(
    0xB8,
    "RES 7,B",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::B,
    }],
);
const RES_7_C: MicroProgram = MicroProgram::new(
    0xB9,
    "RES 7,C",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::C,
    }],
);
const RES_7_D: MicroProgram = MicroProgram::new(
    0xBA,
    "RES 7,D",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::D,
    }],
);
const RES_7_E: MicroProgram = MicroProgram::new(
    0xBB,
    "RES 7,E",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::E,
    }],
);
const RES_7_H: MicroProgram = MicroProgram::new(
    0xBC,
    "RES 7,H",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::H,
    }],
);
const RES_7_L: MicroProgram = MicroProgram::new(
    0xBD,
    "RES 7,L",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::L,
    }],
);
const RES_7_HL: MicroProgram =
    MicroProgram::new(0xBE, "RES 7,(HL)", &[MicroAction::ResetBitHl { bit: 7 }]);
const RES_7_A: MicroProgram = MicroProgram::new(
    0xBF,
    "RES 7,A",
    &[MicroAction::ResetBit {
        bit: 7,
        reg: Reg8::A,
    }],
);

// === SET b,r (seta bit) ===
// Bit 0
const SET_0_B: MicroProgram = MicroProgram::new(
    0xC0,
    "SET 0,B",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::B,
    }],
);
const SET_0_C: MicroProgram = MicroProgram::new(
    0xC1,
    "SET 0,C",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::C,
    }],
);
const SET_0_D: MicroProgram = MicroProgram::new(
    0xC2,
    "SET 0,D",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::D,
    }],
);
const SET_0_E: MicroProgram = MicroProgram::new(
    0xC3,
    "SET 0,E",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::E,
    }],
);
const SET_0_H: MicroProgram = MicroProgram::new(
    0xC4,
    "SET 0,H",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::H,
    }],
);
const SET_0_L: MicroProgram = MicroProgram::new(
    0xC5,
    "SET 0,L",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::L,
    }],
);
const SET_0_HL: MicroProgram =
    MicroProgram::new(0xC6, "SET 0,(HL)", &[MicroAction::SetBitHl { bit: 0 }]);
const SET_0_A: MicroProgram = MicroProgram::new(
    0xC7,
    "SET 0,A",
    &[MicroAction::SetBit {
        bit: 0,
        reg: Reg8::A,
    }],
);

// Bit 1
const SET_1_B: MicroProgram = MicroProgram::new(
    0xC8,
    "SET 1,B",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::B,
    }],
);
const SET_1_C: MicroProgram = MicroProgram::new(
    0xC9,
    "SET 1,C",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::C,
    }],
);
const SET_1_D: MicroProgram = MicroProgram::new(
    0xCA,
    "SET 1,D",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::D,
    }],
);
const SET_1_E: MicroProgram = MicroProgram::new(
    0xCB,
    "SET 1,E",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::E,
    }],
);
const SET_1_H: MicroProgram = MicroProgram::new(
    0xCC,
    "SET 1,H",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::H,
    }],
);
const SET_1_L: MicroProgram = MicroProgram::new(
    0xCD,
    "SET 1,L",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::L,
    }],
);
const SET_1_HL: MicroProgram =
    MicroProgram::new(0xCE, "SET 1,(HL)", &[MicroAction::SetBitHl { bit: 1 }]);
const SET_1_A: MicroProgram = MicroProgram::new(
    0xCF,
    "SET 1,A",
    &[MicroAction::SetBit {
        bit: 1,
        reg: Reg8::A,
    }],
);

// Bit 2
const SET_2_B: MicroProgram = MicroProgram::new(
    0xD0,
    "SET 2,B",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::B,
    }],
);
const SET_2_C: MicroProgram = MicroProgram::new(
    0xD1,
    "SET 2,C",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::C,
    }],
);
const SET_2_D: MicroProgram = MicroProgram::new(
    0xD2,
    "SET 2,D",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::D,
    }],
);
const SET_2_E: MicroProgram = MicroProgram::new(
    0xD3,
    "SET 2,E",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::E,
    }],
);
const SET_2_H: MicroProgram = MicroProgram::new(
    0xD4,
    "SET 2,H",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::H,
    }],
);
const SET_2_L: MicroProgram = MicroProgram::new(
    0xD5,
    "SET 2,L",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::L,
    }],
);
const SET_2_HL: MicroProgram =
    MicroProgram::new(0xD6, "SET 2,(HL)", &[MicroAction::SetBitHl { bit: 2 }]);
const SET_2_A: MicroProgram = MicroProgram::new(
    0xD7,
    "SET 2,A",
    &[MicroAction::SetBit {
        bit: 2,
        reg: Reg8::A,
    }],
);

// Bit 3
const SET_3_B: MicroProgram = MicroProgram::new(
    0xD8,
    "SET 3,B",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::B,
    }],
);
const SET_3_C: MicroProgram = MicroProgram::new(
    0xD9,
    "SET 3,C",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::C,
    }],
);
const SET_3_D: MicroProgram = MicroProgram::new(
    0xDA,
    "SET 3,D",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::D,
    }],
);
const SET_3_E: MicroProgram = MicroProgram::new(
    0xDB,
    "SET 3,E",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::E,
    }],
);
const SET_3_H: MicroProgram = MicroProgram::new(
    0xDC,
    "SET 3,H",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::H,
    }],
);
const SET_3_L: MicroProgram = MicroProgram::new(
    0xDD,
    "SET 3,L",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::L,
    }],
);
const SET_3_HL: MicroProgram =
    MicroProgram::new(0xDE, "SET 3,(HL)", &[MicroAction::SetBitHl { bit: 3 }]);
const SET_3_A: MicroProgram = MicroProgram::new(
    0xDF,
    "SET 3,A",
    &[MicroAction::SetBit {
        bit: 3,
        reg: Reg8::A,
    }],
);

// Bit 4
const SET_4_B: MicroProgram = MicroProgram::new(
    0xE0,
    "SET 4,B",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::B,
    }],
);
const SET_4_C: MicroProgram = MicroProgram::new(
    0xE1,
    "SET 4,C",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::C,
    }],
);
const SET_4_D: MicroProgram = MicroProgram::new(
    0xE2,
    "SET 4,D",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::D,
    }],
);
const SET_4_E: MicroProgram = MicroProgram::new(
    0xE3,
    "SET 4,E",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::E,
    }],
);
const SET_4_H: MicroProgram = MicroProgram::new(
    0xE4,
    "SET 4,H",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::H,
    }],
);
const SET_4_L: MicroProgram = MicroProgram::new(
    0xE5,
    "SET 4,L",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::L,
    }],
);
const SET_4_HL: MicroProgram =
    MicroProgram::new(0xE6, "SET 4,(HL)", &[MicroAction::SetBitHl { bit: 4 }]);
const SET_4_A: MicroProgram = MicroProgram::new(
    0xE7,
    "SET 4,A",
    &[MicroAction::SetBit {
        bit: 4,
        reg: Reg8::A,
    }],
);

// Bit 5
const SET_5_B: MicroProgram = MicroProgram::new(
    0xE8,
    "SET 5,B",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::B,
    }],
);
const SET_5_C: MicroProgram = MicroProgram::new(
    0xE9,
    "SET 5,C",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::C,
    }],
);
const SET_5_D: MicroProgram = MicroProgram::new(
    0xEA,
    "SET 5,D",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::D,
    }],
);
const SET_5_E: MicroProgram = MicroProgram::new(
    0xEB,
    "SET 5,E",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::E,
    }],
);
const SET_5_H: MicroProgram = MicroProgram::new(
    0xEC,
    "SET 5,H",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::H,
    }],
);
const SET_5_L: MicroProgram = MicroProgram::new(
    0xED,
    "SET 5,L",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::L,
    }],
);
const SET_5_HL: MicroProgram =
    MicroProgram::new(0xEE, "SET 5,(HL)", &[MicroAction::SetBitHl { bit: 5 }]);
const SET_5_A: MicroProgram = MicroProgram::new(
    0xEF,
    "SET 5,A",
    &[MicroAction::SetBit {
        bit: 5,
        reg: Reg8::A,
    }],
);

// Bit 6
const SET_6_B: MicroProgram = MicroProgram::new(
    0xF0,
    "SET 6,B",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::B,
    }],
);
const SET_6_C: MicroProgram = MicroProgram::new(
    0xF1,
    "SET 6,C",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::C,
    }],
);
const SET_6_D: MicroProgram = MicroProgram::new(
    0xF2,
    "SET 6,D",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::D,
    }],
);
const SET_6_E: MicroProgram = MicroProgram::new(
    0xF3,
    "SET 6,E",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::E,
    }],
);
const SET_6_H: MicroProgram = MicroProgram::new(
    0xF4,
    "SET 6,H",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::H,
    }],
);
const SET_6_L: MicroProgram = MicroProgram::new(
    0xF5,
    "SET 6,L",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::L,
    }],
);
const SET_6_HL: MicroProgram =
    MicroProgram::new(0xF6, "SET 6,(HL)", &[MicroAction::SetBitHl { bit: 6 }]);
const SET_6_A: MicroProgram = MicroProgram::new(
    0xF7,
    "SET 6,A",
    &[MicroAction::SetBit {
        bit: 6,
        reg: Reg8::A,
    }],
);

// Bit 7
const SET_7_B: MicroProgram = MicroProgram::new(
    0xF8,
    "SET 7,B",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::B,
    }],
);
const SET_7_C: MicroProgram = MicroProgram::new(
    0xF9,
    "SET 7,C",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::C,
    }],
);
const SET_7_D: MicroProgram = MicroProgram::new(
    0xFA,
    "SET 7,D",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::D,
    }],
);
const SET_7_E: MicroProgram = MicroProgram::new(
    0xFB,
    "SET 7,E",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::E,
    }],
);
const SET_7_H: MicroProgram = MicroProgram::new(
    0xFC,
    "SET 7,H",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::H,
    }],
);
const SET_7_L: MicroProgram = MicroProgram::new(
    0xFD,
    "SET 7,L",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::L,
    }],
);
const SET_7_HL: MicroProgram =
    MicroProgram::new(0xFE, "SET 7,(HL)", &[MicroAction::SetBitHl { bit: 7 }]);
const SET_7_A: MicroProgram = MicroProgram::new(
    0xFF,
    "SET 7,A",
    &[MicroAction::SetBit {
        bit: 7,
        reg: Reg8::A,
    }],
);
