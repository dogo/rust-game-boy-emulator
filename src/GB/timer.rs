/// Emula o timer/divisor do Game Boy
pub struct Timer {
    div_counter: u16,
    tima_reload_delay: u8,
    timer_last_signal: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_counter: 0,
            tima_reload_delay: 0,
            timer_last_signal: false,
        }
    }

    fn timer_bit_index(&self, tac: u8) -> u8 {
        match tac & 0x03 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            _ => 7,
        }
    }

    fn timer_enabled(&self, tac: u8) -> bool {
        (tac & 0x04) != 0
    }

    fn current_timer_signal(&self, tac: u8) -> bool {
        let enabled = self.timer_enabled(tac);
        if !enabled { return false; }
        let bit = self.timer_bit_index(tac);
        ((self.div_counter >> bit) & 1) != 0
    }

    pub fn tick(&mut self, cycles: u32, mut tima: u8, tma: u8, tac: u8, mut if_reg: u8) -> (u8, u8) {
        for _ in 0..cycles {
            self.div_counter = self.div_counter.wrapping_add(1);
            if self.tima_reload_delay > 0 {
                self.tima_reload_delay -= 1;
                if self.tima_reload_delay == 0 {
                    tima = tma;
                    if_reg |= 0x04; // Timer interrupt
                }
            }
            let signal = self.current_timer_signal(tac);
            if self.timer_last_signal && !signal {
                if tima == 0xFF {
                    tima = 0x00;
                    self.tima_reload_delay = 4;
                } else {
                    tima = tima.wrapping_add(1);
                }
            }
            self.timer_last_signal = signal;
        }
        (tima, if_reg)
    }

    pub fn read_div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }

    /// Zera o DIV e faz edge detect, incrementando TIMA se necessário
    pub fn reset_div(&mut self, tima: u8, tma: u8, tac: u8, if_reg: u8) -> (u8, u8) {
        let old_signal = self.current_timer_signal(tac);
        self.div_counter = 0;
        let new_signal = self.current_timer_signal(tac);
        let mut tima = tima;
        let mut if_reg = if_reg;
        // Se houve borda de descida, incrementa TIMA
        if old_signal && !new_signal {
            if tima == 0xFF {
                tima = tma;
                if_reg |= 0x04; // Timer interrupt
            } else {
                tima = tima.wrapping_add(1);
            }
        }
        (tima, if_reg)
    }
    /// Detecta edge ao escrever em TAC (FF07)
    pub fn write_tac(&mut self, tima: u8, tma: u8, old_tac: u8, new_tac: u8, if_reg: u8) -> (u8, u8) {
        let old_signal = self.current_timer_signal(old_tac);
        let new_signal = self.current_timer_signal(new_tac);
        let mut tima = tima;
        let mut if_reg = if_reg;
        // Se houve borda de descida, incrementa TIMA
        if old_signal && !new_signal {
            if tima == 0xFF {
                tima = tma;
                if_reg |= 0x04; // Timer interrupt
            } else {
                tima = tima.wrapping_add(1);
            }
        }
        (tima, if_reg)
    }
}
