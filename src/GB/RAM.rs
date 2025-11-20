pub struct RAM {
    // todas as 65.536 posições endereçáveis
    memory: [u8; 65536],
    // Estado interno do temporizador
    div_counter: u16,        // contador divisor interno de 16 bits (incrementa a cada ciclo da CPU)
    timer_last_signal: bool, // último nível do sinal do timer (enable && bit selecionado de div_counter)
    tima_reload_delay: u8,   // se >0, contagem regressiva para recarregar TIMA com TMA e solicitar interrupção
}

impl RAM {
    pub fn new() -> Self {
        RAM { memory: [0; 65536], div_counter: 0, timer_last_signal: false, tima_reload_delay: 0 }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0xFF04 => { // escrita em DIV: zera o divisor interno e o registrador DIV
                self.div_counter = 0;
                self.memory[address as usize] = 0;
                // Recalcula o último sinal do timer após o reset para evitar bordas espúrias
                self.timer_last_signal = self.current_timer_signal();
                println!("[TIMER] DIV<=00 (reset)");
            }
            0xFF07 => { // TAC
                self.memory[address as usize] = byte & 0x07; // apenas os 3 bits menos significativos são usados
                // Atualiza o último sinal conforme a nova configuração do TAC
                self.timer_last_signal = self.current_timer_signal();
                let en = (byte & 0x04) != 0;
                let freq = match byte & 0x03 { 0b00 => 4096, 0b01 => 262144, 0b10 => 65536, _ => 16384 };
                println!("[TIMER] TAC<={:02X} (enable={}, freq={}Hz)", byte & 0x07, en as u8, freq);
            }
            0xFF05 | 0xFF06 => { // TIMA, TMA
                self.memory[address as usize] = byte;
                if address == 0xFF05 {
                    println!("[TIMER] TIMA<={:02X}", byte);
                } else {
                    println!("[TIMER] TMA<={:02X}", byte);
                }
            }
            _ => {
                self.memory[address as usize] = byte;
            }
        }
    }

    pub fn load_bytes(&mut self, data: &[u8]) {
        let len = data.len().min(self.memory.len());
        self.memory[..len].copy_from_slice(&data[..len]);
    }

    // Auxiliares do temporizador
    fn timer_bit_index(&self) -> u8 {
        match self.memory[0xFF07] & 0x03 { // TAC[1:0]
            0b00 => 9,   // 4096 Hz
            0b01 => 3,   // 262144 Hz
            0b10 => 5,   // 65536 Hz
            _ => 7,      // 16384 Hz
        }
    }

    fn timer_enabled(&self) -> bool {
        (self.memory[0xFF07] & 0x04) != 0
    }

    fn current_timer_signal(&self) -> bool {
        let enabled = self.timer_enabled();
        if !enabled { return false; }
        let bit = self.timer_bit_index();
        ((self.div_counter >> bit) & 1) != 0
    }

    // Avança o estado do temporizador pelo número de ciclos da CPU informado
    pub fn tick_timers(&mut self, cycles: u32) {
        // Trata recarga atrasada, se ativa; processa ciclo a ciclo para manter a precisão
        for _ in 0..cycles {
            // incrementa o divisor interno a cada ciclo da CPU
            self.div_counter = self.div_counter.wrapping_add(1);
            // reflete o registrador DIV como o byte alto de div_counter
            self.memory[0xFF04] = (self.div_counter >> 8) as u8;

            // Processa recarga atrasada do TIMA
            if self.tima_reload_delay > 0 {
                self.tima_reload_delay -= 1;
                if self.tima_reload_delay == 0 {
                    // Ao fim do atraso, carrega TMA em TIMA e solicita interrupção de Timer (IF bit 2)
                    let tma = self.memory[0xFF06];
                    self.memory[0xFF05] = tma;
                    self.memory[0xFF0F] |= 0x04; // IF Timer
                    println!("[TIMER] IF(TIMER)=1; TIMA<=TMA({:02X})", tma);
                }
            }

            // Lógica do timer: incrementa TIMA na borda de descida do bit selecionado quando habilitado
            let signal = self.current_timer_signal();
            if self.timer_last_signal && !signal {
                // borda de descida
                let tima = self.memory[0xFF05];
                if tima == 0xFF {
                    // overflow: zera e inicia atraso para recarga
                    self.memory[0xFF05] = 0x00;
                    // Segundo o hardware, a recarga ocorre após 4 ciclos
                    self.tima_reload_delay = 4;
                } else {
                    self.memory[0xFF05] = tima.wrapping_add(1);
                }
            }
            self.timer_last_signal = signal;
        }
    }
}