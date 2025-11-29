// Este módulo implementa microcódigos para instruções da CPU do Game Boy.
// Microcódigos são sequências de micro-operações que simulam o funcionamento interno das instruções.

mod arithmetic;
pub mod cb_prefix;
mod jump;
mod load;
mod logic;
mod stack;

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
    /// Busca dois bytes imediatos (little-endian) e carrega em registrador 16-bit
    FetchImm16ToReg16 { idx: u8 },
    /// Busca dois bytes imediatos, lê da memória e armazena em A
    FetchImm16AndReadToA,
    /// Busca dois bytes imediatos e escreve A na memória
    FetchImm16AndWriteA,
    /// Busca dois bytes imediatos e escreve SP na memória (little-endian)
    FetchImm16AndWriteSP,
    /// Carrega HL com SP (LD SP,HL)
    LoadSpFromHl,
    /// Carrega HL com SP + byte assinado (LD HL,SP+r8)
    LoadHlFromSpPlusSignedImm8,
    /// LDH: Escreve A em 0xFF00 + offset
    WriteAToFF00PlusImm8,
    /// LDH: Lê de 0xFF00 + offset para A
    ReadFromFF00PlusImm8ToA,
    /// LDH: Escreve A em 0xFF00 + C
    WriteAToFF00PlusC,
    /// LDH: Lê de 0xFF00 + C para A
    ReadFromFF00PlusCToA,
    /// LDI: Escreve A em (HL) e incrementa HL
    WriteAToHlAndIncrement,
    /// LDI: Lê de (HL) para A e incrementa HL
    ReadFromHlToAAndIncrement,
    /// LDD: Escreve A em (HL) e decrementa HL
    WriteAToHlAndDecrement,
    /// LDD: Lê de (HL) para A e decrementa HL
    ReadFromHlToAAndDecrement,
    /// Busca um byte imediato assinado do PC e salta relativamente (JR r8)
    JumpRelative,
    /// Busca dois bytes imediatos (little-endian) e salta para o endereço
    FetchImm16AndJump,
    /// Salta para o endereço em HL
    JumpToHl,
    /// Busca byte assinado e salta relativamente se condição for verdadeira
    JumpRelativeConditional { cond: JumpCondition },
    /// Busca endereço 16-bit e salta se condição for verdadeira
    JumpAbsoluteConditional { cond: JumpCondition },
    /// Empilha registrador 16-bit (BC, DE, HL, AF)
    PushReg16 { idx: u8 },
    /// Desempilha para registrador 16-bit
    PopReg16 { idx: u8 },
    /// Empilha PC e salta para endereço 16-bit (CALL)
    CallAbsolute,
    /// Empilha PC e salta para endereço 16-bit se condição verdadeira (CALL cc)
    CallAbsoluteConditional { cond: JumpCondition },
    /// Desempilha PC (RET)
    Return,
    /// Desempilha PC se condição verdadeira (RET cc)
    ReturnConditional { cond: JumpCondition },
    /// Empilha PC e salta para endereço RST (0x00, 0x08, 0x10, etc)
    Reset { addr: u16 },
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
    /// Incrementa registrador 8-bit
    IncReg { reg: Reg8 },
    /// Decrementa registrador 8-bit
    DecReg { reg: Reg8 },
    /// Incrementa valor em (HL) - read-modify-write
    IncHlValue,
    /// Decrementa valor em (HL) - read-modify-write
    DecHlValue,
    /// Executa DAA (Decimal Adjust Accumulator)
    ExecuteDAA,
    /// Incrementa registrador de 16 bits (BC, DE, HL, SP)
    IncReg16 { idx: u8 },
    /// Decrementa registrador de 16 bits (BC, DE, HL, SP)
    DecReg16 { idx: u8 },
    /// Adiciona registrador 16-bit a HL
    AddHlToReg16 { idx: u8 },
    /// Adiciona byte assinado a SP
    AddSpToSignedImm8,
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
    // === CB-prefix operations ===
    // Nota: CB prefix é tratado de forma especial no CPU.rs antes de chamar execute()
    /// CB: RLC (Rotate Left Circular) em registrador
    ExecuteRLC { reg: Reg8 },
    /// CB: RLC (HL)
    ExecuteRLCHl,
    /// CB: RRC (Rotate Right Circular) em registrador
    ExecuteRRC { reg: Reg8 },
    /// CB: RRC (HL)
    ExecuteRRCHl,
    /// CB: RL (Rotate Left through Carry) em registrador
    ExecuteRL { reg: Reg8 },
    /// CB: RL (HL)
    ExecuteRLHl,
    /// CB: RR (Rotate Right through Carry) em registrador
    ExecuteRR { reg: Reg8 },
    /// CB: RR (HL)
    ExecuteRRHl,
    /// CB: SLA (Shift Left Arithmetic) em registrador
    ExecuteSLA { reg: Reg8 },
    /// CB: SLA (HL)
    ExecuteSLAHl,
    /// CB: SRA (Shift Right Arithmetic) em registrador
    ExecuteSRA { reg: Reg8 },
    /// CB: SRA (HL)
    ExecuteSRAHl,
    /// CB: SWAP em registrador
    ExecuteSWAP { reg: Reg8 },
    /// CB: SWAP (HL)
    ExecuteSWAPHl,
    /// CB: SRL (Shift Right Logical) em registrador
    ExecuteSRL { reg: Reg8 },
    /// CB: SRL (HL)
    ExecuteSRLHl,
    /// CB: BIT b,r (testa bit)
    TestBit { bit: u8, reg: Reg8 },
    /// CB: BIT b,(HL) (testa bit)
    TestBitHl { bit: u8 },
    /// CB: RES b,r (reseta bit)
    ResetBit { bit: u8, reg: Reg8 },
    /// CB: RES b,(HL) (reseta bit)
    ResetBitHl { bit: u8 },
    /// CB: SET b,r (seta bit)
    SetBit { bit: u8, reg: Reg8 },
    /// CB: SET b,(HL) (seta bit)
    SetBitHl { bit: u8 },
}

