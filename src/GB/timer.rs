/// Emula o timer/divisor do Game Boy
pub struct Timer {
    div_counter: u16,
    timer_last_signal: bool,
    reload_pending: bool,
    reload_delay: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_counter: 0,
            timer_last_signal: false,
            reload_pending: false,
            reload_delay: 0,
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
        if !enabled {
            return false;
        }
        let bit = self.timer_bit_index(tac);
        ((self.div_counter >> bit) & 1) != 0
    }

    #[inline]
    fn schedule_reload(&mut self) {
        self.reload_pending = true;
        self.reload_delay = 4;
    }

    #[inline]
    fn cancel_reload(&mut self) {
        self.reload_pending = false;
        self.reload_delay = 0;
    }

    #[inline]
    fn step_reload(&mut self, tma: u8, tima: &mut u8, if_reg: &mut u8) {
        if !self.reload_pending {
            return;
        }
        if self.reload_delay > 0 {
            self.reload_delay -= 1;
        }
        if self.reload_delay == 0 {
            *tima = tma;
            *if_reg |= 0x04;
            self.reload_pending = false;
        }
    }

    #[inline]
    fn apply_timer_tick(&mut self, mut tima: u8) -> u8 {
        if tima == 0xFF {
            tima = 0x00;
            self.schedule_reload();
            tima
        } else {
            tima.wrapping_add(1)
        }
    }

    pub fn tick(
        &mut self,
        cycles: u32,
        mut tima: u8,
        tma: u8,
        tac: u8,
        mut if_reg: u8,
    ) -> (u8, u8) {
        for _ in 0..cycles {
            self.div_counter = self.div_counter.wrapping_add(1);
            self.step_reload(tma, &mut tima, &mut if_reg);
            let signal = self.current_timer_signal(tac);
            if self.timer_last_signal && !signal {
                tima = self.apply_timer_tick(tima);
            }
            self.timer_last_signal = signal;
        }
        (tima, if_reg)
    }

    pub fn read_div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }

    /// Inicializa o div_counter para um valor específico (usado no estado pós-boot)
    /// O valor visível de DIV são os 8 bits superiores do contador de 16 bits
    pub fn set_div(&mut self, value: u8) {
        self.div_counter = (value as u16) << 8;
    }

    /// Zera o DIV e faz edge detect, incrementando TIMA se necessário
    pub fn reset_div(&mut self, tima: u8, _tma: u8, tac: u8, if_reg: u8) -> (u8, u8) {
        let old_signal = self.current_timer_signal(tac);
        self.div_counter = 0;
        let new_signal = self.current_timer_signal(tac);
        let mut tima = tima;
        let if_reg = if_reg;
        // Se houve borda de descida, incrementa TIMA
        if old_signal && !new_signal {
            tima = self.apply_timer_tick(tima);
        }
        self.timer_last_signal = new_signal;
        (tima, if_reg)
    }
    /// Detecta edge ao escrever em TAC (FF07)
    pub fn write_tac(
        &mut self,
        tima: u8,
        _tma: u8,
        old_tac: u8,
        new_tac: u8,
        if_reg: u8,
    ) -> (u8, u8) {
        let old_signal = self.current_timer_signal(old_tac);
        let new_signal = self.current_timer_signal(new_tac);
        let mut tima = tima;
        let if_reg = if_reg;
        // Se houve borda de descida, incrementa TIMA
        if old_signal && !new_signal {
            tima = self.apply_timer_tick(tima);
        }
        self.timer_last_signal = new_signal;
        (tima, if_reg)
    }

    /// Ao escrever em TIMA (FF05), cancelamos um reload pendente.
    #[inline]
    pub fn notify_tima_write(&mut self) {
        if self.reload_pending {
            self.cancel_reload();
        }
    }
}
