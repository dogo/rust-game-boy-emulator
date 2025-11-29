/// Debugger interativo para o emulador Game Boy
/// Baseado em: https://aquova.net/emudev/gb/23-debugger.html

use crate::GB::CPU::CPU;
use crate::GB::instructions;
use std::io::{self, Write};

pub struct Debugger {
    debugging: bool,
    breakpoints: Vec<u16>,
    watchpoints: Vec<u16>, // Endereços para watch (read/write)
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            debugging: false,
            breakpoints: Vec::new(),
            watchpoints: Vec::new(),
        }
    }

    /// Verifica se estamos em modo debug
    pub fn is_debugging(&self) -> bool {
        self.debugging
    }

    /// Ativa/desativa modo debug
    pub fn set_debugging(&mut self, debug: bool) {
        self.debugging = debug;
    }

    /// Verifica se PC está em um breakpoint
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
            if io::stdin().read_line(&mut input).is_err() {
                println!("Erro ao ler input");
                continue;
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
                "q" | "quit" => {
                    return true;
                }
                "c" | "continue" => {
                    self.debugging = false;
                    return false;
                }
                "n" | "next" | "s" | "step" => {
                    let (cycles, _) = cpu.execute_next();
                    self.print_current_state(cpu);
                    println!("  ({} cycles)", cycles);
                }
                "b" | "break" => {
                    if words.len() < 2 {
                        println!("Uso: b <endereço>");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        self.add_breakpoint(addr);
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
                        self.remove_breakpoint(addr);
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "l" | "list" => {
                    self.print_breakpoints();
                }
                "reg" | "r" => {
                    self.print_registers(cpu);
                }
                "p" | "print" | "x" => {
                    if words.len() < 2 {
                        println!("Uso: p <endereço> [quantidade]");
                        continue;
                    }
                    if let Some(addr) = parse_address(words[1]) {
                        let count = if words.len() > 2 {
                            words[2].parse().unwrap_or(16)
                        } else {
                            16
                        };
                        self.print_memory(cpu, addr, count);
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "disass" | "dis" => {
                    let count = if words.len() > 1 {
                        words[1].parse().unwrap_or(5)
                    } else {
                        5
                    };
                    self.disassemble(cpu, count);
                }
                "io" => {
                    self.print_io_registers(cpu);
                }
                "stack" => {
                    let count = if words.len() > 1 {
                        words[1].parse().unwrap_or(8)
                    } else {
                        8
                    };
                    self.print_stack(cpu, count);
                }
                "run" => {
                    if words.len() < 2 {
                        println!("Uso: run <número de instruções>");
                        continue;
                    }
                    if let Ok(n) = words[1].parse::<usize>() {
                        self.run_n_instructions(cpu, n);
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
                        self.add_watchpoint(addr);
                    } else {
                        println!("Endereço inválido: {}", words[1]);
                    }
                }
                "h" | "help" | "?" => {
                    self.print_help();
                }
                _ => {
                    println!("Comando desconhecido: '{}'. Digite 'h' para ajuda.", words[0]);
                }
            }
        }
    }

    fn add_breakpoint(&mut self, addr: u16) {
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
            println!("✅ Breakpoint adicionado em 0x{:04X}", addr);
        } else {
            println!("⚠️  Breakpoint já existe em 0x{:04X}", addr);
        }
    }

    fn remove_breakpoint(&mut self, addr: u16) {
        if let Some(pos) = self.breakpoints.iter().position(|&x| x == addr) {
            self.breakpoints.remove(pos);
            println!("🗑️  Breakpoint removido de 0x{:04X}", addr);
        } else {
            println!("⚠️  Nenhum breakpoint em 0x{:04X}", addr);
        }
    }

    fn add_watchpoint(&mut self, addr: u16) {
        if !self.watchpoints.contains(&addr) {
            self.watchpoints.push(addr);
            println!("👁️  Watchpoint adicionado em 0x{:04X}", addr);
        } else {
            println!("⚠️  Watchpoint já existe em 0x{:04X}", addr);
        }
    }

    fn print_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("Nenhum breakpoint definido");
        } else {
            println!("Breakpoints:");
            for (i, &addr) in self.breakpoints.iter().enumerate() {
                println!("  {}: 0x{:04X}", i, addr);
            }
        }
        if !self.watchpoints.is_empty() {
            println!("Watchpoints:");
            for (i, &addr) in self.watchpoints.iter().enumerate() {
                println!("  {}: 0x{:04X}", i, addr);
            }
        }
    }

    fn print_registers(&self, cpu: &CPU) {
        let regs = &cpu.registers;
        println!("┌─────────────────────────────────────┐");
        println!("│           REGISTRADORES             │");
        println!("├─────────────────────────────────────┤");
        println!("│  AF: {:04X}    BC: {:04X}             │", regs.get_af(), regs.get_bc());
        println!("│  DE: {:04X}    HL: {:04X}             │", regs.get_de(), regs.get_hl());
        println!("│  SP: {:04X}    PC: {:04X}             │", regs.get_sp(), regs.get_pc());
        println!("├─────────────────────────────────────┤");
        println!("│  Flags: Z={} N={} H={} C={}            │",
            regs.get_flag_z() as u8,
            regs.get_flag_n() as u8,
            regs.get_flag_h() as u8,
            regs.get_flag_c() as u8
        );
        println!("├─────────────────────────────────────┤");
        println!("│  IME: {}  HALT: {}  STOP: {}           │",
            cpu.ime as u8,
            cpu.halted as u8,
            cpu.stopped as u8
        );
        println!("│  Cycles: {:>10}                 │", cpu.cycles);
        println!("└─────────────────────────────────────┘");
    }

    fn print_memory(&self, cpu: &CPU, addr: u16, count: usize) {
        println!("Memória a partir de 0x{:04X}:", addr);
        let mut current = addr;
        for row in 0..((count + 15) / 16) {
            print!("  {:04X}: ", current);
            let mut ascii = String::new();
            for col in 0..16 {
                if row * 16 + col < count {
                    let byte = cpu.bus.read(current);
                    print!("{:02X} ", byte);
                    ascii.push(if byte >= 0x20 && byte < 0x7F {
                        byte as char
                    } else {
                        '.'
                    });
                    current = current.wrapping_add(1);
                } else {
                    print!("   ");
                }
            }
            println!(" |{}|", ascii);
        }
    }

    fn print_io_registers(&self, cpu: &CPU) {
        println!("┌─────────────────────────────────────┐");
        println!("│         REGISTRADORES I/O           │");
        println!("├─────────────────────────────────────┤");
        println!("│  P1/JOYP: {:02X}    DIV:  {:02X}           │", cpu.bus.read(0xFF00), cpu.bus.read(0xFF04));
        println!("│  TIMA:    {:02X}    TMA:  {:02X}           │", cpu.bus.read(0xFF05), cpu.bus.read(0xFF06));
        println!("│  TAC:     {:02X}    IF:   {:02X}           │", cpu.bus.read(0xFF07), cpu.bus.read(0xFF0F));
        println!("├─────────────────────────────────────┤");
        println!("│  LCDC: {:02X}  STAT: {:02X}  LY: {:02X}        │", cpu.bus.read(0xFF40), cpu.bus.read(0xFF41), cpu.bus.read(0xFF44));
        println!("│  SCY:  {:02X}  SCX:  {:02X}  LYC: {:02X}       │", cpu.bus.read(0xFF42), cpu.bus.read(0xFF43), cpu.bus.read(0xFF45));
        println!("│  WY:   {:02X}  WX:   {:02X}  BGP: {:02X}       │", cpu.bus.read(0xFF4A), cpu.bus.read(0xFF4B), cpu.bus.read(0xFF47));
        println!("├─────────────────────────────────────┤");
        println!("│  IE: {:02X}                             │", cpu.bus.read(0xFFFF));
        println!("└─────────────────────────────────────┘");
    }

    fn print_stack(&self, cpu: &CPU, count: usize) {
        let sp = cpu.registers.get_sp();
        println!("Stack (SP=0x{:04X}):", sp);
        for i in 0..count {
            let addr = sp.wrapping_add(i as u16 * 2);
            let lo = cpu.bus.read(addr);
            let hi = cpu.bus.read(addr.wrapping_add(1));
            let val = ((hi as u16) << 8) | lo as u16;
            println!("  {:04X}: {:04X}", addr, val);
        }
    }

    fn disassemble(&self, cpu: &CPU, count: usize) {
        let mut pc = cpu.registers.get_pc();
        println!("Disassembly:");
        for _ in 0..count {
            let opcode = cpu.bus.read(pc);
            let instr = instructions::decode(opcode);

            // Determina tamanho da instrução
            let len = get_instruction_length(opcode, cpu.bus.read(pc.wrapping_add(1)));

            // Mostra bytes da instrução
            let mut bytes = format!("{:02X}", opcode);
            for i in 1..len {
                bytes.push_str(&format!(" {:02X}", cpu.bus.read(pc.wrapping_add(i as u16))));
            }

            // Formata operandos
            let operands = format_operands(cpu, pc, opcode, len);

            let marker = if pc == cpu.registers.get_pc() { "→" } else { " " };
            println!("{} {:04X}:  {:<12} {:<8} {}", marker, pc, bytes, instr.name, operands);

            pc = pc.wrapping_add(len as u16);
        }
    }

    fn print_current_state(&self, cpu: &CPU) {
        let pc = cpu.registers.get_pc();
        let opcode = cpu.bus.read(pc);
        let instr = instructions::decode(opcode);

        let len = get_instruction_length(opcode, cpu.bus.read(pc.wrapping_add(1)));
        let mut bytes = format!("{:02X}", opcode);
        for i in 1..len {
            bytes.push_str(&format!(" {:02X}", cpu.bus.read(pc.wrapping_add(i as u16))));
        }

        let operands = format_operands(cpu, pc, opcode, len);

        println!("→ {:04X}: {:<12} {:<8} {} | AF={:04X} BC={:04X} DE={:04X} HL={:04X}",
            pc, bytes, instr.name, operands,
            cpu.registers.get_af(),
            cpu.registers.get_bc(),
            cpu.registers.get_de(),
            cpu.registers.get_hl()
        );
    }

    fn run_n_instructions(&mut self, cpu: &mut CPU, n: usize) {
        for i in 0..n {
            let pc = cpu.registers.get_pc();

            // Verifica breakpoints
            if i > 0 && self.breakpoints.contains(&pc) {
                println!("🔴 Breakpoint hit at 0x{:04X} após {} instruções", pc, i);
                return;
            }

            let (cycles, unknown) = cpu.execute_next();
            if unknown {
                println!("⚠️  Opcode desconhecido em 0x{:04X}", pc);
                return;
            }

            if i < 10 || i == n - 1 {
                self.print_current_state(cpu);
                println!("  ({} cycles)", cycles);
            } else if i == 10 {
                println!("  ... ({} instruções restantes)", n - 10);
            }
        }
        println!("✅ Executadas {} instruções", n);
    }

    fn print_help(&self) {
        println!("
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
");
    }

    /// Imprime mensagem de boas-vindas
    pub fn print_info(&self) {
        println!("\n🎮 GBD - Game Boy Debugger");
        println!("Digite 'h' para ajuda\n");
    }
}

