// Módulo de instruções - organizadas por categoria
//
// NOTA: A maioria das instruções foi migrada para o sistema de microcode.
// Todas as instruções principais (LD, Arithmetic, Logic, Jump, Stack, CB-prefix)
// estão agora em src/GB/microcode/.
// Apenas as instruções de controle (DI, EI, HALT, STOP) ainda estão aqui
// porque têm efeitos especiais no CPU que são tratados externamente.

// Submódulos
mod control;
mod helpers;

// Re-exporta tipos públicos
pub use helpers::{FlagBits, Instruction};

// Re-exporta função decode
pub fn decode(opcode: u8) -> Instruction {
    match opcode {
        // NOP
        0x00 => Instruction::nop(),

        // Control - não migrado (instruções simples com efeitos especiais no CPU)
        0xF3 => control::di(opcode),
        0xFB => control::ei(opcode),
        0x76 => control::halt(opcode),
        0x10 => control::stop(opcode),

        // Opcodes ilegais/não documentados - tratados como NOP
        0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
            Instruction::nop()
        }

        // Todas as outras instruções foram migradas para microcode.
        // Este padrão nunca deve ser alcançado porque microcode::lookup() tem prioridade
        // no CPU.rs. Mantido apenas para satisfazer o match exaustivo do Rust.
        _ => Instruction::unknown(opcode),
    }
}
