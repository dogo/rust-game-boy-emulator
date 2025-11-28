// Este módulo implementa microcódigos para instruções da CPU do Game Boy.
// Microcódigos são sequências de micro-operações que simulam o funcionamento interno das instruções.

mod load;
mod logic;
mod jump;
mod arithmetic;

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
    /// Busca um byte imediato assinado do PC e salta relativamente (JR r8)
    JumpRelative,
    /// Busca dois bytes imediatos (little-endian) e salta para o endereço
    FetchImm16AndJump,
    /// Salta para o endereço em HL
    JumpToHl,
    /// Executa RLCA (Rotate Left Circular A)
    ExecuteRLCA,
    /// Executa RRCA (Rotate Right Circular A)
    ExecuteRRCA,
    /// Executa RLA (Rotate Left A through Carry)
    ExecuteRLA,
    /// Executa RRA (Rotate Right A through Carry)
    ExecuteRRA,
    /// Executa CPL (Complement A)
    ExecuteCPL,
    /// Executa SCF (Set Carry Flag)
    ExecuteSCF,
    /// Executa CCF (Complement Carry Flag)
    ExecuteCCF,
    /// Executa ADD A,src (com flags)
    AddAToReg { src: Reg8 },
    /// Executa ADD A,d8
    AddAToImm8,
    /// Executa ADC A,src (com carry)
    AddAWithCarryToReg { src: Reg8 },
    /// Executa ADC A,d8
    AddAWithCarryToImm8,
    /// Executa SUB A,src (com flags)
    SubAFromReg { src: Reg8 },
    /// Executa SUB A,d8
    SubAFromImm8,
    /// Executa SBC A,src (com borrow)
    SubAWithBorrowFromReg { src: Reg8 },
    /// Executa SBC A,d8
    SubAWithBorrowFromImm8,
    /// Executa AND A,src
    AndAToReg { src: Reg8 },
    /// Executa AND A,d8
    AndAToImm8,
    /// Executa OR A,src
    OrAToReg { src: Reg8 },
    /// Executa OR A,d8
    OrAToImm8,
    /// Executa XOR A,src
    XorAToReg { src: Reg8 },
    /// Executa XOR A,d8
    XorAToImm8,
    /// Executa CP A,src (compare, não altera A)
    CompareAToReg { src: Reg8 },
    /// Executa CP A,d8
    CompareAToImm8,
    /// Lê de (HL) e executa ADD A,valor
    AddAToHlValue,
    /// Lê de (HL) e executa ADC A,valor
    AddAWithCarryToHlValue,
    /// Lê de (HL) e executa SUB A,valor
    SubAFromHlValue,
    /// Lê de (HL) e executa SBC A,valor
    SubAWithBorrowFromHlValue,
    /// Lê de (HL) e executa AND A,valor
    AndAToHlValue,
    /// Lê de (HL) e executa OR A,valor
    OrAToHlValue,
    /// Lê de (HL) e executa XOR A,valor
    XorAToHlValue,
    /// Lê de (HL) e executa CP A,valor
    CompareAToHlValue,
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
            MicroAction::JumpRelative => {
                // JR r8: Lê offset assinado e salta relativamente
                // Ciclos: 4 fetch opcode (já feito), 4 ler offset, 4 calcular e saltar
                let offset = bus.cpu_read(regs.get_pc()) as i8;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                // Calcula novo PC e consome 4 ciclos adicionais
                bus.cpu_idle(4);
                // Usa PC já incrementado para calcular o salto
                let new_pc = regs.get_pc().wrapping_add(offset as u16);
                regs.set_pc(new_pc);
            }
            MicroAction::FetchImm16AndJump => {
                // Busca endereço 16-bit e salta (16 ciclos totais: 4 fetch + 4 lo + 4 hi + 4 jump)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16; // 4 ciclos
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16; // 4 ciclos
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                bus.cpu_idle(4); // 4 ciclos para executar o salto
                regs.set_pc(addr);
            }
            MicroAction::JumpToHl => {
                // Salta para o endereço em HL (4 ciclos totais, fetch já foi contado)
                regs.set_pc(regs.get_hl());
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteRLCA => {
                // RLCA: Rotate Left Circular A (4 ciclos totais, fetch já foi contado)
                let a = regs.get_a();
                let carry = (a & 0x80) != 0;
                let res = (a << 1) | (if carry { 1 } else { 0 });
                regs.set_a(res);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(carry);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteRRCA => {
                // RRCA: Rotate Right Circular A (4 ciclos totais, fetch já foi contado)
                let a = regs.get_a();
                let carry = (a & 0x01) != 0;
                let res = (a >> 1) | (if carry { 0x80 } else { 0 });
                regs.set_a(res);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(carry);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteRLA => {
                // RLA: Rotate Left A through Carry (4 ciclos totais, fetch já foi contado)
                let a = regs.get_a();
                let old_c = regs.get_flag_c();
                let carry = (a & 0x80) != 0;
                let res = (a << 1) | (if old_c { 1 } else { 0 });
                regs.set_a(res);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(carry);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteRRA => {
                // RRA: Rotate Right A through Carry (4 ciclos totais, fetch já foi contado)
                let a = regs.get_a();
                let old_c = regs.get_flag_c();
                let carry = (a & 0x01) != 0;
                let res = ((if old_c { 1 } else { 0 }) << 7) | (a >> 1);
                regs.set_a(res);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(carry);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteCPL => {
                // CPL: Complement A (4 ciclos totais, fetch já foi contado)
                regs.set_a(!regs.get_a());
                regs.set_flag_n(true);
                regs.set_flag_h(true);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteSCF => {
                // SCF: Set Carry Flag (4 ciclos totais, fetch já foi contado)
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(true);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::ExecuteCCF => {
                // CCF: Complement Carry Flag (4 ciclos totais, fetch já foi contado)
                let c = regs.get_flag_c();
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(!c);
                // Não adiciona ciclos, o fetch já consumiu os 4 ciclos totais
            }
            MicroAction::AddAToReg { src } => {
                // ADD A,src: Adiciona registrador a A
                let a = regs.get_a();
                let val = src.read(regs);
                let sum = a as u16 + val as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (val & 0x0F)) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::AddAToImm8 => {
                // ADD A,d8: Adiciona imediato a A
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let sum = a as u16 + imm as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (imm & 0x0F)) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::AddAWithCarryToReg { src } => {
                // ADC A,src: Adiciona com carry
                let a = regs.get_a();
                let val = src.read(regs);
                let carry = if regs.get_flag_c() { 1 } else { 0 };
                let sum = a as u16 + val as u16 + carry as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (val & 0x0F) + carry) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::AddAWithCarryToImm8 => {
                // ADC A,d8: Adiciona imediato com carry
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let carry = if regs.get_flag_c() { 1 } else { 0 };
                let sum = a as u16 + imm as u16 + carry as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (imm & 0x0F) + carry) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::SubAFromReg { src } => {
                // SUB A,src: Subtrai registrador de A
                let a = regs.get_a();
                let val = src.read(regs);
                let diff = a as i16 - val as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::SubAFromImm8 => {
                // SUB A,d8: Subtrai imediato de A
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let diff = a as i16 - imm as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (imm & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::SubAWithBorrowFromReg { src } => {
                // SBC A,src: Subtrai com borrow
                let a = regs.get_a();
                let val = src.read(regs);
                let borrow = if regs.get_flag_c() { 1 } else { 0 };
                let diff = a as i16 - val as i16 - borrow as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16 - borrow as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::SubAWithBorrowFromImm8 => {
                // SBC A,d8: Subtrai imediato com borrow
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let borrow = if regs.get_flag_c() { 1 } else { 0 };
                let diff = a as i16 - imm as i16 - borrow as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (imm & 0x0F) as i16 - borrow as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::AndAToReg { src } => {
                // AND A,src: AND lógico
                let a = regs.get_a();
                let val = src.read(regs);
                let res = a & val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(true);
                regs.set_flag_c(false);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::AndAToImm8 => {
                // AND A,d8: AND com imediato
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let res = a & imm;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(true);
                regs.set_flag_c(false);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::OrAToReg { src } => {
                // OR A,src: OR lógico
                let a = regs.get_a();
                let val = src.read(regs);
                let res = a | val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::OrAToImm8 => {
                // OR A,d8: OR com imediato
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let res = a | imm;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::XorAToReg { src } => {
                // XOR A,src: XOR lógico
                let a = regs.get_a();
                let val = src.read(regs);
                let res = a ^ val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::XorAToImm8 => {
                // XOR A,d8: XOR com imediato
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let res = a ^ imm;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::CompareAToReg { src } => {
                // CP A,src: Compara (não altera A)
                let a = regs.get_a();
                let val = src.read(regs);
                let diff = a as i16 - val as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::CompareAToImm8 => {
                // CP A,d8: Compara com imediato
                let pc = regs.get_pc();
                let imm = bus.cpu_read(pc);
                regs.set_pc(pc.wrapping_add(1));
                let a = regs.get_a();
                let diff = a as i16 - imm as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (imm & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais: 4 fetch + 4 ler imm
            }
            MicroAction::AddAToHlValue => {
                // ADD A,(HL): Lê de (HL) e adiciona
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let sum = a as u16 + val as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (val & 0x0F)) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 8 ciclos totais: 4 fetch + 4 ler (HL)
            }
            MicroAction::AddAWithCarryToHlValue => {
                // ADC A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let carry = if regs.get_flag_c() { 1 } else { 0 };
                let sum = a as u16 + val as u16 + carry as u16;
                let res = (sum & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(((a & 0x0F) + (val & 0x0F) + carry) > 0x0F);
                regs.set_flag_c(sum > 0xFF);
                // 8 ciclos totais
            }
            MicroAction::SubAFromHlValue => {
                // SUB A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let diff = a as i16 - val as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais
            }
            MicroAction::SubAWithBorrowFromHlValue => {
                // SBC A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let borrow = if regs.get_flag_c() { 1 } else { 0 };
                let diff = a as i16 - val as i16 - borrow as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16 - borrow as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais
            }
            MicroAction::AndAToHlValue => {
                // AND A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let res = a & val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(true);
                regs.set_flag_c(false);
                // 8 ciclos totais
            }
            MicroAction::OrAToHlValue => {
                // OR A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let res = a | val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 8 ciclos totais
            }
            MicroAction::XorAToHlValue => {
                // XOR A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let res = a ^ val;
                regs.set_a(res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
                // 8 ciclos totais
            }
            MicroAction::CompareAToHlValue => {
                // CP A,(HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let a = regs.get_a();
                let diff = a as i16 - val as i16;
                let res = (diff & 0xFF) as u8;
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h(((a & 0x0F) as i16 - (val & 0x0F) as i16) < 0);
                regs.set_flag_c(diff < 0);
                // 8 ciclos totais
            }
        }
    }
}

/// Retorna o microprograma associado ao opcode, se existir.
/// Orquestra a busca em todos os submódulos de instruções.
pub fn lookup(opcode: u8) -> Option<&'static MicroProgram> {
    // Tenta encontrar em cada categoria de instruções
    load::lookup(opcode)
        .or_else(|| logic::lookup(opcode))
        .or_else(|| jump::lookup(opcode))
        .or_else(|| arithmetic::lookup(opcode))
}
