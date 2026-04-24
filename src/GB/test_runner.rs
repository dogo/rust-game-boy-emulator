//! Módulo para execução de ROMs de teste (Blargg, Mooneye, etc)
//! Suporta saída via serial (FF01/FF02) e memória ($A000)

use crate::GB::CPU::CPU;
use std::io::{self, Write};

/// Resultado de um teste
#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed(u8),
    Timeout,
}

const BLARGG_STATUS_ADDR: u16 = 0xA000;
const BLARGG_SIGNATURE_ADDR: u16 = 0xA001;
const BLARGG_TEXT_ADDR: u16 = 0xA004;
const BLARGG_TEXT_END: u16 = 0xBFFF;
const BLARGG_RUNNING: u8 = 0x80;

/// Verifica status do resultado na memória $A000 (formato Blargg)
fn check_memory_status(cpu: &CPU) -> Option<u8> {
    let sig1 = cpu.bus.read(BLARGG_SIGNATURE_ADDR);
    let sig2 = cpu.bus.read(BLARGG_SIGNATURE_ADDR + 1);
    let sig3 = cpu.bus.read(BLARGG_SIGNATURE_ADDR + 2);

    // Formato Blargg padrão: assinatura DE B0 61
    if sig1 == 0xDE && sig2 == 0xB0 && sig3 == 0x61 {
        Some(cpu.bus.read(BLARGG_STATUS_ADDR))
    } else {
        None
    }
}

#[derive(Default)]
struct BlarggMemoryOutput {
    next_offset: u16,
    writer_ptr_addr: Option<u16>,
}

impl BlarggMemoryOutput {
    fn drain(&mut self, cpu: &mut CPU) {
        if check_memory_status(cpu).is_none() {
            return;
        }

        let mut addr = BLARGG_TEXT_ADDR.saturating_add(self.next_offset);
        let mut chunk = String::new();

        while addr <= BLARGG_TEXT_END {
            let ch = cpu.bus.read(addr);
            if ch == 0 {
                break;
            }

            if ch.is_ascii() {
                chunk.push(ch as char);
            }

            self.next_offset = self.next_offset.saturating_add(1);
            addr = addr.saturating_add(1);
        }

        if !chunk.is_empty() {
            print!("{chunk}");
            let _ = io::stdout().flush();
        }

        if self.next_offset >= 512 {
            self.compact_buffer(cpu);
        }
    }

    fn compact_buffer(&mut self, cpu: &mut CPU) {
        let writer_value = BLARGG_TEXT_ADDR.saturating_add(self.next_offset);
        let writer_addr = self
            .writer_ptr_addr
            .filter(|addr| self.pointer_matches(cpu, *addr, writer_value))
            .or_else(|| self.find_writer_pointer(cpu, writer_value));

        if let Some(addr) = writer_addr {
            self.writer_ptr_addr = Some(addr);
            cpu.bus.write(addr, (BLARGG_TEXT_ADDR & 0x00FF) as u8);
            cpu.bus.write(addr + 1, (BLARGG_TEXT_ADDR >> 8) as u8);
            cpu.bus.write(BLARGG_TEXT_ADDR, 0);
            self.next_offset = 0;
        }
    }

    fn pointer_matches(&self, cpu: &CPU, addr: u16, value: u16) -> bool {
        cpu.bus.read(addr) == (value & 0x00FF) as u8 && cpu.bus.read(addr + 1) == (value >> 8) as u8
    }

    fn find_writer_pointer(&self, cpu: &CPU, value: u16) -> Option<u16> {
        // O shell do Blargg mantém o cursor de escrita de $A004 no BSS da WRAM.
        // Encontrar esse ponteiro permite ao runner headless emitir diagnósticos
        // verbosos sem deixar o buffer fixo de 8 KiB da RAM do cartucho vazar para
        // o código em WRAM.
        (0xD800..0xDA00).find(|addr| self.pointer_matches(cpu, *addr, value))
    }
}

