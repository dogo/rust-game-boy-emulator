/// Emula o timer/divisor do Game Boy
/// Baseado na implementação do SameBoy
/// Ref: https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html

// Bits do div_counter que trigeram TIMA para cada valor de TAC & 0x03
const TAC_TRIGGER_BITS: [u16; 4] = [512, 8, 32, 128]; // bits 9, 3, 5, 7

#[derive(Clone, Copy, PartialEq)]
enum TimaReloadState {
    Running,
    Reloading,
    Reloaded,
}

/// Eventos gerados pelo timer quando div_counter muda
#[derive(Default)]
pub struct TimerEvents {
    pub apu_div_event: bool,      // Falling edge no bit 12 (ou 13 em double speed)
    pub apu_div_secondary: bool,  // Rising edge no bit 12
}

pub struct Timer {
    div_counter: u16,
    tima_reload_state: TimaReloadState,
    last_div_bit: bool, // Para detectar edges no bit do APU
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_counter: 0,
            tima_reload_state: TimaReloadState::Running,
            last_div_bit: false,
        }
    }

    /// Avança a state machine do TIMA reload
    fn advance_tima_state_machine(&mut self, tima: &mut u8, tma: u8, if_reg: &mut u8) {
        match self.tima_reload_state {
            TimaReloadState::Reloaded => {
                self.tima_reload_state = TimaReloadState::Running;
            }
            TimaReloadState::Reloading => {
                *if_reg |= 0x04; // Timer interrupt
                *tima = tma;
                self.tima_reload_state = TimaReloadState::Reloaded;
            }
            TimaReloadState::Running => {}
        }
    }

    /// Incrementa TIMA, gerenciando overflow
    fn increase_tima(&mut self, tima: &mut u8, _tma: u8) {
        *tima = tima.wrapping_add(1);
        if *tima == 0 {
            // TIMA overflow! Fica 0 por um M-cycle, depois é recarregado
            // NÃO recarregamos aqui - isso acontece no próximo M-cycle
            // via advance_tima_state_machine
            self.tima_reload_state = TimaReloadState::Reloading;
        }
    }

    /// Tick do timer - chamado com T-cycles
    /// Retorna eventos do APU
    pub fn tick(
        &mut self,
        cycles: u32,
        mut tima: u8,
        tma: u8,
        tac: u8,
        mut if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        // Avança state machine uma vez por M-cycle (4 T-cycles)
        // Isso é importante para o delay correto do TIMA reload
        let m_cycles = (cycles + 3) / 4; // Arredonda para cima
        for _ in 0..m_cycles {
            self.advance_tima_state_machine(&mut tima, tma, &mut if_reg);
        }

        for _ in 0..cycles {
            let old_div = self.div_counter;
            self.div_counter = self.div_counter.wrapping_add(1);

            // Detecta falling edge para TIMA
            if (tac & 0x04) != 0 {
                let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
                // Falling edge: bit era 1, agora é 0
                if (old_div & trigger_bit) != 0 && (self.div_counter & trigger_bit) == 0 {
                    self.increase_tima(&mut tima, tma);
                }
            }

            // Detecta edges para APU frame sequencer (bit 12 ou 13)
            let apu_bit: u16 = if double_speed { 0x2000 } else { 0x1000 };
            let current_bit = (self.div_counter & apu_bit) != 0;

            if self.last_div_bit && !current_bit {
                events.apu_div_event = true;
            }
            if !self.last_div_bit && current_bit {
                events.apu_div_secondary = true;
            }
            self.last_div_bit = current_bit;
        }

        (tima, if_reg, events)
    }

    /// Tick por M-cycle (4 T-cycles) - wrapper conveniente
    pub fn tick_m_cycle(
        &mut self,
        tima: u8,
        tma: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        self.tick(4, tima, tma, tac, if_reg, double_speed)
    }

    pub fn read_div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }

    pub fn get_div_counter(&self) -> u16 {
        self.div_counter
    }

    pub fn set_div(&mut self, value: u8) {
        self.div_counter = (value as u16) << 8;
    }

    pub fn set_div_counter(&mut self, value: u16) {
        self.div_counter = value;
    }

    /// Zera o DIV e retorna eventos
    pub fn reset_div(
        &mut self,
        mut tima: u8,
        tma: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        // Detecta falling edge para TIMA
        if (tac & 0x04) != 0 {
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            if (self.div_counter & trigger_bit) != 0 {
                self.increase_tima(&mut tima, tma);
            }
        }

        // Detecta edges para APU
        let apu_bit: u16 = if double_speed { 0x2000 } else { 0x1000 };
        let old_bit = (self.div_counter & apu_bit) != 0;

        if old_bit {
            events.apu_div_event = true;
        }

        self.div_counter = 0;
        self.last_div_bit = false;

        (tima, if_reg, events)
    }

    /// Timer glitch ao escrever em TAC
    pub fn write_tac(
        &mut self,
        mut tima: u8,
        tma: u8,
        old_tac: u8,
        new_tac: u8,
        if_reg: u8,
    ) -> (u8, u8) {
        // Glitch só acontece se old_tac estava habilitado
        if (old_tac & 0x04) == 0 {
            return (tima, if_reg);
        }

        let old_bit = TAC_TRIGGER_BITS[(old_tac & 0x03) as usize];
        let new_bit = TAC_TRIGGER_BITS[(new_tac & 0x03) as usize];

        // O bit antigo deve estar em 1
        if (self.div_counter & old_bit) != 0 {
            // E agora ou o timer está desabilitado, ou o novo bit é 0
            if (new_tac & 0x04) == 0 || (self.div_counter & new_bit) == 0 {
                self.increase_tima(&mut tima, tma);
            }
        }

        (tima, if_reg)
    }

    /// Notifica escrita em TIMA
    pub fn notify_tima_write(&mut self) {
        if self.tima_reload_state == TimaReloadState::Reloading {
            self.tima_reload_state = TimaReloadState::Running;
        }
    }
}
