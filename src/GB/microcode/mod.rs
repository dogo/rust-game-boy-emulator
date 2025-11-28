// Este módulo implementa microcódigos para instruções da CPU do Game Boy.
// Microcódigos são sequências de micro-operações que simulam o funcionamento interno das instruções.

mod load;

use crate::GB::bus::MemoryBus;
use crate::GB::registers::Registers;

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
    pub(crate) fn read(self, regs: &Registers) -> u8 {
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
    pub(crate) fn write(self, regs: &mut Registers, value: u8) {
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
    /// Transfere valor entre dois registradores (dest = src)
    CopyReg { dest: Reg8, src: Reg8 },
    /// Busca um byte imediato do PC e armazena no registrador de destino
    FetchImm8 { dest: Reg8 },
    /// Busca um byte imediato do PC e escreve na memória em HL
    FetchImm8ToHl,
    /// Lê da memória no endereço especificado (BC, DE, ou endereço direto) e armazena em A
    ReadFromAddr { addr_src: AddrSrc, dest: Reg8 },
    /// Escreve A na memória no endereço especificado (BC, DE, ou endereço direto)
    WriteAToAddr { addr_src: AddrSrc },
}

/// Fonte de endereço para operações de memória
#[derive(Clone, Copy)]
pub enum AddrSrc {
    BC,
    DE,
    Hl,
}

/// Estrutura que representa um microprograma, ou seja, uma sequência de micro-operações para uma instrução.
pub struct MicroProgram {
    pub opcode: u8,                    // Código da instrução
    pub name: &'static str,            // Nome da instrução
    pub steps: &'static [MicroAction], // Passos do microcódigo
}

impl MicroProgram {
    /// Cria um novo microprograma.
    pub const fn new(opcode: u8, name: &'static str, steps: &'static [MicroAction]) -> Self {
        Self {
            opcode,
            name,
            steps,
        }
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
            MicroAction::CopyReg { dest, src } => {
                // Transfere valor entre registradores (sem acesso à memória)
                let value = src.read(regs);
                dest.write(regs, value);
                // Transferência entre registradores não acessa memória, apenas espera ciclos
                bus.cpu_idle(4);
            }
            MicroAction::FetchImm8 { dest } => {
                // Busca byte imediato do PC e armazena no registrador
                let pc = regs.get_pc();
                let value = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                dest.write(regs, value);
            }
            MicroAction::FetchImm8ToHl => {
                // Busca byte imediato do PC e escreve em HL
                let pc = regs.get_pc();
                let value = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let addr = regs.get_hl();
                bus.cpu_write(addr, value);
            }
            MicroAction::ReadFromAddr { addr_src, dest } => {
                // Lê da memória no endereço especificado (BC, DE, ou HL)
                let addr = match addr_src {
                    AddrSrc::BC => regs.get_bc(),
                    AddrSrc::DE => regs.get_de(),
                    AddrSrc::Hl => regs.get_hl(),
                };
                let value = bus.cpu_read(addr);
                dest.write(regs, value);
            }
            MicroAction::WriteAToAddr { addr_src } => {
                // Escreve A na memória no endereço especificado
                let addr = match addr_src {
                    AddrSrc::BC => regs.get_bc(),
                    AddrSrc::DE => regs.get_de(),
                    AddrSrc::Hl => regs.get_hl(),
                };
                let value = regs.get_a();
                bus.cpu_write(addr, value);
            }
        }
    }
}

/// Retorna o microprograma associado ao opcode, se existir.
/// Orquestra a busca em todos os submódulos de instruções.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    // Tenta encontrar na categoria de instruções de carga (LOAD)
    load::lookup(opcode)
}