fn drain_serial_output(cpu: &mut CPU, serial_output: &mut String) -> bool {
    if cpu.bus.serial_output_buffer.is_empty() {
        return false;
    }

    let bytes: Vec<u8> = cpu.bus.serial_output_buffer.drain(..).collect();
    for byte in bytes {
        if (0x20..=0x7E).contains(&byte) || byte == b'\n' || byte == b'\r' {
            serial_output.push(byte as char);
        }
    }

    true
}

fn serial_result(serial_output: &str) -> Option<TestResult> {
    let lower = serial_output.to_lowercase();
    if lower.contains("passed") || lower.contains("pass") {
        println!("{serial_output}");
        Some(TestResult::Passed)
    } else if lower.contains("failed") || lower.contains("fail") {
        println!("{serial_output}");
        Some(TestResult::Failed(1))
    } else {
        None
    }
}

fn memory_result(
    cpu: &mut CPU,
    memory_output: &mut BlarggMemoryOutput,
    serial_output: &str,
) -> Option<TestResult> {
    memory_output.drain(cpu);

    let status = check_memory_status(cpu)?;
    if status == BLARGG_RUNNING {
        return None;
    }

    memory_output.drain(cpu);
    if !serial_output.is_empty() {
        println!("Serial: {serial_output}");
    }

    Some(if status == 0 {
        TestResult::Passed
    } else {
        TestResult::Failed(status)
    })
}

/// Executa ROM de teste em modo headless
pub fn run(cpu: &mut CPU) -> TestResult {
    // Desabilita renderização gráfica para ganho de performance em testes
    cpu.bus.ppu.headless = true;

    let mut instruction_count = 0u64;
    let mut last_pc = 0u16;
    let mut stuck_count = 0u32;
    let mut serial_output = String::new();
    let mut memory_output = BlarggMemoryOutput::default();

    const MAX_INSTRUCTIONS: u64 = 3_000_000_000;
    const STUCK_THRESHOLD: u32 = 200000; // 200k instruções no mesmo PC = travado
    const MEMORY_CHECK_INTERVAL: u64 = 100_000; // Verifica memória a cada 100k instruções
    const FINAL_CHECK_INTERVAL: u64 = 1_000_000; // Intensifica checks só perto do limite

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
                    if let Some(result) = memory_result(cpu, &mut memory_output, &serial_output) {
                        return result;
                    }

                    if drain_serial_output(cpu, &mut serial_output) {
                        if let Some(result) = serial_result(&serial_output) {
                            return result;
                        }
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
            if let Some(result) = memory_result(cpu, &mut memory_output, &serial_output) {
                return result;
            }
        }

        // Drena buffer serial (bytes capturados no momento da transferência)
        if drain_serial_output(cpu, &mut serial_output) {
            if let Some(result) = serial_result(&serial_output) {
                return result;
            }
        }

        // Verificação final mais intensiva quando se aproxima do limite
        if instruction_count > MAX_INSTRUCTIONS - FINAL_CHECK_INTERVAL {
            if instruction_count % 100 == 0 {
                // Verifica a cada 100 instruções no final
                if let Some(result) = memory_result(cpu, &mut memory_output, &serial_output) {
                    return result;
                }
            }
        }

        // Limite de segurança
        if instruction_count >= MAX_INSTRUCTIONS {
            // Última verificação intensiva
            for _ in 0..1000 {
                if let Some(result) = memory_result(cpu, &mut memory_output, &serial_output) {
                    return result;
                }
            }
            break;
        }
    }

    // Análise final antes de reportar timeout
    if !serial_output.is_empty() {
        println!("Serial: {serial_output}");

        // Se há saída serial, pode ser uma falha não detectada
        let lower = serial_output.to_lowercase();
        if lower.contains("fail") || lower.contains("error") || lower.contains("wrong") {
            return TestResult::Failed(1);
        }
    }

    // Última tentativa de capturar resultado da memória
    for _ in 0..100 {
        if let Some(result) = memory_result(cpu, &mut memory_output, &serial_output) {
            return result;
        }
    }

    if let Some(status) = check_memory_status(cpu) {
        if status == BLARGG_RUNNING {
            memory_output.drain(cpu);
            eprintln!("timeout-debug-text: Blargg memory status is still running");
        }
    }

    TestResult::Timeout
}
