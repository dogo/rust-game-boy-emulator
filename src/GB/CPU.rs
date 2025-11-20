use crate::GB::registers;
use crate::GB::RAM;
use crate::GB::instructions;

pub struct CPU {
    pub registers: registers::Registers,
    pub ram: RAM::RAM,
    pub ime: bool,  // Interrupt Master Enable - Quando true habilita e intercepta interrupções
    pub opcode: u8,  // Opcode da instrução em execução
    pub cycles: u64,  // Contagem total de ciclos
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            ram: RAM::RAM::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
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
}