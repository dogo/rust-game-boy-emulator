/// Emula o timer/divisor do Game Boy
/// Implementação do timer seguindo o comportamento do hardware Game Boy
/// Ref: https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html

// Bits do div_counter que trigeram TIMA para cada valor de TAC & 0x03
const TAC_TRIGGER_BITS: [u16; 4] = [512, 8, 32, 128]; // bits 9, 3, 5, 7

/// Eventos gerados pelo timer quando div_counter muda
#[derive(Default)]
pub struct TimerEvents {
    pub apu_div_event: bool, // Borda de descida no bit 12 (ou 13 em double speed)
    pub apu_div_secondary: bool, // Borda de subida no bit 12
}

pub struct Timer {
    div_counter: u16,            // Contador interno que incrementa a cada T-cycle
    last_div_bit: bool,          // Para detectar edges no bit do APU
    m_cycle_offset: u32,         // Rastreia offset dentro do M-cycle atual (0-3)
    tima_written_in_delay: bool, // Flag: TIMA foi escrito durante o período de delay (4 T-cycles)
    suppress_until: Option<u16>, // Prazo (valor do div_counter) para suprimir bordas de descida após escrita no TIMA
    prev_tima_bit: bool,         // Estado anterior do bit selecionado para detectar falling edges
    reload_pending: Option<u16>, // Valor de temp_counter em que o reload deve disparar (temp_counter + 4)
    tima_reloading: bool,        // true durante os 4 T-cycles entre overflow e reload
    tma_reg: u8,                 // Valor atual de TMA (atualizado quando CPU escreve em TMA)
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div_counter: 0,
            last_div_bit: false,
            m_cycle_offset: 0,
            tima_written_in_delay: false,
            suppress_until: None,
            prev_tima_bit: false,
            reload_pending: None,
            tima_reloading: false,
            tma_reg: 0,
        }
    }

    /// Tick do timer - chamado com T-cycles
    /// Retorna eventos do APU
    pub fn tick(
        &mut self,
        cycles: u32,
        mut tima: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut if_reg = if_reg;
        let mut events = TimerEvents::default();

        let mut remaining_cycles = cycles;

        while remaining_cycles > 0 {
            let cycles_this_batch = remaining_cycles.min(4 - self.m_cycle_offset).min(4);

            // Processa o batch de uma vez ao invés de T-cycle por T-cycle (div_counter)
            self.div_counter = self.div_counter.wrapping_add(cycles_this_batch as u16);

            // Processa TIMA se timer está habilitado - T-cycle por T-cycle para precisão
            if (tac & 0x04) != 0 {
                let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
                let old_counter = self.div_counter.wrapping_sub(cycles_this_batch as u16);
                let mut temp_counter = old_counter;
                let mut temp_prev_bit = self.prev_tima_bit;

                for _ in 0..cycles_this_batch {
                    temp_counter = temp_counter.wrapping_add(1);
                    let cur_bit = (temp_counter & trigger_bit) != 0;

                    // Verifica reload pendente (exatamente 4 T-cycles após overflow)
                    if let Some(reload_at) = self.reload_pending {
                        let dist = temp_counter.wrapping_sub(reload_at);
                        if dist < 0x8000 {
                            // Dispara reload: TIMA = TMA (a menos que TIMA foi escrito durante delay)
                            self.reload_pending = None;
                            self.tima_reloading = false;
                            if !self.tima_written_in_delay {
                                tima = self.tma_reg;
                            }
                            self.tima_written_in_delay = false;
                            if_reg |= 0x04;
                        }
                    }

                    // Limpa suppress_until quando deadline é alcançado
                    if let Some(deadline) = self.suppress_until {
                        let distance = deadline.wrapping_sub(temp_counter);
                        if distance >= 0x8000 || distance == 0 {
                            self.suppress_until = None;
                        }
                    }

                    // Detecta falling edge no bit selecionado por TAC → incrementa TIMA
                    if temp_prev_bit && !cur_bit {
                        let suppressed = if let Some(deadline) = self.suppress_until {
                            let distance = deadline.wrapping_sub(temp_counter);
                            distance > 0 && distance < 0x8000
                        } else {
                            false
                        };
                        if suppressed {
                            temp_prev_bit = cur_bit;
                            continue;
                        }
                        // Incrementa TIMA usando temp_counter para reload_pending preciso
                        if tima == 0xFF {
                            tima = 0x00;
                            // Reload em exatamente 4 T-cycles a partir deste T-cycle
                            self.reload_pending = Some(temp_counter.wrapping_add(4));
                            self.tima_reloading = true;
                            self.tima_written_in_delay = false;
                        } else {
                            tima = tima.wrapping_add(1);
                        }
                    }

                    temp_prev_bit = cur_bit;
                }
                self.prev_tima_bit = temp_prev_bit;
            } else {
                self.prev_tima_bit = false;
                self.suppress_until = None;
            }

            // Processa eventos do APU
            let apu_bit: u16 = if double_speed { 0x2000 } else { 0x1000 };
            let old_counter = self.div_counter.wrapping_sub(cycles_this_batch as u16);
            let old_apu_bit = (old_counter & apu_bit) != 0;
            let new_apu_bit = (self.div_counter & apu_bit) != 0;

            if old_apu_bit && !new_apu_bit {
                events.apu_div_event = true;
            }
            if !old_apu_bit && new_apu_bit {
                events.apu_div_secondary = true;
            }
            self.last_div_bit = new_apu_bit;

            self.m_cycle_offset = (self.m_cycle_offset + cycles_this_batch) % 4;
            remaining_cycles -= cycles_this_batch;
        }

        (tima, if_reg, events)
    }

    /// Tick por M-cycle (4 T-cycles) - wrapper conveniente
    pub fn tick_m_cycle(
        &mut self,
        tima: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        self.tick(4, tima, tac, if_reg, double_speed)
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
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        if (tac & 0x04) != 0 {
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            if (self.div_counter & trigger_bit) != 0 {
                // Borda de descida no bit do timer quando div é resetado
                if tima == 0xFF {
                    tima = 0x00;
                    self.reload_pending = Some(self.div_counter.wrapping_add(4));
                    self.tima_reloading = true;
                    self.tima_written_in_delay = false;
                } else {
                    tima = tima.wrapping_add(1);
                }
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
        self.tima_written_in_delay = false;
        self.suppress_until = None;
        self.prev_tima_bit = false;
        self.reload_pending = None;
        self.tima_reloading = false;

        (tima, if_reg, events)
    }

    pub fn write_tac(&mut self, mut tima: u8, old_tac: u8, new_tac: u8, if_reg: u8) -> (u8, u8) {
        let old_bit = TAC_TRIGGER_BITS[(old_tac & 0x03) as usize];
        let new_bit = TAC_TRIGGER_BITS[(new_tac & 0x03) as usize];

        if (old_tac & 0x04) == 0 {
            // Timer estava parado; iniciar não dispara um tick
            if (new_tac & 0x04) != 0 {
                let cur_bit = (self.div_counter & new_bit) != 0;
                self.prev_tima_bit = cur_bit;
            }
            return (tima, if_reg);
        }

        if (self.div_counter & old_bit) != 0 {
            if (new_tac & 0x04) == 0 || (self.div_counter & new_bit) == 0 {
                if tima == 0xFF {
                    tima = 0x00;
                    self.reload_pending = Some(self.div_counter.wrapping_add(4));
                    self.tima_reloading = true;
                    self.tima_written_in_delay = false;
                } else {
                    tima = tima.wrapping_add(1);
                }
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
        if self.tima_reloading {
            // TIMA escrito durante o delay: reload para TMA é cancelado, interrupção ainda dispara
            self.tima_written_in_delay = true;
        } else {
            // Escrita normal no TIMA: cancela qualquer reload pendente
            self.reload_pending = None;
            self.tima_reloading = false;
            self.tima_written_in_delay = false;
        }

        if (tac & 0x04) != 0 {
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            let cur_bit = (self.div_counter & trigger_bit) != 0;
            self.prev_tima_bit = cur_bit;
        } else {
            self.suppress_until = None;
        }
    }

    /// Retorna true se TIMA está no período de delay (entre overflow e reload)
    pub fn is_reloading_tima(&self) -> bool {
        self.tima_reloading
    }

    pub fn notify_tma_write(&mut self, new_tma: u8) {
        self.tma_reg = new_tma;
    }
}
