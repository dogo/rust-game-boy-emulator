use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

// Este módulo implementa microcódigos para instruções da CPU do Game Boy.
// Microcódigos são sequências de micro-operações que simulam o funcionamento interno das instruções.

/// Representa um registrador de 8 bits da CPU para operações de leitura/escrita no microcódigo.
#[derive(Clone, Copy)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Reg8 {
    // Lê o valor do registrador especificado.
    fn read(self, regs: &Registers) -> u8 {
        match self {
            Reg8::A => regs.get_a(),
            Reg8::B => regs.get_b(),
            Reg8::C => regs.get_c(),
            Reg8::D => regs.get_d(),
            Reg8::E => regs.get_e(),
            Reg8::H => regs.get_h(),
            Reg8::L => regs.get_l(),
        }
    }

    // Escreve um valor no registrador especificado.
    fn write(self, regs: &mut Registers, value: u8) {
        match self {
            Reg8::A => regs.set_a(value),
            Reg8::B => regs.set_b(value),
            Reg8::C => regs.set_c(value),
            Reg8::D => regs.set_d(value),
            Reg8::E => regs.set_e(value),
            Reg8::H => regs.set_h(value),
            Reg8::L => regs.set_l(value),
        }
    }
}

/// Micro-operações individuais executadas sequencialmente para modelar uma instrução.
#[derive(Clone, Copy)]
pub enum MicroAction {
    /// Espera pelo número de ciclos de máquina especificado (1 ciclo de máquina = 4 ciclos de CPU)
    Wait(u8),
    /// Lê da memória no endereço contido em HL e armazena no registrador de destino.
    ReadFromHl { dest: Reg8 },
    /// Escreve o valor do registrador de origem na memória no endereço contido em HL.
    WriteToHl { src: Reg8 },
}

/// Estrutura que representa um microprograma, ou seja, uma sequência de micro-operações para uma instrução.
pub struct MicroProgram {
    pub opcode: u8, // Código da instrução
    pub name: &'static str, // Nome da instrução
    pub steps: &'static [MicroAction], // Passos do microcódigo
}

impl MicroProgram {
    /// Cria um novo microprograma.
    pub const fn new(opcode: u8, name: &'static str, steps: &'static [MicroAction]) -> Self {
        Self { opcode, name, steps }
    }
}

/// Executa um microprograma, consumindo ciclos da CPU diretamente através do barramento de memória.
pub fn execute(program: &MicroProgram, regs: &mut Registers, bus: &mut MemoryBus) {
    for step in program.steps {
        match *step {
            MicroAction::Wait(m_cycles) => {
                // Espera o número de ciclos de máquina especificado
                if m_cycles > 0 {
                    bus.cpu_idle((m_cycles as u32) * 4);
                }
            }
            MicroAction::ReadFromHl { dest } => {
                // Lê da memória no endereço HL e armazena no registrador de destino
                let addr = regs.get_hl();
                let value = bus.cpu_read(addr);
                dest.write(regs, value);
            }
            MicroAction::WriteToHl { src } => {
                // Escreve o valor do registrador de origem na memória no endereço HL
                let addr = regs.get_hl();
                let value = src.read(regs);
                bus.cpu_write(addr, value);
            }
        }
    }
}

// === Definições de microcódigos ===
// Microprograma para a instrução NOP (No Operation)
const NOP_PROGRAM: MicroProgram = MicroProgram::new(0x00, "NOP", &[]);
// Microprograma para carregar o valor da memória (HL) em A
const LD_A_FROM_HL: MicroProgram = MicroProgram::new(
    0x7E,
    "LD A,(HL)",
    &[MicroAction::ReadFromHl { dest: Reg8::A }],
);
// Microprograma para armazenar o valor de A na memória (HL)
const LD_HL_FROM_A: MicroProgram = MicroProgram::new(
    0x77,
    "LD (HL),A",
    &[MicroAction::WriteToHl { src: Reg8::A }],
);

/// Retorna o microprograma associado ao opcode, se existir.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    match opcode {
        0x00 => Some(&NOP_PROGRAM),
        0x7E => Some(&LD_A_FROM_HL),
        0x77 => Some(&LD_HL_FROM_A),
        _ => None,
    }
}
