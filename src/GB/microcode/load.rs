// Microcódigos para instruções de LOAD (carregamento/armazenamento)

use super::{AddrSrc, MicroAction, MicroProgram, Reg8};

// === NOP ===
const NOP_PROGRAM: MicroProgram = MicroProgram::new(0x00, "NOP", &[]);

// === LD r,d8 - Carrega valor imediato 8-bit em registrador ===
const LD_B_D8: MicroProgram = MicroProgram::new(0x06, "LD B,d8", &[MicroAction::FetchImm8 { dest: Reg8::B }]);
const LD_C_D8: MicroProgram = MicroProgram::new(0x0E, "LD C,d8", &[MicroAction::FetchImm8 { dest: Reg8::C }]);
const LD_D_D8: MicroProgram = MicroProgram::new(0x16, "LD D,d8", &[MicroAction::FetchImm8 { dest: Reg8::D }]);
const LD_E_D8: MicroProgram = MicroProgram::new(0x1E, "LD E,d8", &[MicroAction::FetchImm8 { dest: Reg8::E }]);
const LD_H_D8: MicroProgram = MicroProgram::new(0x26, "LD H,d8", &[MicroAction::FetchImm8 { dest: Reg8::H }]);
const LD_L_D8: MicroProgram = MicroProgram::new(0x2E, "LD L,d8", &[MicroAction::FetchImm8 { dest: Reg8::L }]);
const LD_A_D8: MicroProgram = MicroProgram::new(0x3E, "LD A,d8", &[MicroAction::FetchImm8 { dest: Reg8::A }]);
const LD_HL_D8: MicroProgram = MicroProgram::new(0x36, "LD (HL),d8", &[MicroAction::FetchImm8ToHl]);

// === LD A,(BC/DE) e LD (BC/DE),A ===
const LD_A_BC: MicroProgram = MicroProgram::new(0x0A, "LD A,(BC)", &[MicroAction::ReadFromAddr { addr_src: AddrSrc::BC, dest: Reg8::A }]);
const LD_A_DE: MicroProgram = MicroProgram::new(0x1A, "LD A,(DE)", &[MicroAction::ReadFromAddr { addr_src: AddrSrc::DE, dest: Reg8::A }]);
const LD_BC_A: MicroProgram = MicroProgram::new(0x02, "LD (BC),A", &[MicroAction::WriteAToAddr { addr_src: AddrSrc::BC }]);
const LD_DE_A: MicroProgram = MicroProgram::new(0x12, "LD (DE),A", &[MicroAction::WriteAToAddr { addr_src: AddrSrc::DE }]);

// === LD r,(HL) - Carrega da memória (HL) em registrador ===
const LD_B_HL: MicroProgram = MicroProgram::new(0x46, "LD B,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::B }]);
const LD_C_HL: MicroProgram = MicroProgram::new(0x4E, "LD C,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::C }]);
const LD_D_HL: MicroProgram = MicroProgram::new(0x56, "LD D,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::D }]);
const LD_E_HL: MicroProgram = MicroProgram::new(0x5E, "LD E,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::E }]);
const LD_H_HL: MicroProgram = MicroProgram::new(0x66, "LD H,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::H }]);
const LD_L_HL: MicroProgram = MicroProgram::new(0x6E, "LD L,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::L }]);
const LD_A_HL: MicroProgram = MicroProgram::new(0x7E, "LD A,(HL)", &[MicroAction::ReadFromHl { dest: Reg8::A }]);

// === LD (HL),r - Armazena registrador na memória (HL) ===
const LD_HL_B: MicroProgram = MicroProgram::new(0x70, "LD (HL),B", &[MicroAction::WriteToHl { src: Reg8::B }]);
const LD_HL_C: MicroProgram = MicroProgram::new(0x71, "LD (HL),C", &[MicroAction::WriteToHl { src: Reg8::C }]);
const LD_HL_D: MicroProgram = MicroProgram::new(0x72, "LD (HL),D", &[MicroAction::WriteToHl { src: Reg8::D }]);
const LD_HL_E: MicroProgram = MicroProgram::new(0x73, "LD (HL),E", &[MicroAction::WriteToHl { src: Reg8::E }]);
const LD_HL_H: MicroProgram = MicroProgram::new(0x74, "LD (HL),H", &[MicroAction::WriteToHl { src: Reg8::H }]);
const LD_HL_L: MicroProgram = MicroProgram::new(0x75, "LD (HL),L", &[MicroAction::WriteToHl { src: Reg8::L }]);
const LD_HL_A: MicroProgram = MicroProgram::new(0x77, "LD (HL),A", &[MicroAction::WriteToHl { src: Reg8::A }]);

// === LD r,r - Transferências entre registradores ===
// Padrão: dest = (opcode >> 3) & 0x07, src = opcode & 0x07
// 0=B, 1=C, 2=D, 3=E, 4=H, 5=L, 7=A (6=HL é tratado separadamente)
const LD_B_B: MicroProgram = MicroProgram::new(0x40, "LD B,B", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::B }]);
const LD_B_C: MicroProgram = MicroProgram::new(0x41, "LD B,C", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::C }]);
const LD_B_D: MicroProgram = MicroProgram::new(0x42, "LD B,D", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::D }]);
const LD_B_E: MicroProgram = MicroProgram::new(0x43, "LD B,E", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::E }]);
const LD_B_H: MicroProgram = MicroProgram::new(0x44, "LD B,H", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::H }]);
const LD_B_L: MicroProgram = MicroProgram::new(0x45, "LD B,L", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::L }]);
const LD_B_A: MicroProgram = MicroProgram::new(0x47, "LD B,A", &[MicroAction::CopyReg { dest: Reg8::B, src: Reg8::A }]);

