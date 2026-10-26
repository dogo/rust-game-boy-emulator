use crate::GB::microcode::{self, CpuControl};
pub use crate::GB::model::HardwareModel as BootModel;
use crate::GB::registers;

pub struct CPU {
    pub registers: registers::Registers,
    pub bus: crate::GB::bus::MemoryBus,
    pub ime: bool, // Interrupt Master Enable - Quando true habilita e intercepta interrupções
    pub ime_enable_next: bool, // EI habilita IME após a próxima instrução
    pub halted: bool, // CPU está em estado HALT
    pub halt_bug: bool, // HALT bug flag: se true, PC não incrementa após fetch
    pub stopped: bool, // STOP: CPU dormindo até Joypad acordar
    pub locked: bool, // Opcode ilegal: CPU travada até desligar o console
    pub opcode: u8, // Opcode da instrução em execução
    pub cycles: u64, // Contagem total de ciclos
}

impl CPU {
    /// Mantém a escolha padrão anterior para chamadores que não selecionam hardware.
    pub fn new(rom: Vec<u8>) -> Self {
        let model = if crate::GB::cartridge::is_cgb_only_rom(&rom) {
            BootModel::Cgb
        } else {
            BootModel::DmgAbc
        };
        Self::new_with_model(rom, model)
    }

    pub fn new_with_model(rom: Vec<u8>, model: BootModel) -> Self {
        let mbc = crate::GB::mbc::create_mbc(rom);
        CPU {
            registers: registers::Registers::new(),
            bus: crate::GB::bus::MemoryBus::new_with_model(mbc, model),
            ime: false,
            ime_enable_next: false,
            halted: false,
            halt_bug: false,
            stopped: false,
            locked: false,
            opcode: 0,
            cycles: 0,
        }
    }

    pub fn init_post_boot(&mut self) {
        self.init_post_boot_model(self.bus.model);
    }

