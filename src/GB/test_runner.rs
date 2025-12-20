//! Módulo para execução de ROMs de teste (Blargg, Mooneye, etc)
//! Suporta saída via serial (FF01/FF02) e memória ($A000)

use crate::GB::CPU::CPU;

/// Resultado de um teste
#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed(u8),
    Timeout,
}

/// Verifica resultado na memória $A000 (formato Blargg)
fn check_memory_result(cpu: &CPU) -> Option<(u8, String)> {
    let sig1 = cpu.bus.read(0xA001);
    let sig2 = cpu.bus.read(0xA002);
    let sig3 = cpu.bus.read(0xA003);

    // Formato Blargg padrão: assinatura DE B0 61
    if sig1 == 0xDE && sig2 == 0xB0 && sig3 == 0x61 {
        let status = cpu.bus.read(0xA000);
        let mut text = String::new();
        for i in 0..1024 {
            let ch = cpu.bus.read(0xA004 + i);
            if ch == 0 {
                break;
            }
            if ch.is_ascii() {
                text.push(ch as char);
            }
        }
        Some((status, text))
    } else {
        None
    }
}

/// Executa ROM de teste em modo headless - VERSÃO OTIMIZADA PARA CPU TESTS
pub fn run(cpu: &mut CPU) -> TestResult {
    let mut instruction_count = 0u64;
    let mut last_pc = 0u16;
    let mut stuck_count = 0u32;
    let mut serial_output = String::new();

    // Limites otimizados para CPU tests
    const MAX_INSTRUCTIONS: u64 = 50_000_000; // 50M instruções max
    const STUCK_THRESHOLD: u32 = 10000; // 10k instruções no mesmo PC = travado
    const MEMORY_CHECK_INTERVAL: u64 = 1000; // Verifica memória a cada 1k instruções
    const SERIAL_CHECK_INTERVAL: u64 = 100; // Verifica serial a cada 100 instruções

    loop {
        // Executa uma instrução
        let (cycles, _) = cpu.execute_next();
        instruction_count += 1;

        if cycles == 0 {
            break; // CPU parou
        }

        let pc = cpu.registers.get_pc();

        // Detecta se está travado no mesmo PC
        if pc == last_pc {
            stuck_count += 1;
            if stuck_count >= STUCK_THRESHOLD {
                // CPU travado - verifica resultado final
                if let Some((status, text)) = check_memory_result(cpu) {
                    if status != 0x80 {
                        if !text.is_empty() {
                            println!("{}", text);
                        }
                        return if status == 0 {
                            TestResult::Passed
                        } else {
                            TestResult::Failed(status)
                        };
                    }
                }
                break;
            }
        } else {
            stuck_count = 0;
            last_pc = pc;
        }

        // Verifica resultado na memória periodicamente
        if instruction_count % MEMORY_CHECK_INTERVAL == 0 {
            if let Some((status, text)) = check_memory_result(cpu) {
                if status != 0x80 {
                    // 0x80 = ainda executando
                    if !text.is_empty() {
                        println!("{}", text);
                    }
                    if !serial_output.is_empty() {
                        println!("Serial: {}", serial_output);
                    }
                    return if status == 0 {
                        TestResult::Passed
                    } else {
                        TestResult::Failed(status)
                    };
                }
            }
        }

        // Verifica saída serial com mais frequência
        if instruction_count % SERIAL_CHECK_INTERVAL == 0 {
            // Captura qualquer atividade serial
            let if_reg = cpu.bus.read(0xFF0F);
            if (if_reg & 0x08) != 0 {
                let byte = cpu.bus.read(0xFF01);
                if (0x20..=0x7E).contains(&byte) || byte == b'\n' || byte == b'\r' {
                    serial_output.push(byte as char);
                }
                cpu.bus.clear_if_bits(0x08);

                // Verifica padrões de sucesso/falha imediatamente
                let lower = serial_output.to_lowercase();
                if lower.contains("passed") || lower.contains("pass") {
                    println!("{}", serial_output);
                    return TestResult::Passed;
                }
                if lower.contains("failed") || lower.contains("fail") {
                    println!("{}", serial_output);
                    return TestResult::Failed(1);
                }
            }
        }

        // Limite de segurança
        if instruction_count >= MAX_INSTRUCTIONS {
            // Última verificação antes do timeout
            if let Some((status, text)) = check_memory_result(cpu) {
                if status != 0x80 {
                    if !text.is_empty() {
                        println!("{}", text);
                    }
                    return if status == 0 {
                        TestResult::Passed
                    } else {
                        TestResult::Failed(status)
                    };
                }
            }
            break;
        }
    }

    // Timeout - mostra qualquer saída capturada
    if !serial_output.is_empty() {
        println!("Serial: {}", serial_output);
    }
    TestResult::Timeout
}