/// Fonte de endereço para operações de memória
#[derive(Clone, Copy)]
pub enum AddrSrc {
    BC,
    DE,
    Hl,
}

/// Condições para jumps condicionais
#[derive(Clone, Copy)]
pub enum JumpCondition {
    NZ, // Não zero (!Z)
    Z,  // Zero
    NC, // Não carry (!C)
    C,  // Carry
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
            MicroAction::JumpRelativeConditional { cond } => {
                // JR cc,r8: Salta relativamente se condição verdadeira
                // 8 ciclos se não saltar, 12 ciclos se saltar
                let offset = bus.cpu_read(regs.get_pc()) as i8;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let cond_true = match cond {
                    JumpCondition::NZ => !regs.get_flag_z(),
                    JumpCondition::Z => regs.get_flag_z(),
                    JumpCondition::NC => !regs.get_flag_c(),
                    JumpCondition::C => regs.get_flag_c(),
                };
                if cond_true {
                    bus.cpu_idle(4); // 4 ciclos adicionais para calcular e saltar
                    let new_pc = regs.get_pc().wrapping_add(offset as u16);
                    regs.set_pc(new_pc);
                }
                // Se condição falsa, apenas 8 ciclos totais (4 fetch + 4 ler offset)
            }
            MicroAction::JumpAbsoluteConditional { cond } => {
                // JP cc,a16: Salta absolutamente se condição verdadeira
                // 12 ciclos se não saltar, 16 ciclos se saltar
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                let cond_true = match cond {
                    JumpCondition::NZ => !regs.get_flag_z(),
                    JumpCondition::Z => regs.get_flag_z(),
                    JumpCondition::NC => !regs.get_flag_c(),
                    JumpCondition::C => regs.get_flag_c(),
                };
                if cond_true {
                    bus.cpu_idle(4); // 4 ciclos adicionais para saltar
                    regs.set_pc(addr);
                }
                // Se condição falsa, 12 ciclos totais (4 fetch + 4 lo + 4 hi)
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
            MicroAction::IncReg { reg } => {
                // INC reg: Incrementa registrador
                let val = reg.read(regs);
                let res = val.wrapping_add(1);
                reg.write(regs, res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h((val & 0x0F) + 1 > 0x0F);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::DecReg { reg } => {
                // DEC reg: Decrementa registrador
                let val = reg.read(regs);
                let res = val.wrapping_sub(1);
                reg.write(regs, res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h((val & 0x0F) == 0);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::IncHlValue => {
                // INC (HL): Read-modify-write (12 ciclos: 4 fetch + 4 read + 4 write)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let res = val.wrapping_add(1);
                bus.cpu_write(addr, res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(false);
                regs.set_flag_h((val & 0x0F) + 1 > 0x0F);
                // Total: 12 ciclos (4 fetch já feito + 4 read + 4 write)
            }
            MicroAction::DecHlValue => {
                // DEC (HL): Read-modify-write (12 ciclos)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let res = val.wrapping_sub(1);
                bus.cpu_write(addr, res);
                regs.set_flag_z(res == 0);
                regs.set_flag_n(true);
                regs.set_flag_h((val & 0x0F) == 0);
                // Total: 12 ciclos
            }
            MicroAction::ExecuteDAA => {
                // DAA: Decimal Adjust Accumulator
                let mut a = regs.get_a();
                let n = regs.get_flag_n();
                let mut c = regs.get_flag_c();
                let h = regs.get_flag_h();

                let mut adjust: u8 = 0;
                if !n {
                    if c || a > 0x99 {
                        adjust |= 0x60;
                        c = true;
                    }
                    if h || (a & 0x0F) > 0x09 {
                        adjust |= 0x06;
                    }
                    a = a.wrapping_add(adjust);
                } else {
                    if c {
                        adjust |= 0x60;
                    }
                    if h {
                        adjust |= 0x06;
                    }
                    a = a.wrapping_sub(adjust);
                }

                regs.set_a(a);
                regs.set_flag_z(a == 0);
                // N permanece como está
                regs.set_flag_h(false);
                regs.set_flag_c(c);
                // 4 ciclos totais, fetch já foi contado
            }
            MicroAction::IncReg16 { idx } => {
                // INC rr: Incrementa registrador 16-bit (BC, DE, HL, SP)
                // 8 ciclos totais: 4 fetch + 4 operação
                // idx: 0=BC, 1=DE, 2=HL, 3=SP
                let val = match idx {
                    0 => regs.get_bc(),
                    1 => regs.get_de(),
                    2 => regs.get_hl(),
                    3 => regs.get_sp(),
                    _ => 0,
                };
                // OAM Bug acontece no início do M-cycle 2 (T4)
                // O valor é colocado no barramento de endereços imediatamente
                bus.oam_bug_inc_dec(val);
                bus.cpu_idle(4);
                let res = val.wrapping_add(1);
                match idx {
                    0 => regs.set_bc(res),
                    1 => regs.set_de(res),
                    2 => regs.set_hl(res),
                    3 => regs.set_sp(res),
                    _ => {}
                }
            }
            MicroAction::DecReg16 { idx } => {
                // DEC rr: Decrementa registrador 16-bit
                // 8 ciclos totais: 4 fetch + 4 operação
                let val = match idx {
                    0 => regs.get_bc(),
                    1 => regs.get_de(),
                    2 => regs.get_hl(),
                    3 => regs.get_sp(),
                    _ => 0,
                };
                // OAM Bug acontece no início do M-cycle 2 (T4)
                bus.oam_bug_inc_dec(val);
                bus.cpu_idle(4);
                let res = val.wrapping_sub(1);
                match idx {
                    0 => regs.set_bc(res),
                    1 => regs.set_de(res),
                    2 => regs.set_hl(res),
                    3 => regs.set_sp(res),
                    _ => {}
                }
            }
            MicroAction::AddHlToReg16 { idx } => {
                // ADD HL,rr: Adiciona registrador 16-bit a HL
                let hl = regs.get_hl();
                let rr = match idx {
                    0 => regs.get_bc(),
                    1 => regs.get_de(),
                    2 => regs.get_hl(),
                    3 => regs.get_sp(),
                    _ => 0,
                };
                let res = hl.wrapping_add(rr);
                regs.set_hl(res);
                regs.set_flag_n(false);
                regs.set_flag_h(((hl & 0x0FFF) + (rr & 0x0FFF)) > 0x0FFF);
                regs.set_flag_c((hl as u32 + rr as u32) > 0xFFFF);
                bus.cpu_idle(4); // 8 ciclos totais
            }
            MicroAction::AddSpToSignedImm8 => {
                // ADD SP,r8: Adiciona byte assinado a SP
                let pc = regs.get_pc();
                let offset = bus.cpu_read(pc) as i8;
                regs.set_pc(pc.wrapping_add(1));
                let sp = regs.get_sp();
                let res = sp.wrapping_add(offset as u16);
                regs.set_sp(res);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                // Half-carry e carry são calculados nos 4 bits baixos
                regs.set_flag_h(((sp & 0x0F) as u16 + (offset as u8 & 0x0F) as u16) > 0x0F);
                regs.set_flag_c(((sp & 0xFF) as u16 + (offset as u8 as u16)) > 0xFF);
                bus.cpu_idle(4); // 16 ciclos totais: 4 fetch + 4 ler imm + 4 calcular + 4 idle
            }
            MicroAction::PushReg16 { idx } => {
                // PUSH rr: Empilha registrador 16-bit (16 ciclos)
                // OAM Bug: 4 vezes (efetivamente 3) - dois writes + dois glitched writes do dec SP
                // idx: 0=BC, 1=DE, 2=HL, 3=AF
                let val = match idx {
                    0 => regs.get_bc(),
                    1 => regs.get_de(),
                    2 => regs.get_hl(),
                    3 => regs.get_af(),
                    _ => 0,
                };
                let mut sp = regs.get_sp();
                // Primeiro decremento de SP (glitched write)
                bus.cpu_idle(2);
                bus.oam_bug_inc_dec(sp);
                sp = sp.wrapping_sub(1);
                bus.cpu_idle(2);
                // Write byte alto (write normal)
                bus.cpu_write(sp, (val >> 8) as u8);
                // Segundo decremento de SP (glitched write)
                bus.oam_bug_inc_dec(sp);
                sp = sp.wrapping_sub(1);
                // Write byte baixo (write normal)
                bus.cpu_write(sp, (val & 0xFF) as u8);
                regs.set_sp(sp);
            }
            MicroAction::PopReg16 { idx } => {
                // POP rr: Desempilha para registrador 16-bit (12 ciclos)
                // OAM Bug: 3 vezes - read, glitched write do inc SP, read, glitched write
                // idx: 0=BC, 1=DE, 2=HL, 3=AF
                let mut sp = regs.get_sp();
                // Read byte baixo
                let lo = bus.cpu_read(sp) as u16;
                // Primeiro incremento de SP (glitched write se SP estava em OAM)
                bus.oam_bug_inc_dec(sp);
                sp = sp.wrapping_add(1);
                // Read byte alto (pode triggerar bug se SP agora está em OAM)
                let hi = bus.cpu_read(sp) as u16;
                // Segundo incremento de SP (também pode triggerar bug)
                bus.oam_bug_inc_dec(sp);
                sp = sp.wrapping_add(1);
                regs.set_sp(sp);
                let val = (hi << 8) | lo;
                match idx {
                    0 => regs.set_bc(val),
                    1 => regs.set_de(val),
                    2 => regs.set_hl(val),
                    3 => regs.set_af(val & 0xFFF0), // Lower 4 bits of F always 0
                    _ => {}
                }
            }
            MicroAction::CallAbsolute => {
                // CALL a16: Empilha PC e salta (24 ciclos)
                // TODO: OAM Bug para CALL (timing precisa ser ajustado)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                let pc_to_push = regs.get_pc();
                // Empilha PC
                let mut sp = regs.get_sp();
                sp = sp.wrapping_sub(1);
                bus.cpu_write(sp, (pc_to_push >> 8) as u8);
                sp = sp.wrapping_sub(1);
                bus.cpu_write(sp, (pc_to_push & 0xFF) as u8);
                regs.set_sp(sp);
                bus.cpu_idle(4);
                regs.set_pc(addr);
            }
            MicroAction::CallAbsoluteConditional { cond } => {
                // CALL cc,a16: Condicional (12 ciclos se não chamar, 24 se chamar)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                let cond_true = match cond {
                    JumpCondition::NZ => !regs.get_flag_z(),
                    JumpCondition::Z => regs.get_flag_z(),
                    JumpCondition::NC => !regs.get_flag_c(),
                    JumpCondition::C => regs.get_flag_c(),
                };
                if cond_true {
                    let pc_to_push = regs.get_pc();
                    let mut sp = regs.get_sp();
                    sp = sp.wrapping_sub(1);
                    bus.cpu_write(sp, (pc_to_push >> 8) as u8);
                    sp = sp.wrapping_sub(1);
                    bus.cpu_write(sp, (pc_to_push & 0xFF) as u8);
                    regs.set_sp(sp);
                    bus.cpu_idle(4);
                    regs.set_pc(addr);
                }
            }
            MicroAction::Return => {
                // RET: Desempilha PC (16 ciclos)
                // TODO: OAM Bug para RET (timing precisa ser ajustado)
                let mut sp = regs.get_sp();
                let lo = bus.cpu_read(sp) as u16;
                sp = sp.wrapping_add(1);
                let hi = bus.cpu_read(sp) as u16;
                sp = sp.wrapping_add(1);
                regs.set_sp(sp);
                let addr = (hi << 8) | lo;
                bus.cpu_idle(4);
                regs.set_pc(addr);
            }
            MicroAction::ReturnConditional { cond } => {
                // RET cc: Condicional (8 ciclos se não retornar, 20 se retornar)
                let cond_true = match cond {
                    JumpCondition::NZ => !regs.get_flag_z(),
                    JumpCondition::Z => regs.get_flag_z(),
                    JumpCondition::NC => !regs.get_flag_c(),
                    JumpCondition::C => regs.get_flag_c(),
                };
                if cond_true {
                    let mut sp = regs.get_sp();
                    let lo = bus.cpu_read(sp) as u16;
                    sp = sp.wrapping_add(1);
                    let hi = bus.cpu_read(sp) as u16;
                    sp = sp.wrapping_add(1);
                    regs.set_sp(sp);
                    let addr = (hi << 8) | lo;
                    bus.cpu_idle(4);
                    regs.set_pc(addr);
                }
                bus.cpu_idle(4);
            }
            MicroAction::Reset { addr } => {
                // RST addr: Empilha PC e salta para endereço (16 ciclos)
                // TODO: OAM Bug para RST (timing precisa ser ajustado)
                let pc = regs.get_pc();
                let mut sp = regs.get_sp();
                sp = sp.wrapping_sub(1);
                bus.cpu_write(sp, (pc >> 8) as u8);
                sp = sp.wrapping_sub(1);
                bus.cpu_write(sp, (pc & 0xFF) as u8);
                regs.set_sp(sp);
                bus.cpu_idle(4);
                regs.set_pc(addr);
            }
            MicroAction::FetchImm16ToReg16 { idx } => {
                // LD rr,d16: Carrega registrador 16-bit com valor imediato (12 ciclos: 4 fetch + 4 lo + 4 hi)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let val = (hi << 8) | lo;
                match idx {
                    0 => regs.set_bc(val),
                    1 => regs.set_de(val),
                    2 => regs.set_hl(val),
                    3 => regs.set_sp(val),
                    _ => {}
                }
                // Total: 12 ciclos (4 fetch já feito + 4 lo + 4 hi)
            }
            MicroAction::FetchImm16AndReadToA => {
                // LD A,(a16): Lê de endereço absoluto para A (16 ciclos: 4 fetch + 4 lo + 4 hi + 4 read)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                let val = bus.cpu_read(addr);
                regs.set_a(val);
                // Total: 16 ciclos
            }
            MicroAction::FetchImm16AndWriteA => {
                // LD (a16),A: Escreve A em endereço absoluto (16 ciclos: 4 fetch + 4 lo + 4 hi + 4 write)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                bus.cpu_write(addr, regs.get_a());
                // Total: 16 ciclos
            }
            MicroAction::FetchImm16AndWriteSP => {
                // LD (a16),SP: Escreve SP em endereço absoluto (20 ciclos: 4 fetch + 4 lo + 4 hi + 4 write lo + 4 write hi)
                let pc = regs.get_pc();
                let lo = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let hi = bus.cpu_read(regs.get_pc()) as u16;
                regs.set_pc(regs.get_pc().wrapping_add(1));
                let addr = (hi << 8) | lo;
                let sp = regs.get_sp();
                bus.cpu_write(addr, (sp & 0xFF) as u8);
                bus.cpu_write(addr.wrapping_add(1), (sp >> 8) as u8);
                // Total: 20 ciclos
            }
            MicroAction::LoadSpFromHl => {
                // LD SP,HL: Carrega SP com HL (8 ciclos: 4 fetch + 4 operação)
                regs.set_sp(regs.get_hl());
                bus.cpu_idle(4); // 8 ciclos totais
            }
            MicroAction::LoadHlFromSpPlusSignedImm8 => {
                // LD HL,SP+r8: Carrega HL com SP + byte assinado (12 ciclos: 4 fetch + 4 ler offset + 4 calcular)
                let pc = regs.get_pc();
                let offset = bus.cpu_read(pc) as i8;
                regs.set_pc(pc.wrapping_add(1));
                let sp = regs.get_sp();
                let result = sp.wrapping_add(offset as i16 as u16);
                regs.set_flag_z(false);
                regs.set_flag_n(false);
                regs.set_flag_h(((sp & 0x0F) + ((offset as u8 as u16) & 0x0F)) > 0x0F);
                regs.set_flag_c(((sp & 0xFF) + (offset as u8 as u16)) > 0xFF);
                regs.set_hl(result);
                bus.cpu_idle(4); // 12 ciclos totais
            }
            MicroAction::WriteAToFF00PlusImm8 => {
                // LDH (n),A: Escreve A em 0xFF00 + offset (12 ciclos: 4 fetch + 4 ler offset + 4 write)
                let pc = regs.get_pc();
                let offset = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                bus.cpu_write(0xFF00 + offset, regs.get_a());
                // Total: 12 ciclos
            }
            MicroAction::ReadFromFF00PlusImm8ToA => {
                // LDH A,(n): Lê de 0xFF00 + offset para A (12 ciclos)
                let pc = regs.get_pc();
                let offset = bus.cpu_read(pc) as u16;
                regs.set_pc(pc.wrapping_add(1));
                let val = bus.cpu_read(0xFF00 + offset);
                regs.set_a(val);
                // Total: 12 ciclos
            }
            MicroAction::WriteAToFF00PlusC => {
                // LD (C),A: Escreve A em 0xFF00 + C (8 ciclos: 4 fetch + 4 write)
                let c = regs.get_c() as u16;
                bus.cpu_write(0xFF00 + c, regs.get_a());
                // Total: 8 ciclos
            }
            MicroAction::ReadFromFF00PlusCToA => {
                // LD A,(C): Lê de 0xFF00 + C para A (8 ciclos)
                let c = regs.get_c() as u16;
                let val = bus.cpu_read(0xFF00 + c);
                regs.set_a(val);
                // Total: 8 ciclos
            }
            MicroAction::WriteAToHlAndIncrement => {
                // LDI (HL),A: Escreve A em (HL) e incrementa HL (8 ciclos: 4 fetch + 4 write)
                let hl = regs.get_hl();
                // OAM Bug: write + inc triggera corrupção (se comporta como uma única write)
                bus.oam_bug_write_inc_dec(hl);
                bus.cpu_write(hl, regs.get_a());
                regs.set_hl(hl.wrapping_add(1));
                // Total: 8 ciclos
            }
            MicroAction::ReadFromHlToAAndIncrement => {
                // LDI A,(HL): Lê de (HL) para A e incrementa HL (8 ciclos)
                let hl = regs.get_hl();
                // OAM Bug: read + inc triggera corrupção complexa
                bus.oam_bug_read_inc_dec(hl);
                let val = bus.cpu_read(hl);
                regs.set_a(val);
                regs.set_hl(hl.wrapping_add(1));
                // Total: 8 ciclos
            }
            MicroAction::WriteAToHlAndDecrement => {
                // LDD (HL),A: Escreve A em (HL) e decrementa HL (8 ciclos)
                let hl = regs.get_hl();
                // OAM Bug: write + dec triggera corrupção (se comporta como uma única write)
                bus.oam_bug_write_inc_dec(hl);
                bus.cpu_write(hl, regs.get_a());
                regs.set_hl(hl.wrapping_sub(1));
                // Total: 8 ciclos
            }
            MicroAction::ReadFromHlToAAndDecrement => {
                // LDD A,(HL): Lê de (HL) para A e decrementa HL (8 ciclos)
                let hl = regs.get_hl();
                // OAM Bug: read + dec triggera corrupção complexa
                bus.oam_bug_read_inc_dec(hl);
                let val = bus.cpu_read(hl);
                regs.set_a(val);
                regs.set_hl(hl.wrapping_sub(1));
                // Total: 8 ciclos
            }
            // === CB-prefix operations ===
            // Nota: CB prefix é tratado de forma especial no CPU.rs antes de chamar execute()
            MicroAction::ExecuteRLC { reg } => {
                // RLC r: Rotate Left Circular (8 ciclos para registrador)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados antes de chamar execute()
                let val = reg.read(regs);
                let bit7 = (val >> 7) & 1;
                let result = (val << 1) | bit7;
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
                // Não adiciona ciclos extras - já temos 8 ciclos totais
            }
            MicroAction::ExecuteRLCHl => {
                // RLC (HL): Rotate Left Circular em memória (16 ciclos)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit7 = (val >> 7) & 1;
                let result = (val << 1) | bit7;
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
                // Total: 16 ciclos (4 fetch CB + 4 fetch opcode + 4 read + 4 write)
            }
            MicroAction::ExecuteRRC { reg } => {
                // RRC r: Rotate Right Circular (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let bit0 = val & 1;
                let result = (val >> 1) | (bit0 << 7);
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteRRCHl => {
                // RRC (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit0 = val & 1;
                let result = (val >> 1) | (bit0 << 7);
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteRL { reg } => {
                // RL r: Rotate Left through Carry (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let old_carry = if regs.get_flag_c() { 1 } else { 0 };
                let bit7 = (val >> 7) & 1;
                let result = (val << 1) | old_carry;
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
            }
            MicroAction::ExecuteRLHl => {
                // RL (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let old_carry = if regs.get_flag_c() { 1 } else { 0 };
                let bit7 = (val >> 7) & 1;
                let result = (val << 1) | old_carry;
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
            }
            MicroAction::ExecuteRR { reg } => {
                // RR r: Rotate Right through Carry (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let old_carry = if regs.get_flag_c() { 1 } else { 0 };
                let bit0 = val & 1;
                let result = (val >> 1) | (old_carry << 7);
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteRRHl => {
                // RR (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let old_carry = if regs.get_flag_c() { 1 } else { 0 };
                let bit0 = val & 1;
                let result = (val >> 1) | (old_carry << 7);
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteSLA { reg } => {
                // SLA r: Shift Left Arithmetic (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let bit7 = (val >> 7) & 1;
                let result = val << 1;
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
            }
            MicroAction::ExecuteSLAHl => {
                // SLA (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit7 = (val >> 7) & 1;
                let result = val << 1;
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit7 == 1);
            }
            MicroAction::ExecuteSRA { reg } => {
                // SRA r: Shift Right Arithmetic (preserva MSB) (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let bit0 = val & 1;
                let bit7 = val & 0x80;
                let result = (val >> 1) | bit7;
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteSRAHl => {
                // SRA (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit0 = val & 1;
                let bit7 = val & 0x80;
                let result = (val >> 1) | bit7;
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteSWAP { reg } => {
                // SWAP r: Troca nibbles (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
            }
            MicroAction::ExecuteSWAPHl => {
                // SWAP (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(false);
            }
            MicroAction::ExecuteSRL { reg } => {
                // SRL r: Shift Right Logical (zero fill) (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let bit0 = val & 1;
                let result = val >> 1;
                reg.write(regs, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::ExecuteSRLHl => {
                // SRL (HL)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit0 = val & 1;
                let result = val >> 1;
                bus.cpu_write(addr, result);
                regs.set_flag_z(result == 0);
                regs.set_flag_n(false);
                regs.set_flag_h(false);
                regs.set_flag_c(bit0 == 1);
            }
            MicroAction::TestBit { bit, reg } => {
                // BIT b,r: Testa bit (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let bit_set = (val & (1 << bit)) != 0;
                regs.set_flag_z(!bit_set);
                regs.set_flag_n(false);
                regs.set_flag_h(true);
            }
            MicroAction::TestBitHl { bit } => {
                // BIT b,(HL): Testa bit em memória (12 ciclos)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let bit_set = (val & (1 << bit)) != 0;
                regs.set_flag_z(!bit_set);
                regs.set_flag_n(false);
                regs.set_flag_h(true);
                // Total: 12 ciclos (4 fetch CB + 4 fetch opcode + 4 read)
            }
            MicroAction::ResetBit { bit, reg } => {
                // RES b,r: Reseta bit (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let result = val & !(1 << bit);
                reg.write(regs, result);
            }
            MicroAction::ResetBitHl { bit } => {
                // RES b,(HL): Reseta bit em memória (16 ciclos)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let result = val & !(1 << bit);
                bus.cpu_write(addr, result);
                // Total: 16 ciclos
            }
            MicroAction::SetBit { bit, reg } => {
                // SET b,r: Seta bit (8 ciclos)
                // 4 ciclos fetch CB + 4 ciclos fetch opcode já foram contados
                let val = reg.read(regs);
                let result = val | (1 << bit);
                reg.write(regs, result);
            }
            MicroAction::SetBitHl { bit } => {
                // SET b,(HL): Seta bit em memória (16 ciclos)
                let addr = regs.get_hl();
                let val = bus.cpu_read(addr);
                let result = val | (1 << bit);
                bus.cpu_write(addr, result);
                // Total: 16 ciclos
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
        .or_else(|| stack::lookup(opcode))
        .or_else(|| cb_prefix::lookup(opcode))
}