    pub fn init_post_boot_model(&mut self, model: BootModel) {
        if self.bus.model != model {
            self.bus.set_model(model);
        }
        if model.is_cgb() {
            // A boot ROM CGB limpa os dois bancos de VRAM antes de entregar o cartucho.
            self.bus.ppu.vram.fill(0);
            self.bus.ppu.vram_bank1.fill(0);
        }
        match model {
            BootModel::Dmg0 => {
                self.registers.set_af(0x0100);
                self.registers.set_bc(0xFF13);
                self.registers.set_de(0x00C1);
                self.registers.set_hl(0x8403);
            }
            BootModel::DmgAbc => {
                self.registers.set_af(0x01B0);
                self.registers.set_bc(0x0013);
                self.registers.set_de(0x00D8);
                self.registers.set_hl(0x014D);
            }
            BootModel::Mgb => {
                self.registers.set_af(0xFFB0);
                self.registers.set_bc(0x0013);
                self.registers.set_de(0x00D8);
                self.registers.set_hl(0x014D);
            }
            BootModel::Sgb => {
                self.registers.set_af(0x0100);
                self.registers.set_bc(0x0014);
                self.registers.set_de(0x0000);
                self.registers.set_hl(0xC060);
            }
            BootModel::Sgb2 => {
                self.registers.set_af(0xFF00);
                self.registers.set_bc(0x0014);
                self.registers.set_de(0x0000);
                self.registers.set_hl(0xC060);
            }
            BootModel::Cgb | BootModel::Cgb0 => {
                self.registers.set_af(0x1180);
                self.registers.set_bc(0x0000);
                self.registers.set_de(0x0008);
                self.registers.set_hl(0x007C);
            }
            BootModel::Agb => {
                self.registers.set_af(0x1100);
                self.registers.set_bc(0x0100);
                self.registers.set_de(0x0008);
                self.registers.set_hl(0x007C);
            }
        }
        self.registers.set_sp(0xFFFE);
        self.registers.set_pc(0x0100);

        // IO registers pós-boot (valores DMG)
        // DIV deve ser setado POR ÚLTIMO pois writes consomem ciclos
        self.bus.set_cgb_compat_hwio(false);
        self.bus.write(0xFF00, 0x00); // P1
        self.bus.write(0xFF05, 0x00); // TIMA
        self.bus.write(0xFF06, 0x00); // TMA
        self.bus.write(0xFF07, 0xF8); // TAC
        self.bus.write(0xFF0F, 0x01); // IF - VBlank pendente após boot DMG
        self.bus.write(0xFFFF, 0x00); // IE
        self.bus.write(0xFF10, 0x80); // NR10
        self.bus.write(0xFF11, 0xBF); // NR11
        self.bus.write(0xFF12, 0xF3); // NR12
        self.bus.write(0xFF13, 0xFF); // NR13
        self.bus.write(0xFF14, 0xBF); // NR14
        self.bus.write(0xFF16, 0x3F); // NR21
        self.bus.write(0xFF17, 0x00); // NR22
        self.bus.write(0xFF18, 0xFF); // NR23
        self.bus.write(0xFF19, 0xBF); // NR24
        self.bus.write(0xFF1A, 0x7F); // NR30
        self.bus.write(0xFF1B, 0xFF); // NR31
        self.bus.write(0xFF1C, 0x9F); // NR32
        self.bus.write(0xFF1D, 0xFF); // NR33
        self.bus.write(0xFF1E, 0xBF); // NR34
        self.bus.write(0xFF20, 0xFF); // NR41
        self.bus.write(0xFF21, 0x00); // NR42
        self.bus.write(0xFF22, 0x00); // NR43
        self.bus.write(0xFF23, 0xBF); // NR44
        self.bus.write(0xFF24, 0x77); // NR50
        self.bus.write(0xFF25, 0xF3); // NR51
        self.bus.write(0xFF26, 0xF1); // NR52
        for i in 0xFF30..=0xFF3F {
            self.bus.write(i, 0x00);
        } // Wave RAM
        self.bus.write(0xFF40, 0x91); // LCDC
        self.bus.write(0xFF41, 0x85); // STAT
        self.bus.write(0xFF42, 0x00); // SCY
        self.bus.write(0xFF43, 0x00); // SCX
        self.bus.write(0xFF44, 0x00); // LY
        self.bus.write(0xFF45, 0x00); // LYC
        // NÃO escreve 0xFF46 (DMA) - isso iniciaria uma transferência DMA!
        // O registrador DMA não deve ser inicializado com valor que cause DMA ativo
        self.bus.write(0xFF47, 0xFC); // BGP
        self.bus.write(0xFF48, 0xFF); // OBP0
        self.bus.write(0xFF49, 0xFF); // OBP1
        self.bus.write(0xFF4A, 0x00); // WY
        self.bus.write(0xFF4B, 0x00); // WX

        // Snapshot observado ao sair da boot ROM DMG/MGB.
        self.bus.ppu.mode = 0;
        self.bus.ppu.mode_clock = 0;
        self.bus.ppu.ly = 0x00;
        self.bus.ppu.stat = 0x00;

        match model {
            BootModel::Dmg0 => {
                self.bus.write(0xFF00, 0x00);
                self.bus.ppu.stat = 0x83;
                self.bus.ppu.ly = 0x91;
                self.bus.ppu.mode = 0x03;
                self.bus.ppu.mode_clock = 200;
                self.bus.set_div_counter(0x1830);
            }
            BootModel::Sgb | BootModel::Sgb2 => {
                self.bus.write(0xFF00, 0x30);
                // No SGB, NR52 inicia com master ligado e nenhum canal ativo (0xF0).
                // Faz um power-cycle do APU e restaura o snapshot esperado de registradores.
                self.bus.write(0xFF26, 0x00);
                self.bus.write(0xFF26, 0x80);
                self.bus.write(0xFF10, 0x80);
                self.bus.write(0xFF11, 0xBF);
                self.bus.write(0xFF12, 0xF3);
                self.bus.write(0xFF16, 0x3F);
                self.bus.write(0xFF17, 0x00);
                self.bus.write(0xFF1A, 0x7F);
                self.bus.write(0xFF1C, 0x9F);
                self.bus.write(0xFF21, 0x00);
                self.bus.write(0xFF22, 0x00);
                self.bus.write(0xFF24, 0x77);
                self.bus.write(0xFF25, 0xF3);
                self.bus.ppu.stat = 0x00;
                self.bus.ppu.ly = 0x00;
                let div_counter = if model == BootModel::Sgb2 {
                    0xD850
                } else {
                    0xD860
                };
                self.bus.set_div_counter(div_counter);
            }
            BootModel::DmgAbc | BootModel::Mgb => {
                // Boot ROM do DMG ABC/MGB termina com div_counter = 0xABCC.
                self.bus.set_div_counter(0xABCC);
            }
            BootModel::Cgb => {
                self.bus.set_cgb_compat_hwio(true);
                self.bus.write(0xFF00, 0x30);
                // CGB-ABCDE: div_counter = 0x2678
                self.bus.set_div_counter(0x2678);
            }
            BootModel::Cgb0 => {
                self.bus.set_cgb_compat_hwio(true);
                self.bus.write(0xFF00, 0x30);
                // CGB-0 (first revision): div_counter = 0x2884
                self.bus.set_div_counter(0x2884);
            }
            BootModel::Agb => {
                self.bus.set_cgb_compat_hwio(true);
                self.bus.write(0xFF00, 0x30);
                // AGB/AGS: div_counter = 0x267C
                self.bus.set_div_counter(0x267C);
            }
        }
    }

