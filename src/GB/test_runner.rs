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
        // Verifica se há dados válidos (não só 0xFF ou 0x00)
        let status = cpu.bus.read(0xA000);

        // Ignora se for só 0xFF (RAM não inicializada) ou 0x00 (vazio)
        if status == 0xFF || status == 0x00 {
            return None;
        }

        // Verifica se há texto legível na área
        let mut text = String::new();
        let mut has_text = false;
        for i in 0..256 {
            let ch = cpu.bus.read(0xA000 + i);
            if ch == 0 {
                break;
            }
            if ch.is_ascii() && ch >= 0x20 && ch <= 0x7E {
                text.push(ch as char);
                has_text = true;
            } else if ch != 0xFF {
                text.push_str(&format!("<{:02X}>", ch));
            }
        }

        if has_text && text.len() > 5 {
            Some((status, text))
        } else {
            None
        }
    }
}

/// Lê saída da porta serial (FF01/FF02)
/// Verifica se houve interrupção serial (transferência completa)
/// Em modo loopback (sem dispositivo conectado), captura o byte transmitido
fn poll_serial(cpu: &mut CPU, log: &mut String) {
    // Verifica se interrupção serial foi disparada (bit 3 do IF)
    let if_reg = cpu.bus.read(0xFF0F);
    if (if_reg & 0x08) != 0 {
        // Lê o byte da porta serial
        let byte = cpu.bus.read(0xFF01);

        // Processa byte
        if (0x20..=0x7E).contains(&byte) || byte == b'\n' || byte == b'\r' {
            log.push(byte as char);
        } else if byte != 0xFF && byte != 0x00 {
            log.push_str(&format!("<{:02X}>", byte));
        }
    }
}

/// Executa ROM de teste em modo headless
pub fn run(cpu: &mut CPU) -> TestResult {
    let mut serial_log = String::new();
    let max_cycles: u64 = 4_194_304 * 30; // 30 segundos
    let mut executed_cycles: u64 = 0;
    let mut last_pc = 0u16;
    let mut same_pc_count = 0u32;

    loop {
        let (cycles, _) = cpu.execute_next();

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

        // Verifica resultado na memória com mais frequência
        if executed_cycles % 1000 == 0 {
            if let Some((status, text)) = check_memory_result(cpu) {
                if status != 0x80 {
                    if !text.is_empty() {
                        println!("{}", text);
                    }
                    if !serial_log.is_empty() {
                        println!("Serial: {}", serial_log);
                    }
                    return if status == 0 {
                        TestResult::Passed
                    } else {
                        TestResult::Failed(status)
                    };
                }
            }
        }

        // Verifica padrões de sucesso/falha na saída serial
        if !serial_log.is_empty() {
            let log_lower = serial_log.to_lowercase();

            // Padrões de sucesso
            if log_lower.contains("passed")
                || log_lower.contains("pass")
                || log_lower.contains("ok")
                || log_lower.contains("success")
                || serial_log.ends_with("Passed")
                || serial_log.ends_with("OK")
            {
                println!("{}", serial_log);
                return TestResult::Passed;
            }

            // Padrões de falha
            if log_lower.contains("failed")
                || log_lower.contains("fail")
                || log_lower.contains("error")
                || log_lower.contains("wrong")
            {
                println!("{}", serial_log);
                return TestResult::Failed(1);
            }
        }

        if executed_cycles >= max_cycles {
            if let Some((status, text)) = check_memory_result(cpu) {
                if !text.is_empty() {
                    println!("{}", text);
                }
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

    if !serial_log.is_empty() {
        println!("{}", serial_log);
    }
    TestResult::Timeout
}
