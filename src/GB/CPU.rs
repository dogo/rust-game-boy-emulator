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
            println!("{:05} PC={:04X} OP={:02X} {}", step, pc, opcode, instr.name);
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