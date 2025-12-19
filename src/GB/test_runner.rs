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

/// Lê saída da porta serial (FF01/FF02)
/// Verifica se houve interrupção serial (transferência completa)
/// Em modo loopback (sem dispositivo conectado), captura o byte transmitido
fn poll_serial(cpu: &mut CPU, log: &mut String) {
    // Verifica se interrupção serial foi disparada (bit 3 do IF)
    let if_reg = cpu.bus.read(0xFF0F);
    if (if_reg & 0x08) != 0 {
        // Limpa flag de interrupção
        cpu.bus.write(0xFF0F, if_reg & !0x08);

        // Em modo loopback, o byte recebido é sempre 0xFF
        // Mas para testes, queremos capturar o byte que foi transmitido
        // Vamos ler SB diretamente (que contém o byte transmitido)
        let byte = cpu.bus.read(0xFF01);

        // Processa byte (em loopback, pode ser 0xFF, mas geralmente é o byte transmitido)
        if (0x20..=0x7E).contains(&byte) || byte == b'\n' || byte == b'\r' {
            log.push(byte as char);
        } else if byte != 0xFF {
            // Mostra bytes não-FF em formato hexadecimal
            log.push_str(&format!("<{:02X}>", byte));
        }
    }
}

/// Executa ROM de teste em modo headless
pub fn run(cpu: &mut CPU) -> TestResult {
    let mut serial_log = String::new();
    let max_cycles: u64 = 4_194_304 * 120; // ~120 segundos
    let mut executed_cycles: u64 = 0;
    let mut steps: u64 = 0;
    let mut last_pc = 0u16;
    let mut same_pc_count = 0u32;

    loop {
        let (cycles, _) = cpu.execute_next();
        steps += 1;

        let pc = cpu.registers.get_pc();

        // Detecta loop infinito (mesmo PC por muitas instruções)
        if pc == last_pc {
            same_pc_count += 1;
            if same_pc_count >= 1000 {
                break;
            }
        } else {
            same_pc_count = 0;
            last_pc = pc;
        }

        if cycles == 0 {
            break;
        }

        executed_cycles = executed_cycles.wrapping_add(cycles);
        poll_serial(cpu, &mut serial_log);

        // Log de progresso e detecção de loop
        if executed_cycles / 1_000_000 != (executed_cycles - cycles as u64) / 1_000_000 {
            let mega_cycles = executed_cycles / 1_000_000;

            // Verifica resultado na memória
            if let Some((status, text)) = check_memory_result(cpu) {
                if status != 0x80 {
                    print_result(status, &text);
                    println!("Serial log:\n{}", serial_log);
                    return if status == 0 {
                        TestResult::Passed
                    } else {
                        TestResult::Failed(status)
                    };
                }
            }
        }

        if serial_log.contains("Passed") || serial_log.contains("PASS") {
            println!("✅ Teste passou!");
            println!("Serial log:\n{}", serial_log);
            return TestResult::Passed;
        }

        if executed_cycles >= max_cycles {
            if let Some((status, text)) = check_memory_result(cpu) {
                print_result(status, &text);
                println!("Serial log:\n{}", serial_log);
                return if status == 0 {
                    TestResult::Passed
                } else if status == 0x80 {
                    TestResult::Timeout
                } else {
                    TestResult::Failed(status)
                };
            }
            break;
        }
    }

    println!("Serial log:\n{}", serial_log);
    TestResult::Timeout
}

fn print_result(status: u8, text: &str) {
    match status {
        0 => println!("✅ Teste passou! (via memória)"),
        0x80 => println!("Teste ainda rodando (status=0x80)"),
        code => println!("❌ Teste falhou com código {} (via memória)", code),
    }
    if !text.is_empty() {
        println!("Resultado: {}", text);
    }
}
