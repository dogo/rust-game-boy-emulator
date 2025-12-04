/// Emula o timer/divisor do Game Boy
/// Baseado na implementação do SameBoy
/// Ref: https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html

// Bits do div_counter que trigeram TIMA para cada valor de TAC & 0x03
const TAC_TRIGGER_BITS: [u16; 4] = [512, 8, 32, 128]; // bits 9, 3, 5, 7
// Índices dos bits (para calcular período completo: 2^(k+1))
const TAC_BIT_INDICES: [u8; 4] = [9, 3, 5, 7];

#[derive(Clone, Copy, PartialEq, Debug)]
enum TimaReloadState {
    Running,
    #[allow(dead_code)] // Mantido para compatibilidade com advance_tima_state_machine
    Reloading,
    Reloaded,
}

/// Eventos gerados pelo timer quando div_counter muda
#[derive(Default)]
pub struct TimerEvents {
    pub apu_div_event: bool,     // Falling edge no bit 12 (ou 13 em double speed)
    pub apu_div_secondary: bool, // Rising edge no bit 12
}

    pub struct Timer {
        div_counter: u16, // Contador interno que incrementa a cada T-cycle
        tima_reload_state: TimaReloadState,
        last_div_bit: bool,            // Para detectar edges no bit do APU
        m_cycle_offset: u32,           // Rastreia offset dentro do M-cycle atual (0-3)
        tima_written_this_cycle: bool, // Flag para rastrear se TIMA foi escrito durante o ciclo B
        tima_increment_counter: u32,   // Contador para rastrear quantas vezes TIMA foi incrementado
        suppress_until: Option<u16>, // Deadline absoluto (div_counter value) até quando suprimir falling edges
        prev_tima_bit: bool,         // Estado anterior do bit selecionado para detectar falling edges
        reload_pending: Option<u16>, // Deadline absoluto (div_counter value) para reload após overflow
        reload_just_reached: bool,   // Flag para forçar delay de um M-cycle antes de executar reload
        tma_reg: u8,                 // Valor atual de TMA (atualizado quando CPU escreve em TMA)
    }

impl Timer {
        pub fn new() -> Self {
            Timer {
                div_counter: 0,
                tima_reload_state: TimaReloadState::Running,
                last_div_bit: false,
                m_cycle_offset: 0,
                tima_written_this_cycle: false,
                tima_increment_counter: 0,
                suppress_until: None,
                prev_tima_bit: false,
                reload_pending: None,
                reload_just_reached: false,
                tma_reg: 0,
            }
        }

    fn advance_tima_state_machine(&mut self, tima: &mut u8, _tma: u8, if_reg: &mut u8) {
        match self.tima_reload_state {
            TimaReloadState::Reloaded => {
                if self.reload_just_reached {
                    self.reload_just_reached = false;
                } else {
                    if !self.tima_written_this_cycle {
                        *tima = self.tma_reg;
                        *if_reg |= 0x04;
                    }
                    self.tima_written_this_cycle = false;
                    self.tima_reload_state = TimaReloadState::Running;
                }
            }
            TimaReloadState::Reloading => {
                self.tima_reload_state = TimaReloadState::Running;
            }
            TimaReloadState::Running => {}
        }
    }

    /// Incrementa TIMA, gerenciando overflow
    /// Segundo aquova.net: quando TIMA transborda, fica em 0 por 4 T-cycles antes de ser recarregado
    fn increment_tima(&mut self, tima: &mut u8, _tma: u8) {
        if *tima == 0xFF {
            *tima = 0x00;
            self.tima_increment_counter += 1;
            let reload_delay_div_units = 3;
            self.reload_pending = Some(self.div_counter.wrapping_add(reload_delay_div_units));
            self.tima_reload_state = TimaReloadState::Reloading;
        } else {
            *tima = tima.wrapping_add(1);
            self.tima_increment_counter += 1;
        }
    }

