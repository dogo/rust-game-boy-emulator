/// Debugger interativo para o emulador Game Boy
/// Baseado em: https://aquova.net/emudev/gb/23-debugger.html
/// Suporta modo single-thread e multi-thread (via channels)
use crate::GB::CPU::CPU;
use crate::GB::instructions;
use crate::GB::microcode;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

// =============================================================================
// COMANDOS E RESPOSTAS (para modo threaded)
// =============================================================================

/// Comandos de debug enviados para a thread de emulação
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Continue,
    Quit,
    Step,
    StepN(usize),
    ShowRegisters,
    ShowMemory(u16, usize),
    ShowIO,
    ShowStack(usize),
    Disassemble(usize),
    AddBreakpoint(u16),
    RemoveBreakpoint(u16),
    AddWatchpoint(u16),
    ListBreakpoints,
}

/// Respostas da thread de emulação
#[derive(Debug)]
pub enum DebugResponse {
    Text(String),
    Resume,
    Quit,
}

// =============================================================================
// DEBUGGER STRUCT
// =============================================================================

pub struct Debugger {
    debugging: bool,
    breakpoints: Vec<u16>,
    watchpoints: Vec<u16>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            debugging: false,
            breakpoints: Vec::new(),
            watchpoints: Vec::new(),
        }
    }

    // =========================================================================
    // GETTERS / SETTERS
    // =========================================================================

    pub fn is_debugging(&self) -> bool {
        self.debugging
    }

    pub fn set_debugging(&mut self, debug: bool) {
        self.debugging = debug;
    }

    pub fn get_breakpoints(&self) -> &[u16] {
        &self.breakpoints
    }

    pub fn check_breakpoint(&self, pc: u16) -> bool {
        self.breakpoints.contains(&pc)
    }

    // =========================================================================
    // BREAKPOINT / WATCHPOINT MANAGEMENT
    // =========================================================================

    pub fn add_breakpoint(&mut self, addr: u16) -> String {
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
            format!("✅ Breakpoint adicionado em 0x{:04X}", addr)
        } else {
            format!("⚠️  Breakpoint já existe em 0x{:04X}", addr)
        }
    }

    pub fn remove_breakpoint(&mut self, addr: u16) -> String {
        if let Some(pos) = self.breakpoints.iter().position(|&x| x == addr) {
            self.breakpoints.remove(pos);
            format!("🗑️  Breakpoint removido de 0x{:04X}", addr)
        } else {
            format!("⚠️  Nenhum breakpoint em 0x{:04X}", addr)
        }
    }

    pub fn add_watchpoint(&mut self, addr: u16) -> String {
        if !self.watchpoints.contains(&addr) {
            self.watchpoints.push(addr);
            format!("👁️  Watchpoint adicionado em 0x{:04X}", addr)
        } else {
            format!("⚠️  Watchpoint já existe em 0x{:04X}", addr)
        }
    }

    pub fn list_breakpoints(&self) -> String {
        let mut result = String::new();
        if self.breakpoints.is_empty() {
            result.push_str("Nenhum breakpoint definido\n");
        } else {
            result.push_str("Breakpoints:\n");
            for (i, &addr) in self.breakpoints.iter().enumerate() {
                result.push_str(&format!("  {}: 0x{:04X}\n", i, addr));
            }
        }
        if !self.watchpoints.is_empty() {
            result.push_str("Watchpoints:\n");
            for (i, &addr) in self.watchpoints.iter().enumerate() {
                result.push_str(&format!("  {}: 0x{:04X}\n", i, addr));
            }
        }
        result
    }

    // =========================================================================
    // FORMATAÇÃO (retorna String para uso em ambos os modos)
    // =========================================================================

    pub fn format_registers(cpu: &CPU) -> String {
        let regs = &cpu.registers;
        format!(
            "┌─────────────────────────────────────┐\n\
             │           REGISTRADORES             │\n\
             ├─────────────────────────────────────┤\n\
             │  AF: {:04X}    BC: {:04X}             │\n\
             │  DE: {:04X}    HL: {:04X}             │\n\
             │  SP: {:04X}    PC: {:04X}             │\n\
             ├─────────────────────────────────────┤\n\
             │  Flags: Z={} N={} H={} C={}            │\n\
             ├─────────────────────────────────────┤\n\
             │  IME: {}  HALT: {}  STOP: {}           │\n\
             │  Cycles: {:>10}                 │\n\
             └─────────────────────────────────────┘",
            regs.get_af(),
            regs.get_bc(),
            regs.get_de(),
            regs.get_hl(),
            regs.get_sp(),
            regs.get_pc(),
            regs.get_flag_z() as u8,
            regs.get_flag_n() as u8,
            regs.get_flag_h() as u8,
            regs.get_flag_c() as u8,
            cpu.ime as u8,
            cpu.halted as u8,
            cpu.stopped as u8,
            cpu.cycles
        )
    }

    pub fn format_memory(cpu: &CPU, addr: u16, count: usize) -> String {
        let mut result = format!("Memória a partir de 0x{:04X}:\n", addr);
        let mut current = addr;
        for _ in 0..((count + 15) / 16) {
            result.push_str(&format!("  {:04X}: ", current));
            let mut ascii = String::new();
            for _ in 0..16 {
                let byte = cpu.bus.read(current);
                result.push_str(&format!("{:02X} ", byte));
                ascii.push(if byte >= 0x20 && byte < 0x7F {
                    byte as char
                } else {
                    '.'
                });
                current = current.wrapping_add(1);
            }
            result.push_str(&format!(" |{}|\n", ascii));
        }
        result
    }

    pub fn format_io(cpu: &CPU) -> String {
        format!(
            "┌─────────────────────────────────────┐\n\
             │         REGISTRADORES I/O           │\n\
             ├─────────────────────────────────────┤\n\
             │  P1/JOYP: {:02X}    DIV:  {:02X}           │\n\
             │  TIMA:    {:02X}    TMA:  {:02X}           │\n\
             │  TAC:     {:02X}    IF:   {:02X}           │\n\
             ├─────────────────────────────────────┤\n\
             │  LCDC: {:02X}  STAT: {:02X}  LY: {:02X}        │\n\
             │  SCY:  {:02X}  SCX:  {:02X}  LYC: {:02X}       │\n\
             │  WY:   {:02X}  WX:   {:02X}  BGP: {:02X}       │\n\
             ├─────────────────────────────────────┤\n\
             │  IE: {:02X}                             │\n\
             └─────────────────────────────────────┘",
            cpu.bus.read(0xFF00),
            cpu.bus.read(0xFF04),
            cpu.bus.read(0xFF05),
            cpu.bus.read(0xFF06),
            cpu.bus.read(0xFF07),
            cpu.bus.read(0xFF0F),
            cpu.bus.read(0xFF40),
            cpu.bus.read(0xFF41),
            cpu.bus.read(0xFF44),
            cpu.bus.read(0xFF42),
            cpu.bus.read(0xFF43),
            cpu.bus.read(0xFF45),
            cpu.bus.read(0xFF4A),
            cpu.bus.read(0xFF4B),
            cpu.bus.read(0xFF47),
            cpu.bus.read(0xFFFF)
        )
    }

    pub fn format_stack(cpu: &CPU, count: usize) -> String {
        let sp = cpu.registers.get_sp();
        let mut result = format!("Stack (SP=0x{:04X}):\n", sp);
        for i in 0..count {
            let addr = sp.wrapping_add(i as u16 * 2);
            let lo = cpu.bus.read(addr);
            let hi = cpu.bus.read(addr.wrapping_add(1));
            let val = ((hi as u16) << 8) | lo as u16;
            result.push_str(&format!("  {:04X}: {:04X}\n", addr, val));
        }
        result
    }

    pub fn format_disassembly(cpu: &CPU, count: usize) -> String {
        let mut result = String::from("Disassembly:\n");
        let mut pc = cpu.registers.get_pc();

        for _ in 0..count {
            let opcode = cpu.bus.read(pc);
            let name = instruction_name(cpu, pc);
            let len = get_instruction_length(opcode);

            let mut bytes = format!("{:02X}", opcode);
            for i in 1..len {
                bytes.push_str(&format!(" {:02X}", cpu.bus.read(pc.wrapping_add(i as u16))));
            }

            let operands = format_operands(cpu, pc, opcode, len);
            let marker = if pc == cpu.registers.get_pc() {
                "→"
            } else {
                " "
            };
            result.push_str(&format!(
                "{} {:04X}:  {:<12} {:<8} {}\n",
                marker, pc, bytes, name, operands
            ));

            pc = pc.wrapping_add(len as u16);
        }
        result
    }

    pub fn format_current_state(cpu: &CPU, cycles: u64) -> String {
        let pc = cpu.registers.get_pc();
        let opcode = cpu.bus.read(pc);
        let name = instruction_name(cpu, pc);
        let len = get_instruction_length(opcode);

        let mut bytes = format!("{:02X}", opcode);
        for i in 1..len {
            bytes.push_str(&format!(" {:02X}", cpu.bus.read(pc.wrapping_add(i as u16))));
        }

        format!(
            "→ {:04X}: {:<12} {:<8} | AF={:04X} BC={:04X} DE={:04X} HL={:04X} ({} cycles)",
            pc,
            bytes,
            name,
            cpu.registers.get_af(),
            cpu.registers.get_bc(),
            cpu.registers.get_de(),
            cpu.registers.get_hl(),
            cycles
        )
    }

    // =========================================================================
    // EXECUÇÃO DE COMANDOS (retorna resultado como String)
    // =========================================================================

    /// Executa uma instrução e retorna o estado
    pub fn step(cpu: &mut CPU) -> String {
        let (cycles, _) = cpu.execute_next();
        Self::format_current_state(cpu, cycles)
    }

    /// Executa N instruções, verificando breakpoints
    pub fn step_n(&mut self, cpu: &mut CPU, n: usize) -> String {
        let mut output = String::new();
        for i in 0..n {
            if i > 0 && self.breakpoints.contains(&cpu.registers.get_pc()) {
                output.push_str(&format!(
                    "🔴 Breakpoint hit at 0x{:04X} após {} instruções\n",
                    cpu.registers.get_pc(),
                    i
                ));
                break;
            }

            let (cycles, unknown) = cpu.execute_next();
            if unknown {
                output.push_str(&format!(
                    "⚠️  Opcode desconhecido em 0x{:04X}\n",
                    cpu.registers.get_pc()
                ));
                break;
            }

            if i < 10 || i == n - 1 {
                output.push_str(&Self::format_current_state(cpu, cycles));
                output.push('\n');
            } else if i == 10 {
                output.push_str(&format!("  ... ({} instruções restantes)\n", n - 10));
            }
        }
        output.push_str(&format!("✅ Executadas {} instruções", n));
        output
    }

    // =========================================================================
    // MODO SINGLE-THREAD (loop original)
    // =========================================================================

    /// Verifica se PC está em um breakpoint (modo single-thread)
    pub fn check_breakpoints(&mut self, pc: u16) {
        if self.breakpoints.contains(&pc) {
            println!("\n🔴 Breakpoint hit at 0x{:04X}", pc);
            self.debugging = true;
        }
    }

    /// Loop principal do debugger - retorna true se deve sair do emulador
    pub fn debugloop(&mut self, cpu: &mut CPU) -> bool {
        loop {
            print!("(gbd) ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            // Sem terminal (EOF), repetir o prompt vira um laço infinito
            if !matches!(io::stdin().read_line(&mut input), Ok(n) if n > 0) {
                println!("\n⚠️  Terminal indisponível; retomando execução");
                self.debugging = false;
                return false;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let words: Vec<&str> = input.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }

            match words[0] {
                "q" | "quit" => return true,
                "c" | "continue" => {
                    self.debugging = false;
                    return false;
                }
                "n" | "next" | "s" | "step" => {
                    println!("{}", Self::step(cpu));
                }
                "b" | "break" => {
                    if words.len() < 2 {
                        println!("Uso: b <endereço>");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        println!("{}", self.add_breakpoint(addr));
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "d" | "delete" => {
                    if words.len() < 2 {
                        println!("Uso: d <endereço>");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        println!("{}", self.remove_breakpoint(addr));
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "l" | "list" => println!("{}", self.list_breakpoints()),
                "reg" | "r" => println!("{}", Self::format_registers(cpu)),
                "p" | "print" | "x" => {
                    if words.len() < 2 {
                        println!("Uso: p <endereço> [quantidade]");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        let count = words.get(2).and_then(|s| s.parse().ok()).unwrap_or(16);
                        println!("{}", Self::format_memory(cpu, addr, count));
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "disass" | "dis" => {
                    let count = words.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
                    println!("{}", Self::format_disassembly(cpu, count));
                }
                "io" => println!("{}", Self::format_io(cpu)),
                "stack" => {
                    let count = words.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
                    println!("{}", Self::format_stack(cpu, count));
                }
                "run" => {
                    if words.len() < 2 {
                        println!("Uso: run <número de instruções>");
                        continue;
                    }
                    if let Ok(n) = words[1].parse::<usize>() {
                        println!("{}", self.step_n(cpu, n));
                    } else {
                        println!("Número inválido: {}", words[1]);
                    }
                }
                "w" | "watch" => {
                    if words.len() < 2 {
                        println!("Uso: w <endereço>");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        println!("{}", self.add_watchpoint(addr));
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "h" | "help" | "?" => self.print_help(),
                _ => println!(
                    "Comando desconhecido: '{}'. Digite 'h' para ajuda.",
                    words[0]
                ),
            }
        }
    }

    // =========================================================================
    // MODO MULTI-THREAD (via channels)
    // =========================================================================

    /// Processa um comando de debug e retorna a resposta
    pub fn process_command(&mut self, cmd: DebugCommand, cpu: &mut CPU) -> DebugResponse {
        match cmd {
            DebugCommand::Continue => DebugResponse::Resume,
            DebugCommand::Quit => DebugResponse::Quit,
            DebugCommand::Step => DebugResponse::Text(Self::step(cpu)),
            DebugCommand::StepN(n) => DebugResponse::Text(self.step_n(cpu, n)),
            DebugCommand::ShowRegisters => DebugResponse::Text(Self::format_registers(cpu)),
            DebugCommand::ShowMemory(addr, count) => {
                DebugResponse::Text(Self::format_memory(cpu, addr, count))
            }
            DebugCommand::ShowIO => DebugResponse::Text(Self::format_io(cpu)),
            DebugCommand::ShowStack(count) => DebugResponse::Text(Self::format_stack(cpu, count)),
            DebugCommand::Disassemble(count) => {
                DebugResponse::Text(Self::format_disassembly(cpu, count))
            }
            DebugCommand::AddBreakpoint(addr) => DebugResponse::Text(self.add_breakpoint(addr)),
            DebugCommand::RemoveBreakpoint(addr) => {
                DebugResponse::Text(self.remove_breakpoint(addr))
            }
            DebugCommand::AddWatchpoint(addr) => DebugResponse::Text(self.add_watchpoint(addr)),
            DebugCommand::ListBreakpoints => DebugResponse::Text(self.list_breakpoints()),
        }
    }

    /// Loop de debug para modo threaded (roda na thread de emulação)
    pub fn debug_command_loop(
        &mut self,
        cpu: &mut CPU,
        cmd_rx: &Receiver<DebugCommand>,
        resp_tx: &Sender<DebugResponse>,
        running: &AtomicBool,
    ) -> bool {
        loop {
            match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(cmd) => {
                    let response = self.process_command(cmd, cpu);
                    let should_exit = matches!(response, DebugResponse::Quit);
                    let should_resume = matches!(response, DebugResponse::Resume);

                    let _ = resp_tx.send(response);

                    if should_exit {
                        return true; // Sair do emulador
                    }
                    if should_resume {
                        return false; // Continuar execução
                    }
                }
                // A interface pode fechar (ESC) sem mandar Quit
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if !running.load(Ordering::Relaxed) {
                        return true;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return true,
            }
        }
    }

    /// Loop de input do terminal para modo threaded (roda no main thread)
    pub fn terminal_input_loop(
        cmd_tx: &Sender<DebugCommand>,
        resp_rx: &Receiver<DebugResponse>,
    ) -> bool {
        println!("\n🎮 GBD - Game Boy Debugger");
        println!("Digite 'h' para ajuda\n");

        loop {
            print!("(gbd) ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            // Sem terminal (EOF), repetir o prompt vira um laço infinito
            if !matches!(io::stdin().read_line(&mut input), Ok(n) if n > 0) {
                println!("\n⚠️  Terminal indisponível; retomando emulação");
                input = String::from("c");
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let words: Vec<&str> = input.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }

            let cmd = match words[0] {
                "q" | "quit" => DebugCommand::Quit,
                "c" | "continue" => DebugCommand::Continue,
                "n" | "next" | "s" | "step" => DebugCommand::Step,
                "run" => {
                    if words.len() < 2 {
                        println!("Uso: run <número de instruções>");
                        continue;
                    }
                    match words[1].parse::<usize>() {
                        Ok(n) => DebugCommand::StepN(n),
                        Err(_) => {
                            println!("Número inválido: {}", words[1]);
                            continue;
                        }
                    }
                }
                "reg" | "r" => DebugCommand::ShowRegisters,
                "p" | "print" | "x" => {
                    if words.len() < 2 {
                        println!("Uso: p <endereço> [quantidade]");
                        continue;
                    }
                    match parse_address(words[1]) {
                        Some(addr) => {
                            let count = words.get(2).and_then(|s| s.parse().ok()).unwrap_or(16);
                            DebugCommand::ShowMemory(addr, count)
                        }
                        None => {
                            println!("Endereço inválido: {}", words[1]);
                            continue;
                        }
                    }
                }
                "io" => DebugCommand::ShowIO,
                "stack" => {
                    let count = words.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
                    DebugCommand::ShowStack(count)
                }
                "disass" | "dis" => {
                    let count = words.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
                    DebugCommand::Disassemble(count)
                }
                "b" | "break" => {
                    if words.len() < 2 {
                        println!("Uso: b <endereço>");
                        continue;
                    }
                    match parse_address(words[1]) {
                        Some(addr) => DebugCommand::AddBreakpoint(addr),
                        None => {
                            println!("Endereço inválido: {}", words[1]);
                            continue;
                        }
                    }
                }
                "d" | "delete" => {
                    if words.len() < 2 {
                        println!("Uso: d <endereço>");
                        continue;
                    }
                    match parse_address(words[1]) {
                        Some(addr) => DebugCommand::RemoveBreakpoint(addr),
                        None => {
                            println!("Endereço inválido: {}", words[1]);
                            continue;
                        }
                    }
                }
                "w" | "watch" => {
                    if words.len() < 2 {
                        println!("Uso: w <endereço>");
                        continue;
                    }
                    match parse_address(words[1]) {
                        Some(addr) => DebugCommand::AddWatchpoint(addr),
                        None => {
                            println!("Endereço inválido: {}", words[1]);
                            continue;
                        }
                    }
                }
                "l" | "list" => DebugCommand::ListBreakpoints,
                "h" | "help" | "?" => {
                    Self::print_help_static();
                    continue;
                }
                _ => {
                    println!(
                        "Comando desconhecido: '{}'. Digite 'h' para ajuda.",
                        words[0]
                    );
                    continue;
                }
            };

            if cmd_tx.send(cmd).is_err() {
                println!("Erro: thread de emulação desconectada");
                return true;
            }

            match resp_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(DebugResponse::Text(text)) => println!("{}", text),
                Ok(DebugResponse::Resume) => return false,
                Ok(DebugResponse::Quit) => return true,
                Err(_) => println!("Timeout esperando resposta"),
            }
        }
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    pub fn print_info(&self) {
        println!("\n🎮 GBD - Game Boy Debugger");
        println!("Digite 'h' para ajuda\n");
    }

    fn print_help(&self) {
        Self::print_help_static();
    }

    fn print_help_static() {
        println!(
            "
┌─────────────────────────────────────────────────────────────┐
│                    GBD - Game Boy Debugger                  │
├─────────────────────────────────────────────────────────────┤
│  CONTROLE                                                   │
│    c, continue    Continua execução                         │
│    n, next, step  Executa próxima instrução                 │
│    run <N>        Executa N instruções                      │
│    q, quit        Sai do emulador                           │
├─────────────────────────────────────────────────────────────┤
│  BREAKPOINTS                                                │
│    b <addr>       Adiciona breakpoint (ex: b 0x0150)        │
│    d <addr>       Remove breakpoint                         │
│    l, list        Lista breakpoints                         │
│    w <addr>       Adiciona watchpoint                       │
├─────────────────────────────────────────────────────────────┤
│  INSPEÇÃO                                                   │
│    reg, r         Mostra registradores                      │
│    p <addr> [n]   Mostra N bytes de memória (default: 16)   │
│    disass [n]     Disassembly de N instruções (default: 5)  │
│    io             Mostra registradores I/O                  │
│    stack [n]      Mostra stack (default: 8 entries)         │
├─────────────────────────────────────────────────────────────┤
│  AJUDA                                                      │
│    h, help, ?     Mostra esta mensagem                      │
└─────────────────────────────────────────────────────────────┘
"
        );
    }
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Parseia endereço em hexadecimal (com ou sem 0x)
pub fn parse_address(s: &str) -> Option<u16> {
    let s = s.trim().to_lowercase();
    let s = s.strip_prefix("0x").unwrap_or(&s);
    u16::from_str_radix(s, 16).ok()
}

/// Nome da instrução em `pc`, com a mesma prioridade da CPU: CB, microcódigo, legado
pub fn instruction_name(cpu: &CPU, pc: u16) -> &'static str {
    let opcode = cpu.bus.read(pc);
    if opcode == 0xCB {
        let cb_opcode = cpu.bus.read(pc.wrapping_add(1));
        return microcode::cb_prefix::lookup(cb_opcode).map_or("UNKNOWN", |p| p.name);
    }
    if instructions::ILLEGAL_OPCODES.contains(&opcode) {
        return "ILLEGAL";
    }
    microcode::lookup(opcode).map_or_else(|| instructions::decode(opcode).name, |p| p.name)
}

/// Retorna o tamanho da instrução em bytes
pub fn get_instruction_length(opcode: u8) -> u8 {
    match opcode {
        0xCB | 0x10 => 2,
        0x01 | 0x11 | 0x21 | 0x31 | 0x08 | 0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA | 0xC4 | 0xCC
        | 0xCD | 0xD4 | 0xDC | 0xEA | 0xFA => 3,
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E | 0xC6 | 0xCE | 0xD6 | 0xDE
        | 0xE6 | 0xEE | 0xF6 | 0xFE | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xE0 | 0xF0 | 0xE8
        | 0xF8 => 2,
        _ => 1,
    }
}

/// Formata os operandos da instrução para exibição
pub fn format_operands(cpu: &CPU, pc: u16, opcode: u8, len: u8) -> String {
    // O segundo byte de CB faz parte do nome; o de STOP é ignorado pela CPU
    if opcode == 0xCB || opcode == 0x10 {
        return String::new();
    }
    match len {
        2 => {
            let byte = cpu.bus.read(pc.wrapping_add(1));
            match opcode {
                0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                    let offset = byte as i8;
                    let target = pc.wrapping_add(2).wrapping_add(offset as u16);
                    format!("${:+} → {:04X}", offset, target)
                }
                0xE0 | 0xF0 => format!("(FF{:02X})", byte),
                _ => format!("${:02X}", byte),
            }
        }
        3 => {
            let lo = cpu.bus.read(pc.wrapping_add(1));
            let hi = cpu.bus.read(pc.wrapping_add(2));
            let addr = ((hi as u16) << 8) | lo as u16;
            format!("${:04X}", addr)
        }
        _ => String::new(),
    }
}
