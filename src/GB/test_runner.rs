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

/// Executa ROM de teste em modo headless
pub fn run(cpu: &mut CPU) -> TestResult {
    let mut instruction_count = 0u64;
    let mut last_pc = 0u16;
    let mut stuck_count = 0u32;
    let mut serial_output = String::new();

    // Limites otimizados para captura melhor
    const MAX_INSTRUCTIONS: u64 = 300_000_000; // 300M instruções max
    const STUCK_THRESHOLD: u32 = 200000; // 200k instruções no mesmo PC = travado
    const MEMORY_CHECK_INTERVAL: u64 = 1000; // Verifica memória a cada 1k instruções
    const SERIAL_CHECK_INTERVAL: u64 = 50; // Verifica serial a cada 50 instruções
    const FINAL_CHECK_INTERVAL: u64 = 50000; // Verificação final mais frequente

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
                // Verificação final intensiva antes de desistir
                for _ in 0..20 {
                    if let Some((status, text)) = check_memory_result(cpu) {
                        if status != 0x80 {
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

                    // Verifica serial uma última vez
                    let if_reg = cpu.bus.read(0xFF0F);
                    if (if_reg & 0x08) != 0 {
                        let byte = cpu.bus.read(0xFF01);
                        if (0x20..=0x7E).contains(&byte) || byte == b'\n' || byte == b'\r' {
                            serial_output.push(byte as char);
                        }
                        cpu.bus.clear_if_bits(0x08);
                    }
                }

                // Se CPU está halted, tenta acordar com interrupções
                if cpu.halted {
                    let ie = cpu.bus.get_ie();
                    let if_reg = cpu.bus.get_if();
                    if (ie & if_reg) != 0 {
                        cpu.halted = false;
                        stuck_count = 0;
                        continue;
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

        // Verifica saída serial com alta frequência
        if instruction_count % SERIAL_CHECK_INTERVAL == 0 {
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

        // Verificação final mais intensiva quando se aproxima do limite
        if instruction_count > MAX_INSTRUCTIONS - FINAL_CHECK_INTERVAL {
            if instruction_count % 100 == 0 {
                // Verifica a cada 100 instruções no final
                if let Some((status, text)) = check_memory_result(cpu) {
                    if status != 0x80 {
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
        }

        // Limite de segurança
        if instruction_count >= MAX_INSTRUCTIONS {
            // Última verificação intensiva
            for _ in 0..1000 {
                if let Some((status, text)) = check_memory_result(cpu) {
                    if status != 0x80 {
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
            break;
        }
    }

    // Análise final antes de reportar timeout
    if !serial_output.is_empty() {
        println!("Serial: {}", serial_output);

        // Se há saída serial, pode ser uma falha não detectada
        let lower = serial_output.to_lowercase();
        if lower.contains("fail") || lower.contains("error") || lower.contains("wrong") {
            return TestResult::Failed(1);
        }
    }

    // Última tentativa de capturar resultado da memória
    for _ in 0..100 {
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
    }

    TestResult::Timeout
}