    /// Tick do timer - chamado com T-cycles
    /// Retorna eventos do APU
    /// IMPORTANTE:
    /// - div_counter interno incrementa a cada T-cycle (para detectar edges precisos)
    /// - read_div() retorna apenas os 8 bits superiores (que mudam a cada 256 T-cycles)
    /// - State machine avança no INÍCIO de cada M-cycle
    pub fn tick(
        &mut self,
        cycles: u32,
        mut tima: u8,
        tma: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut if_reg = if_reg; // Make mutable locally
        let mut events = TimerEvents::default();

        // Processa T-cycles, avançando state machine no INÍCIO de cada M-cycle
        let mut remaining_cycles = cycles;

        while remaining_cycles > 0 {
            let cycles_this_batch = remaining_cycles.min(4 - self.m_cycle_offset);

            for _ in 0..cycles_this_batch {
                self.div_counter = self.div_counter.wrapping_add(1);

                if let Some(reload_at) = self.reload_pending {
                    if self.div_counter.wrapping_sub(reload_at) < 0x8000 {
                        self.reload_pending = None;
                        self.tima_reload_state = TimaReloadState::Reloaded;
                        self.reload_just_reached = true;
                    }
                }

                if (tac & 0x04) != 0 {
                    let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
                    let cur_bit = (self.div_counter & trigger_bit) != 0;

                    if let Some(deadline) = self.suppress_until {
                        let distance = deadline.wrapping_sub(self.div_counter);
                        if distance >= 0x8000 || distance == 0 {
                            self.suppress_until = None;
                        }
                    }

                    if self.prev_tima_bit && !cur_bit {
                        let suppressed = if let Some(deadline) = self.suppress_until {
                            let distance = deadline.wrapping_sub(self.div_counter);
                            distance > 0 && distance < 0x8000
                        } else {
                            false
                        };

                        if !suppressed {
                            if self.suppress_until.is_some() {
                                self.suppress_until = None;
                            }
                            self.increment_tima(&mut tima, tma);
                        }
                    }

                    self.prev_tima_bit = cur_bit;
                } else {
                    self.prev_tima_bit = false;
                    self.suppress_until = None;
                }

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

            self.m_cycle_offset = (self.m_cycle_offset + cycles_this_batch) % 4;
            if self.m_cycle_offset == 0 {
                self.advance_tima_state_machine(&mut tima, tma, &mut if_reg);
            }
            remaining_cycles -= cycles_this_batch;
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

    pub fn reset_div(
        &mut self,
        mut tima: u8,
        tma: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        if (tac & 0x04) != 0 {
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            if (self.div_counter & trigger_bit) != 0 {
                self.increment_tima(&mut tima, tma);
            }
        }

        let apu_bit: u16 = if double_speed { 0x2000 } else { 0x1000 };
        let old_bit = (self.div_counter & apu_bit) != 0;

        if old_bit {
            events.apu_div_event = true;
        }

        self.div_counter = 0;
        self.last_div_bit = false;
        self.m_cycle_offset = 0;
        self.tima_written_this_cycle = false;
        self.tima_increment_counter = 0;
        self.suppress_until = None;
        self.prev_tima_bit = false;
        self.reload_pending = None;
        self.reload_just_reached = false;

        (tima, if_reg, events)
    }

    pub fn write_tac(
        &mut self,
        mut tima: u8,
        tma: u8,
        old_tac: u8,
        new_tac: u8,
        if_reg: u8,
    ) -> (u8, u8) {
        let old_bit = TAC_TRIGGER_BITS[(old_tac & 0x03) as usize];
        let new_bit = TAC_TRIGGER_BITS[(new_tac & 0x03) as usize];

        if (old_tac & 0x04) == 0 {
            return (tima, if_reg);
        }

        if (self.div_counter & old_bit) != 0 {
            if (new_tac & 0x04) == 0 || (self.div_counter & new_bit) == 0 {
                self.increment_tima(&mut tima, tma);
            }
        }

        if (new_tac & 0x04) != 0 {
            let cur_bit = (self.div_counter & new_bit) != 0;
            self.prev_tima_bit = cur_bit;
        } else {
            self.prev_tima_bit = false;
        }

        (tima, if_reg)
    }

    pub fn notify_tima_write(&mut self, tac: u8) {
        if self.reload_pending.is_some() || self.tima_reload_state == TimaReloadState::Reloading {
            self.reload_pending = None;
            self.tima_reload_state = TimaReloadState::Running;
            self.reload_just_reached = false;
        }

        if self.tima_reload_state == TimaReloadState::Reloaded {
            self.tima_written_this_cycle = true;
        }

        if (tac & 0x04) != 0 {
            let bit_index = TAC_BIT_INDICES[(tac & 0x03) as usize];
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            let period = 1u16 << (bit_index + 1);
            self.suppress_until = Some(self.div_counter.wrapping_add(period));
            let cur_bit = (self.div_counter & trigger_bit) != 0;
            self.prev_tima_bit = cur_bit;
        } else {
            self.suppress_until = None;
        }
    }

    pub fn is_reloading_tima(&self) -> bool {
        self.tima_reload_state == TimaReloadState::Reloaded
    }

    pub fn notify_tma_write(&mut self, new_tma: u8) {
        self.tma_reg = new_tma;
    }
}