    pub fn fetch_next(&mut self) -> u8 {
        let pc_before = self.registers.get_pc();

        // Lê o byte na posição do Program Counter
        let byte = self.bus.cpu_read(pc_before);

        // HALT bug: se ativo, não incrementa PC após fetch (apenas uma vez)
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.registers.set_pc(pc_before.wrapping_add(1));
        }
        byte
    }

    /// Executa uma instrução (ou um passo de espera) e retorna os T-cycles consumidos
    pub fn execute_next(&mut self) -> u64 {
        // Nem interrupções destravam; PPU e timer seguem rodando
        if self.locked {
            self.bus.tick(4);
            return 4;
        }
        // Em STOP o oscilador para: nada avança até uma linha do joypad ir a zero
        if self.stopped {
            if !self.bus.joypad_line_low() {
                return 4;
            }
            self.stopped = false;
        }
        // Se CPU está em HALT, não executa instruções até uma interrupção acordar
        if self.halted {
            let if_reg = self.bus.read(0xFF0F);
            let ie_reg = self.bus.read(0xFFFF);

            if (if_reg & ie_reg & 0x1F) != 0 {
                // Acorda da HALT normal (somente bits 0-4 são interrupções reais)
                self.halted = false;
            } else {
                // CPU ainda halted, simula 4 ciclos de espera
                self.bus.tick(4);
                return 4;
            }
        }

        let mut enable_ime_after_instruction = self.ime_enable_next;
        self.ime_enable_next = false;

        if self.service_interrupts_with_ime(self.ime) {
            return 20;
        }

        // FETCH
        self.bus.reset_cpu_cycle_log();
        let opcode = self.fetch_next();
        self.opcode = opcode;

        // EXECUTE: toda instrução tem microprograma; IME, HALT, STOP e travamento
        // voltam como efeito de controle
        let program = if opcode == 0xCB {
            let cb_opcode = self.bus.cpu_read(self.registers.get_pc());
            self.registers
                .set_pc(self.registers.get_pc().wrapping_add(1));
            microcode::cb_prefix::lookup(cb_opcode)
        } else {
            microcode::lookup(opcode)
        }
        .expect("o microcódigo cobre todos os opcodes");
        let control = microcode::execute(program, &mut self.registers, &mut self.bus);
        let cycles = self.bus.take_cpu_cycle_log() as u64;
        self.cycles += cycles;

        match control {
            Some(CpuControl::DisableInterrupts) => {
                self.ime = false;
                self.ime_enable_next = false;
                enable_ime_after_instruction = false;
            }
            Some(CpuControl::EnableInterrupts) => {
                if !enable_ime_after_instruction {
                    self.ime_enable_next = true;
                }
            }
            Some(CpuControl::EnableInterruptsNow) => self.ime = true,
            Some(CpuControl::Halt) => {
                // Somente bits 0-4 são interrupções reais; bits 5-7 são ignorados
                let pending = self.bus.get_if() & self.bus.get_ie() & 0x1F;
                if !self.ime && pending != 0 {
                    // HALT bug: PC já foi incrementado pelo fetch, então a próxima
                    // instrução será executada duas vezes (uma com PC não incrementando)
                    self.halt_bug = true;
                } else {
                    self.halted = true;
                }
            }
            Some(CpuControl::Stop) => {
                // Em modo CGB com KEY1 bit 0 setado, STOP troca a velocidade da CPU
                if self.bus.is_cgb_mode() && (self.bus.key1 & 0x01) != 0 {
                    self.bus.cgb_speed = !self.bus.cgb_speed;
                    self.bus.key1 = 0; // limpa solicitação de troca
                } else {
                    self.enter_stop();
                }
            }
            Some(CpuControl::Lock) => self.locked = true,
            None => {}
        }
        if enable_ime_after_instruction {
            self.ime = true;
        }

        cycles
    }

    /// Fluxograma de STOP do Pan Docs (DMG, sem troca de velocidade): com botão
    /// pressionado vira HALT ou NOP; o byte seguinte só é pulado sem interrupção pendente.
    fn enter_stop(&mut self) {
        let interrupt_pending = self.bus.get_ie() & self.bus.get_if() & 0x1F != 0;
        if !interrupt_pending {
            self.registers
                .set_pc(self.registers.get_pc().wrapping_add(1));
        }
        if self.bus.joypad_line_low() {
            self.halted = !interrupt_pending;
        } else {
            self.bus.write(0xFF04, 0);
            self.stopped = true;
        }
    }

    fn service_interrupts_with_ime(&mut self, effective_ime: bool) -> bool {
        let ie = self.bus.get_ie();
        let iflags = self.bus.get_if();

        let pending = ie & iflags & 0x1F;

        if !effective_ime {
            return false;
        }

        if pending == 0 {
            return false;
        }

        let old_pc = self.registers.get_pc();

        // ISR leva exatamente 20 T-cycles (5 M-cycles):
        // - 2 M-cycles internos (8 T)
        // - Push PC alto (4 T via cpu_write)
        // - Push PC baixo (4 T via cpu_write)
        // - Salto para o vetor (4 T)
        self.bus.tick(8); // 2 M-cycles internos

        let mut sp = self.registers.get_sp().wrapping_sub(1);
        self.registers.set_sp(sp);
        self.bus.cpu_write(sp, (old_pc >> 8) as u8);

        let pending_after_high_push = self.bus.get_ie() & self.bus.get_if() & 0x1F;
        let Some((vector, mask)) = Self::select_interrupt(pending_after_high_push) else {
            self.bus.tick(8);
            self.registers.set_pc(0x0000);
            self.ime = false;
            self.cycles += 20;
            return true;
        };

        sp = self.registers.get_sp().wrapping_sub(1);
        self.registers.set_sp(sp);
        self.bus.cpu_write(sp, (old_pc & 0xFF) as u8);

        self.bus.tick(4); // 1 M-cycle para carregar o vetor no PC
        self.registers.set_pc(vector);

        self.bus.clear_if_bits(mask);
        self.ime = false;

        self.cycles += 20;

        true
    }

    fn select_interrupt(pending: u8) -> Option<(u16, u8)> {
        if (pending & 0x01) != 0 {
            Some((0x0040, 0x01))
        } else if (pending & 0x02) != 0 {
            Some((0x0048, 0x02))
        } else if (pending & 0x04) != 0 {
            Some((0x0050, 0x04))
        } else if (pending & 0x08) != 0 {
            Some((0x0058, 0x08))
        } else if (pending & 0x10) != 0 {
            Some((0x0060, 0x10))
        } else {
            None
        }
    }
}
