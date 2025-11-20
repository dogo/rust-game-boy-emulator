use crate::GB::registers;
use crate::GB::RAM;
use crate::GB::instructions;

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: RAM::RAM,
    pub ime: bool,  // Interrupt Master Enable - Quando true habilita e intercepta interrupções
    pub opcode: u8,  // Opcode da instrução em execução
    pub cycles: u64,  // Contagem total de ciclos
    // Estado mínimo de temporizador/PPU para avançar loops de polling
    ppu_line_cycles: u16, // ciclos acumulados na linha atual (456 ciclos por linha)
    ppu_ly: u8,           // linha atual (0..=153), espelhada em 0xFF44
    timer_div_counter: u32, // acumula ciclos para incrementar DIV (a cada 256 ciclos)
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            ram: RAM::RAM::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            ppu_line_cycles: 0,
            ppu_ly: 0,
            timer_div_counter: 0,
        }
    }

    pub fn init_post_boot(&mut self) {
        // Estados típicos pós BIOS (DMG = Dot Matrix Game, Game Boy clássico modelo DMG-01) para permitir pular boot ROM
        self.registers.set_af(0x01B0);
        self.registers.set_bc(0x0013);
        self.registers.set_de(0x00D8);
        self.registers.set_hl(0x014D);
        self.registers.set_sp(0xFFFE);
        self.registers.set_pc(0x0100); // Entrada no cartucho
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.ram.load_bytes(data);
        // Inicializa alguns registradores de IO comuns
        self.ram.write(0xFF44, 0); // LY
        self.ram.write(0xFF04, 0); // DIV
        self.ram.write(0xFF0F, 0); // IF
        self.ram.write(0xFFFF, 0); // IE
    }

    pub fn fetch_next(&mut self) -> u8 {
        // Lê o byte na posição do Program Counter
        let byte = self.ram.read(self.registers.get_pc());
        // Incrementa o PC para apontar para o próximo byte
        self.registers.set_pc(self.registers.get_pc().wrapping_add(1));
        byte
    }

    pub fn decode(opcode: u8, _cb_opcode: bool) -> instructions::Instruction {
        instructions::decode(opcode)
    }

    pub fn execute_next(&mut self) -> (u64, bool) {
        let opcode = self.fetch_next();
        self.opcode = opcode;
        let instr = CPU::decode(opcode, false);
        let unknown = instr.name == "UNKNOWN";
        let cycles = (instr.execute)(&instr, self);
        self.cycles += cycles as u64;
        self.tick(cycles as u32);
        self.service_interrupts();
        (cycles, unknown)
    }

    pub fn run_with_trace(&mut self, max_steps: usize) {
        for step in 0..max_steps {
            let pc = self.registers.get_pc();
            let opcode = self.ram.read(pc); // peek
            let instr = instructions::decode(opcode);
            // Detalhes extras para diagnosticar polling em IO
            let mut extra = String::new();
            match opcode {
                // CB prefix — mostra operação, registrador/bit e valores relevantes
                0xCB => {
                    let cb = self.ram.read(pc.wrapping_add(1));
                    let r_idx = cb & 0x07;
                    let bit_idx = (cb >> 3) & 0x07;
                    let r_name = match r_idx { 0 => "B", 1 => "C", 2 => "D", 3 => "E", 4 => "H", 5 => "L", 6 => "(HL)", _ => "A" };
                    let val: u8 = if r_idx == 6 { self.ram.read(self.registers.get_hl()) } else { match r_idx { 0 => self.registers.get_b(), 1 => self.registers.get_c(), 2 => self.registers.get_d(), 3 => self.registers.get_e(), 4 => self.registers.get_h(), 5 => self.registers.get_l(), _ => self.registers.get_a(), } };

                    let desc = if cb <= 0x07 {
                        "RLC"
                    } else if cb <= 0x0F {
                        "RRC"
                    } else if cb <= 0x17 {
                        "RL"
                    } else if cb <= 0x1F {
                        "RR"
                    } else if cb <= 0x27 {
                        "SLA"
                    } else if cb <= 0x2F {
                        "SRA"
                    } else if cb <= 0x37 {
                        "SWAP"
                    } else if cb <= 0x3F {
                        "SRL"
                    } else if cb <= 0x7F {
                        "BIT"
                    } else if cb <= 0xBF {
                        "RES"
                    } else {
                        "SET"
                    };

                    extra = match desc {
                        "BIT" => {
                            let bit_set = (val & (1u8 << bit_idx)) != 0;
                            format!(" CB={:02X} {} {},{} val={:02X} => Z={}", cb, desc, bit_idx, r_name, val, (!bit_set) as u8)
                        }
                        "RES" | "SET" => {
                            format!(" CB={:02X} {} {},{} before={:02X}", cb, desc, bit_idx, r_name, val)
                        }
                        _ => {
                            // Predict carry flag outcome for shifts/rotates
                            let c_in = if self.registers.get_flag_c() { 1 } else { 0 };
                            let bit7 = (val >> 7) & 1;
                            let bit0 = val & 1;
                            let c_out = match desc {
                                "RLC" | "RL" | "SLA" => bit7,
                                "RRC" | "RR" | "SRA" | "SRL" => bit0,
                                "SWAP" => 0,
                                _ => c_in,
                            };
                            if desc == "SWAP" {
                                format!(" CB={:02X} {} {} val={:02X} C_in={} => C_out=0", cb, desc, r_name, val, c_in)
                            } else {
                                format!(" CB={:02X} {} {} val={:02X} C_in={} => C_out={}", cb, desc, r_name, val, c_in, c_out)
                            }
                        }
                    };
                }
                // LDH A,(n)
                0xF0 => {
                    let n = self.ram.read(pc.wrapping_add(1));
                    let val = self.ram.read(0xFF00u16.wrapping_add(n as u16));
                    extra = format!(" n={:02X} [FF{:02X}]=>{:02X}", n, n, val);
                }
                // LDH (n),A
                0xE0 => {
                    let n = self.ram.read(pc.wrapping_add(1));
                    let a = self.registers.get_a();
                    extra = format!(" n={:02X} [FF{:02X}]<=A({:02X})", n, n, a);
                }
                // LD A,(C)
                0xF2 => {
                    let c = self.registers.get_c();
                    let val = self.ram.read(0xFF00u16.wrapping_add(c as u16));
                    extra = format!(" C={:02X} [FF{:02X}]=>{:02X}", c, c, val);
                }
                // LD (C),A
                0xE2 => {
                    let c = self.registers.get_c();
                    let a = self.registers.get_a();
                    extra = format!(" C={:02X} [FF{:02X}]<=A({:02X})", c, c, a);
                }
                // LD A,(a16)
                0xFA => {
                    let lo = self.ram.read(pc.wrapping_add(1)) as u16;
                    let hi = self.ram.read(pc.wrapping_add(2)) as u16;
                    let addr = (hi << 8) | lo;
                    let val = self.ram.read(addr);
                    extra = format!(" a16={:04X}=>{:02X}", addr, val);
                }
                // LD (a16),A
                0xEA => {
                    let lo = self.ram.read(pc.wrapping_add(1)) as u16;
                    let hi = self.ram.read(pc.wrapping_add(2)) as u16;
                    let addr = (hi << 8) | lo;
                    let a = self.registers.get_a();
                    extra = format!(" a16={:04X}<=A({:02X})", addr, a);
                }
                // CP A,d8 (FE) — mostra comparacao e flags resultantes
                0xFE => {
                    let n = self.ram.read(pc.wrapping_add(1));
                    let a = self.registers.get_a();
                    let z = a == n; // Z set if equal
                    let c = a < n;  // C set if borrow (a < n)
                    let h = (a & 0x0F) < (n & 0x0F); // half-borrow
                    extra = format!(" A={:02X} n={:02X} => Z={} N=1 H={} C={}", a, n, z as u8, h as u8, c as u8);
                }
                // JR cc,r8 — 20,28,30,38: mostra offset, condicao e alvo
                0x20 | 0x28 | 0x30 | 0x38 => {
                    let off = self.ram.read(pc.wrapping_add(1)) as i8;
                    let base = self.registers.get_pc().wrapping_add(2) as i32;
                    let target = (base + off as i32) as u16;
                    let z = self.registers.get_flag_z();
                    let c = self.registers.get_flag_c();
                    let cond = match opcode {
                        0x20 => !z, // NZ
                        0x28 => z,  // Z
                        0x30 => !c, // NC
                        0x38 => c,  // C
                        _ => false,
                    };
                    extra = format!(" r8={:+#04X} cond={} target={:04X}", off, cond as u8, target);
                }
                // JR r8 incondicional — 18
                0x18 => {
                    let off = self.ram.read(pc.wrapping_add(1)) as i8;
                    let base = self.registers.get_pc().wrapping_add(2) as i32;
                    let target = (base + off as i32) as u16;
                    extra = format!(" r8={:+#04X} target={:04X}", off, target);
                }
                _ => {}
            }
            if extra.is_empty() {
                println!("{:05} PC={:04X} OP={:02X} {}", step, pc, opcode, instr.name);
            } else {
                println!("{:05} PC={:04X} OP={:02X} {}{}", step, pc, opcode, instr.name, extra);
            }
            let (_cycles, unknown) = self.execute_next();
            if unknown { println!("Parando: opcode desconhecido {:02X} em {:04X}", opcode, pc); break; }
        }
        println!("Total cycles: {}", self.cycles);
    }

    // Avança temporizadores e PPU com base nos ciclos consumidos pela instrução
    // PPU = Picture Processing Unit (Unidade de Processamento de Imagem)
    fn tick(&mut self, cycles: u32) {
        // Timer DIV (0xFF04) incrementa a cada 256 ciclos de CPU
        self.timer_div_counter += cycles;
        while self.timer_div_counter >= 256 {
            self.timer_div_counter -= 256;
            let div = self.ram.read(0xFF04).wrapping_add(1);
            self.ram.write(0xFF04, div);
        }

        // PPU: 456 ciclos por linha, 154 linhas por frame (0..=153)
        let mut add = cycles as u16;
        while add > 0 {
            let space = 456u16.saturating_sub(self.ppu_line_cycles);
            let step = add.min(space);
            self.ppu_line_cycles = self.ppu_line_cycles.saturating_add(step);
            add -= step;
            if self.ppu_line_cycles >= 456 {
                self.ppu_line_cycles = 0;
                // próxima linha
                self.ppu_ly = self.ppu_ly.wrapping_add(1);
                if self.ppu_ly == 144 {
                    // Início de VBlank: seta IF bit 0 (VBlank)
                    let mut iflags = self.ram.read(0xFF0F);
                    iflags |= 0x01;
                    self.ram.write(0xFF0F, iflags);
                }
                if self.ppu_ly > 153 { self.ppu_ly = 0; }
                // Espelha LY em 0xFF44
                self.ram.write(0xFF44, self.ppu_ly);
            }
        }
    }

    // Atende interrupções se habilitadas (IME) e pendentes (IF & IE)
    fn service_interrupts(&mut self) {
        if !self.ime { return; }
        let ie = self.ram.read(0xFFFF);
        let mut iflags = self.ram.read(0xFF0F);
        let pending = ie & iflags;
        if pending == 0 { return; }

        // Desabilita IME e atende na ordem de prioridade
        self.ime = false;
        let (vector, mask) = if (pending & 0x01) != 0 {
            (0x0040u16, 0x01u8) // VBlank
        } else if (pending & 0x02) != 0 {
            (0x0048u16, 0x02u8) // LCD STAT
        } else if (pending & 0x04) != 0 {
            (0x0050u16, 0x04u8) // Timer
        } else if (pending & 0x08) != 0 {
            (0x0058u16, 0x08u8) // Serial
        } else {
            (0x0060u16, 0x10u8) // Joypad
        };

        // Limpa o bit atendido em IF
        iflags &= !mask;
        self.ram.write(0xFF0F, iflags);

        // Push PC e jump para vetor
        let pc = self.registers.get_pc();
        // push u16 na pilha
        let mut sp = self.registers.get_sp();
        sp = sp.wrapping_sub(1);
        self.ram.write(sp, (pc >> 8) as u8);
        sp = sp.wrapping_sub(1);
        self.ram.write(sp, (pc & 0xFF) as u8);
        self.registers.set_sp(sp);
        self.registers.set_pc(vector);

        // Tempo para atendimento de interrupção (~20 ciclos)
        self.cycles += 20;
        self.tick(20);
    }
}