/// Parseia endereço em hexadecimal (com ou sem 0x)
fn parse_address(s: &str) -> Option<u16> {
    let s = s.trim().to_lowercase();
    let s = s.strip_prefix("0x").unwrap_or(&s);
    u16::from_str_radix(s, 16).ok()
}

/// Retorna o tamanho da instrução em bytes
fn get_instruction_length(opcode: u8, _cb_opcode: u8) -> u8 {
    match opcode {
        // CB prefix
        0xCB => 2,
        // Instruções de 3 bytes (16-bit immediate ou address)
        0x01 | 0x11 | 0x21 | 0x31 | // LD rr,d16
        0x08 | // LD (a16),SP
        0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA | // JP
        0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC | // CALL
        0xEA | 0xFA => 3, // LD (a16),A / LD A,(a16)
        // Instruções de 2 bytes (8-bit immediate ou relative)
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E | // LD r,d8
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE | // ALU A,d8
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 | // JR
        0xE0 | 0xF0 | // LDH
        0xE8 | 0xF8 => 2, // ADD SP,r8 / LD HL,SP+r8
        // Todas as outras são 1 byte
        _ => 1,
    }
}

/// Formata os operandos da instrução para exibição
fn format_operands(cpu: &CPU, pc: u16, opcode: u8, len: u8) -> String {
    match len {
        2 => {
            let byte = cpu.bus.read(pc.wrapping_add(1));
            match opcode {
                // JR (signed offset)
                0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                    let offset = byte as i8;
                    let target = pc.wrapping_add(2).wrapping_add(offset as u16);
                    format!("${:+} → {:04X}", offset, target)
                }
                // LDH
                0xE0 | 0xF0 => format!("(FF{:02X})", byte),
                // Immediate
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
