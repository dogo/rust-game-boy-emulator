// Módulo de instruções - organizadas por categoria

// Submódulos
mod helpers;
mod load;
mod arithmetic;
mod logic;
mod jump;
mod stack;
mod control;
mod cb_prefix;

// Re-exporta tipos públicos
pub use helpers::{Instruction, FlagBits};

// Re-exporta função decode
pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        0x00 => Instruction::nop(),

        // LD r,d8 (B,C,D,E,H,L,(HL),A)
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => load::ld_r_d8(opcode),
        0x36 => load::ld_hl_d8(opcode),

        // LD r,r (0x40-0x7F exceto 0x76 que é HALT)
        0x40..=0x75 | 0x77..=0x7F => load::ld_r_r(opcode),

        // LD A,(BC/DE)
        0x0A => load::ld_a_bc(opcode),
        0x1A => load::ld_a_de(opcode),

        // LD (BC/DE),A
        0x02 => load::ld_bc_a(opcode),
        0x12 => load::ld_de_a(opcode),

        // LD A,(a16) e LD (a16),A
        0xFA => load::ld_a_a16(opcode),
        0xEA => load::ld_a16_a(opcode),

        // LDH
        0xE0 => load::ldh_n_a(opcode),
        0xF0 => load::ldh_a_n(opcode),
        0xE2 => load::ld_c_a(opcode),
        0xF2 => load::ld_a_c(opcode),

        // LDI/LDD
        0x22 => load::ldi_hl_a(opcode),
        0x2A => load::ldi_a_hl(opcode),
        0x32 => load::ldd_hl_a(opcode),
        0x3A => load::ldd_a_hl(opcode),

        // LD 16-bit
        0x01 | 0x11 | 0x21 | 0x31 => load::ld_rr_d16(opcode),
        0x08 => load::ld_a16_sp(opcode),
        0xF9 => load::ld_sp_hl(opcode),
        0xF8 => load::ld_hl_sp_r8(opcode),

        // Arithmetic 8-bit
        // INC r (registradores) e (HL)
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => arithmetic::inc_r(opcode),
        // DEC r (registradores) e (HL)
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => arithmetic::dec_r(opcode),
        0x80..=0x87 => arithmetic::add_a_r(opcode),
        0xC6 => arithmetic::add_a_d8(opcode),
        0x88..=0x8F => arithmetic::adc_a_r(opcode),
        0xCE => arithmetic::adc_a_d8(opcode),
        0x90..=0x97 => arithmetic::sub_a_r(opcode),
        0xD6 => arithmetic::sub_a_d8(opcode),
        0x98..=0x9F => arithmetic::sbc_a_r(opcode),
        0xDE => arithmetic::sbc_a_d8(opcode),

        // Logic
        // Rotates simples sem CB
        0x07 => logic::rlca(opcode),
        0x0F => logic::rrca(opcode),
        0x17 => logic::rla(opcode),
        0x1F => logic::rra(opcode),
        // CPL, SCF, CCF
        0x2F => logic::cpl(opcode),
        0x37 => logic::scf(opcode),
        0x3F => logic::ccf(opcode),
        0xA0..=0xA7 => logic::and_a_r(opcode),
        0xE6 => logic::and_a_d8(opcode),
        0xB0..=0xB7 => logic::or_a_r(opcode),
        0xF6 => logic::or_a_d8(opcode),
        0xA8..=0xAF => logic::xor_a_r(opcode),
        0xEE => logic::xor_a_d8(opcode),
        0xB8..=0xBF => logic::cp_a_r(opcode),
        0xFE => logic::cp_a_d8(opcode),

        // 16-bit arithmetic
        0x03 | 0x13 | 0x23 | 0x33 => arithmetic::inc_rr(opcode),
        0x0B | 0x1B | 0x2B | 0x3B => arithmetic::dec_rr(opcode),
        0x09 | 0x19 | 0x29 | 0x39 => arithmetic::add_hl_rr(opcode),
        0xE8 => arithmetic::add_sp_r8(opcode),

        // DAA (ajuste decimal)
        0x27 => arithmetic::daa(opcode),

        // Jumps
        0xC3 => jump::jp_a16(opcode),
        0xC2 | 0xCA | 0xD2 | 0xDA => jump::jp_cc_a16(opcode),
        0xE9 => jump::jp_hl(opcode),
        0x18 => jump::jr_r8(opcode),
        0x20 | 0x28 | 0x30 | 0x38 => jump::jr_cc_r8(opcode),

        // Stack
        0xCD => stack::call_a16(opcode),
        0xC4 | 0xCC | 0xD4 | 0xDC => stack::call_cc_a16(opcode),
        0xC9 => stack::ret(opcode),
        0xC0 | 0xC8 | 0xD0 | 0xD8 => stack::ret_cc(opcode),
        0xD9 => stack::reti(opcode),
        0xC5 | 0xD5 | 0xE5 | 0xF5 => stack::push(opcode),
        0xC1 | 0xD1 | 0xE1 | 0xF1 => stack::pop(opcode),
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => stack::rst(opcode),

        // Control
        0xF3 => control::di(opcode),
        0xFB => control::ei(opcode),
        0x76 => control::halt(opcode),
        0x10 => control::stop(opcode),

        // CB prefix
        0xCB => cb_prefix::cb(opcode),

        // Opcodes ilegais/não documentados - tratados como NOP
        0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => Instruction::nop(),
    }
}
