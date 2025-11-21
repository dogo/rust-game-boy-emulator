use crate::GB::registers;
use crate::GB::RAM;
use crate::GB::instructions;
use crate::GB::instructions::helpers::push_u16;

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: RAM::RAM,
    pub ime: bool,  // Interrupt Master Enable - Quando true habilita e intercepta interrupções
    pub ime_enable_next: bool, // EI habilita IME após a próxima instrução
    pub halted: bool, // CPU está em estado HALT
    pub opcode: u8,  // Opcode da instrução em execução
    pub cycles: u64,  // Contagem total de ciclos
    // Estado mínimo de temporizador/PPU para avançar loops de polling
    ppu_line_cycles: u16, // ciclos acumulados na linha atual (456 ciclos por linha)
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            ram: RAM::RAM::new(),
            ime: false,
            ime_enable_next: false,
            halted: false,
            opcode: 0,
            cycles: 0,
            ppu_line_cycles: 0,
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

        // IO registers pós-boot (valores típicos DMG)
        self.ram.write(0xFF40, 0x91); // LCDC - LCD ligado, BG habilitado (sprites o jogo habilita depois)
        self.ram.write(0xFF42, 0x00); // SCY
        self.ram.write(0xFF43, 0x00); // SCX
        self.ram.write(0xFF44, 0x00); // LY
        self.ram.write(0xFF45, 0x00); // LYC
        self.ram.write(0xFF47, 0xFC); // BGP - paleta background
        self.ram.write(0xFF48, 0xFF); // OBP0 - paleta sprites 0
        self.ram.write(0xFF49, 0xFF); // OBP1 - paleta sprites 1
        self.ram.write(0xFF4A, 0x00); // WY - window Y
        self.ram.write(0xFF4B, 0x00); // WX - window X

        eprintln!("🚀 POST-BOOT STATE 🚀");
        eprintln!("Registers: AF={:04X} BC={:04X} DE={:04X} HL={:04X} SP={:04X} PC={:04X}",
                 self.registers.get_af(), self.registers.get_bc(), self.registers.get_de(),
                 self.registers.get_hl(), self.registers.get_sp(), self.registers.get_pc());
        eprintln!("IO: LCDC={:02X} BGP={:02X} OBP0={:02X} OBP1={:02X}",
                 self.ram.read(0xFF40), self.ram.read(0xFF47), self.ram.read(0xFF48), self.ram.read(0xFF49));
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.ram.load_bytes(data);
        // Inicializa alguns registradores de IO comuns
        self.ram.write(0xFF04, 0); // DIV
        self.ram.write(0xFF0F, 0); // IF
        self.ram.write(0xFFFF, 0); // IE
    }

    pub fn fetch_next(&mut self) -> u8 {
        let pc_before = self.registers.get_pc();

        // Lê o byte na posição do Program Counter
        let byte = self.ram.read(pc_before);

        // Incrementa o PC para apontar para o próximo byte
        self.registers.set_pc(pc_before.wrapping_add(1));
        byte
    }

    pub fn decode(opcode: u8, _cb_opcode: bool) -> instructions::Instruction {
        instructions::decode(opcode)
    }

    pub fn execute_next(&mut self) -> (u64, bool) {
        // Se CPU está em HALT, não executa instruções até uma interrupção acordar
        if self.halted {
            let if_reg = self.ram.read(0xFF0F);
            let ie_reg = self.ram.read(0xFFFF);
            if (if_reg & ie_reg) != 0 {
                self.halted = false;
                // TODO: Implementar HALT bug (IME=0 com interrupção pendente)
                // No hardware real, se IME=0 e há interrupção pendente, PC não incrementa corretamente
            } else {
                // CPU ainda halted, simula 4 ciclos de espera
                self.tick(4);
                return (4, false);
            }
        }

        self.service_interrupts();

        let opcode = self.fetch_next();
        self.opcode = opcode;
        let instr = CPU::decode(opcode, false);
        let unknown = instr.name == "UNKNOWN";
        let cycles = (instr.execute)(&instr, self);
        self.cycles += cycles as u64;

        // EI habilita IME após a próxima instrução
        if self.ime_enable_next {
            self.ime = true;
            self.ime_enable_next = false;
        }

        self.tick(cycles as u32);
        (cycles, unknown)
    }

    // Avança temporizadores, APU e PPU com base nos ciclos consumidos pela instrução
    // PPU = Picture Processing Unit (Unidade de Processamento de Imagem)
    // APU = Audio Processing Unit (Unidade de Processamento de Áudio)
    fn tick(&mut self, cycles: u32) {
        // Timers (DIV/TIMA/TMA/TAC)
        self.ram.tick_timers(cycles);

        // APU (Audio Processing Unit)
        for _ in 0..cycles {
            self.ram.apu.tick();
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

                // Avança para a próxima linha a partir do LY atual do PPU
                let old_ly = self.ram.ppu.ly;
                let mut new_ly = old_ly.wrapping_add(1);

                // Região visível: 0–143
                if old_ly < 144 {
                    // Garante que o PPU sabe em que linha está renderizando
                    self.ram.ppu.ly = old_ly;

                    // Mode 3: Pixel Transfer (durante renderização)
                    self.ram.ppu.update_stat_mode(3);
                    self.ram.ppu.render_bg_scanline();
                    self.ram.ppu.render_window_scanline();
                    self.ram.ppu.render_sprites_scanline(old_ly);
                    // Mode 0: HBlank (após renderização)
                    self.ram.ppu.update_stat_mode(0);
                } else if old_ly >= 144 && old_ly <= 153 {
                    // VBlank lines (144-153)
                    self.ram.ppu.update_stat_mode(1);
                }

                // Início de VBlank
                if new_ly == 144 {
                    let mut iflags = self.ram.read(0xFF0F);
                    iflags |= 0x01; // VBlank

                    // STAT interrupt (Mode 1 + condições STAT)
                    self.ram.ppu.update_stat_mode(1);
                    if self.ram.ppu.check_stat_interrupt() {
                        iflags |= 0x02; // LCD STAT
                    }

                    self.ram.write(0xFF0F, iflags);
                    self.ram.ppu.frame_ready = true;
                }

                // Fim do frame (153 -> 0)
                if new_ly > 153 {
                    new_ly = 0;
                    // Mode 2: OAM Search (início de frame)
                    self.ram.ppu.update_stat_mode(2);
                }

                // Atualiza LY no PPU e no registrador 0xFF44
                self.ram.ppu.ly = new_ly;
                self.ram.write(0xFF44, new_ly);

                // Atualiza flag LYC=LY e possível STAT interrupt
                self.ram.ppu.update_lyc_flag();
                if self.ram.ppu.check_stat_interrupt() {
                    let mut iflags = self.ram.read(0xFF0F);
                    iflags |= 0x02; // LCD STAT
                    self.ram.write(0xFF0F, iflags);
                }
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

        // Push PC usando push_u16 para manter consistência total com CALL/RET
        let pc = self.registers.get_pc();
        push_u16(self, pc);

        self.registers.set_pc(vector);

        // Tempo para atendimento de interrupção (~20 ciclos)
        // NOTE: Custo fixo approximation, independente do vetor
        self.cycles += 20;
        self.tick(20);
    }

    // === API pública de joypad ===
    // Botões D-pad: Right=0, Left=1, Up=2, Down=3
    // Botões ação: A=0, B=1, Select=2, Start=3

    pub fn press_button(&mut self, button: &str) {
        let btn = button.to_uppercase();
        match btn.as_str() {
            // D-pad
            "RIGHT" => self.ram.press_joypad_button(0, true),
            "LEFT"  => self.ram.press_joypad_button(1, true),
            "UP"    => self.ram.press_joypad_button(2, true),
            "DOWN"  => self.ram.press_joypad_button(3, true),
            // Ação
            "A"      => self.ram.press_joypad_button(0, false),
            "B"      => self.ram.press_joypad_button(1, false),
            "SELECT" => self.ram.press_joypad_button(2, false),
            "START"  => self.ram.press_joypad_button(3, false),
            _ => return,
        }
        if self.ram.trace_enabled {
            crate::GB::trace::trace_joypad_press(&btn);
        }

        // Opcional: requisitar interrupção de joypad (bit 4 de IF)
        // let mut iflags = self.ram.read(0xFF0F);
        // iflags |= 0x10;
        // self.ram.write(0xFF0F, iflags);
    }

    pub fn release_button(&mut self, button: &str) {
        let btn = button.to_uppercase();
        match btn.as_str() {
            // D-pad
            "RIGHT" => self.ram.release_joypad_button(0, true),
            "LEFT"  => self.ram.release_joypad_button(1, true),
            "UP"    => self.ram.release_joypad_button(2, true),
            "DOWN"  => self.ram.release_joypad_button(3, true),
            // Ação
            "A"      => self.ram.release_joypad_button(0, false),
            "B"      => self.ram.release_joypad_button(1, false),
            "SELECT" => self.ram.release_joypad_button(2, false),
            "START"  => self.ram.release_joypad_button(3, false),
            _ => return,
        }
        if self.ram.trace_enabled {
            crate::GB::trace::trace_joypad_release(&btn);
        }
    }
}