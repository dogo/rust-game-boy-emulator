// Microcódigos das instruções de controle da CPU: só o fetch, mais o efeito
// que `CPU::execute_next` aplica (IME, HALT, STOP, travamento)

use super::{CpuControl, MicroAction, MicroProgram};

/// Opcodes inválidos que travam a CPU (Pan Docs)
pub const ILLEGAL_OPCODES: [u8; 11] = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

/// Retorna o microprograma de controle associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        0xF3 => Some(&DI_PROGRAM),
        0xFB => Some(&EI_PROGRAM),
        0x76 => Some(&HALT_PROGRAM),
        0x10 => Some(&STOP_PROGRAM),
        op if ILLEGAL_OPCODES.contains(&op) => Some(&ILLEGAL_PROGRAM),
        _ => None,
    }
}

const DI_PROGRAM: MicroProgram = MicroProgram::new(
    0xF3,
    "DI",
    &[MicroAction::Control(CpuControl::DisableInterrupts)],
);
const EI_PROGRAM: MicroProgram = MicroProgram::new(
    0xFB,
    "EI",
    &[MicroAction::Control(CpuControl::EnableInterrupts)],
);
const HALT_PROGRAM: MicroProgram =
    MicroProgram::new(0x76, "HALT", &[MicroAction::Control(CpuControl::Halt)]);
// O byte de padding é pulado (ou não) pela CPU, conforme o fluxograma de STOP
const STOP_PROGRAM: MicroProgram =
    MicroProgram::new(0x10, "STOP", &[MicroAction::Control(CpuControl::Stop)]);
// O opcode real fica em `MicroProgram::opcode` só como referência
const ILLEGAL_PROGRAM: MicroProgram =
    MicroProgram::new(0xD3, "ILLEGAL", &[MicroAction::Control(CpuControl::Lock)]);