const LD_C_B: MicroProgram = MicroProgram::new(0x48, "LD C,B", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::B }]);
const LD_C_C: MicroProgram = MicroProgram::new(0x49, "LD C,C", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::C }]);
const LD_C_D: MicroProgram = MicroProgram::new(0x4A, "LD C,D", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::D }]);
const LD_C_E: MicroProgram = MicroProgram::new(0x4B, "LD C,E", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::E }]);
const LD_C_H: MicroProgram = MicroProgram::new(0x4C, "LD C,H", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::H }]);
const LD_C_L: MicroProgram = MicroProgram::new(0x4D, "LD C,L", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::L }]);
const LD_C_A: MicroProgram = MicroProgram::new(0x4F, "LD C,A", &[MicroAction::CopyReg { dest: Reg8::C, src: Reg8::A }]);

const LD_D_B: MicroProgram = MicroProgram::new(0x50, "LD D,B", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::B }]);
const LD_D_C: MicroProgram = MicroProgram::new(0x51, "LD D,C", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::C }]);
const LD_D_D: MicroProgram = MicroProgram::new(0x52, "LD D,D", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::D }]);
const LD_D_E: MicroProgram = MicroProgram::new(0x53, "LD D,E", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::E }]);
const LD_D_H: MicroProgram = MicroProgram::new(0x54, "LD D,H", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::H }]);
const LD_D_L: MicroProgram = MicroProgram::new(0x55, "LD D,L", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::L }]);
const LD_D_A: MicroProgram = MicroProgram::new(0x57, "LD D,A", &[MicroAction::CopyReg { dest: Reg8::D, src: Reg8::A }]);

const LD_E_B: MicroProgram = MicroProgram::new(0x58, "LD E,B", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::B }]);
const LD_E_C: MicroProgram = MicroProgram::new(0x59, "LD E,C", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::C }]);
const LD_E_D: MicroProgram = MicroProgram::new(0x5A, "LD E,D", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::D }]);
const LD_E_E: MicroProgram = MicroProgram::new(0x5B, "LD E,E", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::E }]);
const LD_E_H: MicroProgram = MicroProgram::new(0x5C, "LD E,H", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::H }]);
const LD_E_L: MicroProgram = MicroProgram::new(0x5D, "LD E,L", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::L }]);
const LD_E_A: MicroProgram = MicroProgram::new(0x5F, "LD E,A", &[MicroAction::CopyReg { dest: Reg8::E, src: Reg8::A }]);

const LD_H_B: MicroProgram = MicroProgram::new(0x60, "LD H,B", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::B }]);
const LD_H_C: MicroProgram = MicroProgram::new(0x61, "LD H,C", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::C }]);
const LD_H_D: MicroProgram = MicroProgram::new(0x62, "LD H,D", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::D }]);
const LD_H_E: MicroProgram = MicroProgram::new(0x63, "LD H,E", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::E }]);
const LD_H_H: MicroProgram = MicroProgram::new(0x64, "LD H,H", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::H }]);
const LD_H_L: MicroProgram = MicroProgram::new(0x65, "LD H,L", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::L }]);
const LD_H_A: MicroProgram = MicroProgram::new(0x67, "LD H,A", &[MicroAction::CopyReg { dest: Reg8::H, src: Reg8::A }]);

const LD_L_B: MicroProgram = MicroProgram::new(0x68, "LD L,B", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::B }]);
const LD_L_C: MicroProgram = MicroProgram::new(0x69, "LD L,C", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::C }]);
const LD_L_D: MicroProgram = MicroProgram::new(0x6A, "LD L,D", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::D }]);
const LD_L_E: MicroProgram = MicroProgram::new(0x6B, "LD L,E", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::E }]);
const LD_L_H: MicroProgram = MicroProgram::new(0x6C, "LD L,H", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::H }]);
const LD_L_L: MicroProgram = MicroProgram::new(0x6D, "LD L,L", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::L }]);
const LD_L_A: MicroProgram = MicroProgram::new(0x6F, "LD L,A", &[MicroAction::CopyReg { dest: Reg8::L, src: Reg8::A }]);

