use crate::GB::instructions;
use crate::GB::microcode;
use crate::GB::registers;

pub struct CPU {
    pub registers: registers::Registers,
    pub bus: crate::GB::bus::MemoryBus,
    pub ime: bool, // Interrupt Master Enable - Quando true habilita e intercepta interrupções
    pub ime_enable_next: bool, // EI habilita IME após a próxima instrução
    pub halted: bool, // CPU está em estado HALT
    pub halt_bug: bool, // HALT bug flag: se true, PC não incrementa após fetch
    pub stopped: bool, // STOP: CPU dormindo até Joypad acordar
    pub opcode: u8, // Opcode da instrução em execução
    pub cycles: u64, // Contagem total de ciclos
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        let mbc = crate::GB::mbc::create_mbc(rom);
        CPU {
            registers: registers::Registers::new(),
            bus: crate::GB::bus::MemoryBus::new(mbc),
            ime: false,
            ime_enable_next: false,
            halted: false,
            halt_bug: false,
            stopped: false,
            opcode: 0,
            cycles: 0,
        }
    }

    // Stack operations
    #[inline]
    pub fn push_u16(&mut self, value: u16) {
        let mut sp = self.registers.get_sp();
        sp = sp.wrapping_sub(1);
        self.bus.cpu_write(sp, (value >> 8) as u8);
        sp = sp.wrapping_sub(1);
        self.bus.cpu_write(sp, (value & 0xFF) as u8);
        self.registers.set_sp(sp);
    }

    #[inline]
    pub fn pop_u16(&mut self) -> u16 {
        let mut sp = self.registers.get_sp();
        let lo = self.bus.cpu_read(sp) as u16;
        sp = sp.wrapping_add(1);
        let hi = self.bus.cpu_read(sp) as u16;
        sp = sp.wrapping_add(1);
        self.registers.set_sp(sp);
        (hi << 8) | lo
    }

    pub fn init_post_boot(&mut self) {
        // Estados típicos pós BIOS (DMG)
        self.registers.set_af(0x01B0);
        self.registers.set_bc(0x0013);
        self.registers.set_de(0x00D8);
        self.registers.set_hl(0x014D);
        self.registers.set_sp(0xFFFE);
        self.registers.set_pc(0x0100);

        // IO registers pós-boot (valores DMG)
        // DIV deve ser setado POR ÚLTIMO pois writes consomem ciclos
        self.bus.write(0xFF05, 0x00); // TIMA
        self.bus.write(0xFF06, 0x00); // TMA
        self.bus.write(0xFF07, 0xF8); // TAC
        self.bus.write(0xFF0F, 0x00); // IF - sem interrupções pendentes
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

        // DIV é setado por último pois os writes acima consomem ciclos
        self.bus.set_div(0xAB);

        eprintln!("🚀 POST-BOOT STATE 🚀");
        eprintln!(
            "Registers: AF={:04X} BC={:04X} DE={:04X} HL={:04X} SP={:04X} PC={:04X}",
            self.registers.get_af(),
            self.registers.get_bc(),
            self.registers.get_de(),
            self.registers.get_hl(),
            self.registers.get_sp(),
            self.registers.get_pc()
        );
        eprintln!(
            "IO: LCDC={:02X} STAT={:02X} DIV={:02X} TIMA={:02X} TMA={:02X} TAC={:02X} BGP={:02X} OBP0={:02X} OBP1={:02X}",
            self.bus.read(0xFF40),
            self.bus.read(0xFF41),
            self.bus.read(0xFF04),
            self.bus.read(0xFF05),
            self.bus.read(0xFF06),
            self.bus.read(0xFF07),
            self.bus.read(0xFF47),
            self.bus.read(0xFF48),
            self.bus.read(0xFF49)
        );
    }

    pub fn fetch_next(&mut self) -> u8 {
        let pc_before = self.registers.get_pc();

        // Lê o byte na posição do Program Counter
        let byte = self.bus.cpu_read(pc_before);

        // HALT bug: se ativo, não incrementa PC após fetch
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.registers.set_pc(pc_before.wrapping_add(1));
        }
        byte
    }

    pub fn decode(opcode: u8, _cb_opcode: bool) -> instructions::Instruction {
        instructions::decode(opcode)
    }

    pub fn execute_next(&mut self) -> (u64, bool) {
        // Se CPU está em STOP, só acorda com Joypad
        if self.stopped {
            if self.bus.joypad_should_wake_from_stop() {
                self.stopped = false;
            } else {
                // Continua “dormindo”: PPU/timer/APU seguem rodando
                self.bus.tick(4);
                return (4, false);
            }
        }
        // Se CPU está em HALT, não executa instruções até uma interrupção acordar
        if self.halted {
            let if_reg = self.bus.read(0xFF0F);
            let ie_reg = self.bus.read(0xFFFF);
            if (if_reg & ie_reg) != 0 {
                // Acorda da HALT normal
                self.halted = false;
            } else {
                // CPU ainda halted, simula 4 ciclos de espera
                self.bus.tick(4);
                return (4, false);
            }
        }

        // EI habilita IME no INÍCIO da próxima instrução (antes do fetch)
        // Ref: https://gbdev.io/pandocs/Interrupts.html
        if self.ime_enable_next {
            self.ime = true;
            self.ime_enable_next = false;
        }

        // FETCH
        self.bus.reset_cpu_cycle_log();
        let opcode = self.fetch_next();
        self.opcode = opcode;

        // DECODE
        let instr = CPU::decode(opcode, false);
        let mut unknown = instr.name == "UNKNOWN";
        let cycles: u64;

        // Trata CB-prefix de forma especial
        if opcode == 0xCB {
            // Busca o segundo byte (opcode CB real)
            let cb_opcode = self.bus.cpu_read(self.registers.get_pc());
            self.registers
                .set_pc(self.registers.get_pc().wrapping_add(1));

            if let Some(program) = microcode::cb_prefix::lookup(cb_opcode) {
                microcode::execute(program, &mut self.registers, &mut self.bus);
                cycles = self.bus.take_cpu_cycle_log() as u64;
                unknown = false;
            } else {
                // Fallback para implementação antiga
                let exec_cycles = (instr.execute)(&instr, &mut self.registers, &mut self.bus);
                let consumed = self.bus.take_cpu_cycle_log();
                if exec_cycles as u32 > consumed {
                    self.bus.tick(exec_cycles as u32 - consumed);
                }
                cycles = exec_cycles;
            }
        } else if let Some(program) = microcode::lookup(opcode) {
            microcode::execute(program, &mut self.registers, &mut self.bus);
            cycles = self.bus.take_cpu_cycle_log() as u64;
            unknown = false;
        } else {
            let exec_cycles = (instr.execute)(&instr, &mut self.registers, &mut self.bus);
            let consumed = self.bus.take_cpu_cycle_log();
            if exec_cycles as u32 > consumed {
                self.bus.tick(exec_cycles as u32 - consumed);
            }
            cycles = exec_cycles;
        }
        self.cycles += cycles;

        // 🔧 EFEITOS ESPECIAIS NO CPU (fora dos registradores)
        match opcode {
            0xF3 => {
                // DI
                self.ime = false;
            }
            0xFB => {
                // EI
                // Habilita IME após a PRÓXIMA instrução
                self.ime_enable_next = true;
            }
            0x76 => {
                // HALT
                let if_reg = self.bus.read(0xFF0F);
                let ie_reg = self.bus.read(0xFFFF);
                let pending = if_reg & ie_reg;

                if !self.ime && pending != 0 {
                    // HALT bug: IME=0 e existe interrupção pendente -> NÃO entra em halt, apenas ativa o bug
                    self.halt_bug = true;
                } else {
                    // HALT normal
                    self.halted = true;
                }
            }
            0x10 => {
                // STOP: para a CPU até Joypad acordar
                self.stopped = true;
            }
            0xD9 => {
                // RETI
                // RET já foi feito na própria instrução (pop PC), aqui só reabilita IME
                self.ime = true;
            }
            _ => {}
        }

        // Atende interrupções se habilitadas (IME) e pendentes (IF & IE)
        self.service_interrupts();

        (cycles, unknown)
    }

    // Atende interrupções se habilitadas (IME) e pendentes (IF & IE)
    fn service_interrupts(&mut self) {
        // 1) Só faz qualquer coisa se IME estiver habilitado
        if !self.ime {
            return;
        }

        let ie = self.bus.get_ie();
        let iflags = self.bus.get_if();
        let pending = ie & iflags;

        // 2) Se não tem pending, sai
        if pending == 0 {
            return;
        }

        // 3) Decide vetor real
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

        // 4) Desabilita IME enquanto atende
        self.ime = false;

        // 5) Limpa bit em IF pela API do bus
        self.bus.clear_if_bits(mask);

        // 6) Push PC e salta pro vetor
        let pc = self.registers.get_pc();
        self.push_u16(pc);
        self.registers.set_pc(vector);

        // 7) Custo da interrupção (~20 ciclos)
        self.cycles += 20;
        self.bus.tick(20);
    }
}