const LD_A_B: MicroProgram = MicroProgram::new(0x78, "LD A,B", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::B }]);
const LD_A_C: MicroProgram = MicroProgram::new(0x79, "LD A,C", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::C }]);
const LD_A_D: MicroProgram = MicroProgram::new(0x7A, "LD A,D", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::D }]);
const LD_A_E: MicroProgram = MicroProgram::new(0x7B, "LD A,E", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::E }]);
const LD_A_H: MicroProgram = MicroProgram::new(0x7C, "LD A,H", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::H }]);
const LD_A_L: MicroProgram = MicroProgram::new(0x7D, "LD A,L", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::L }]);
const LD_A_A: MicroProgram = MicroProgram::new(0x7F, "LD A,A", &[MicroAction::CopyReg { dest: Reg8::A, src: Reg8::A }]);

/// Retorna o microprograma de LOAD associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        // NOP
        0x00 => Some(&NOP_PROGRAM),

        // LD r,d8
        0x06 => Some(&LD_B_D8),
        0x0E => Some(&LD_C_D8),
        0x16 => Some(&LD_D_D8),
        0x1E => Some(&LD_E_D8),
        0x26 => Some(&LD_H_D8),
        0x2E => Some(&LD_L_D8),
        0x3E => Some(&LD_A_D8),
        0x36 => Some(&LD_HL_D8),

        // LD A,(BC/DE) e LD (BC/DE),A
        0x0A => Some(&LD_A_BC),
        0x1A => Some(&LD_A_DE),
        0x02 => Some(&LD_BC_A),
        0x12 => Some(&LD_DE_A),

        // LD r,(HL)
        0x46 => Some(&LD_B_HL),
        0x4E => Some(&LD_C_HL),
        0x56 => Some(&LD_D_HL),
        0x5E => Some(&LD_E_HL),
        0x66 => Some(&LD_H_HL),
        0x6E => Some(&LD_L_HL),
        0x7E => Some(&LD_A_HL),

        // LD (HL),r
        0x70 => Some(&LD_HL_B),
        0x71 => Some(&LD_HL_C),
        0x72 => Some(&LD_HL_D),
        0x73 => Some(&LD_HL_E),
        0x74 => Some(&LD_HL_H),
        0x75 => Some(&LD_HL_L),
        0x77 => Some(&LD_HL_A),

        // LD r,r (0x40-0x7F exceto 0x76 que é HALT)
        0x40 => Some(&LD_B_B), 0x41 => Some(&LD_B_C), 0x42 => Some(&LD_B_D), 0x43 => Some(&LD_B_E),
        0x44 => Some(&LD_B_H), 0x45 => Some(&LD_B_L), 0x47 => Some(&LD_B_A),
        0x48 => Some(&LD_C_B), 0x49 => Some(&LD_C_C), 0x4A => Some(&LD_C_D), 0x4B => Some(&LD_C_E),
        0x4C => Some(&LD_C_H), 0x4D => Some(&LD_C_L), 0x4F => Some(&LD_C_A),
        0x50 => Some(&LD_D_B), 0x51 => Some(&LD_D_C), 0x52 => Some(&LD_D_D), 0x53 => Some(&LD_D_E),
        0x54 => Some(&LD_D_H), 0x55 => Some(&LD_D_L), 0x57 => Some(&LD_D_A),
        0x58 => Some(&LD_E_B), 0x59 => Some(&LD_E_C), 0x5A => Some(&LD_E_D), 0x5B => Some(&LD_E_E),
        0x5C => Some(&LD_E_H), 0x5D => Some(&LD_E_L), 0x5F => Some(&LD_E_A),
        0x60 => Some(&LD_H_B), 0x61 => Some(&LD_H_C), 0x62 => Some(&LD_H_D), 0x63 => Some(&LD_H_E),
        0x64 => Some(&LD_H_H), 0x65 => Some(&LD_H_L), 0x67 => Some(&LD_H_A),
        0x68 => Some(&LD_L_B), 0x69 => Some(&LD_L_C), 0x6A => Some(&LD_L_D), 0x6B => Some(&LD_L_E),
        0x6C => Some(&LD_L_H), 0x6D => Some(&LD_L_L), 0x6F => Some(&LD_L_A),
        0x78 => Some(&LD_A_B), 0x79 => Some(&LD_A_C), 0x7A => Some(&LD_A_D), 0x7B => Some(&LD_A_E),
        0x7C => Some(&LD_A_H), 0x7D => Some(&LD_A_L), 0x7F => Some(&LD_A_A),

        _ => None,
    }
}
