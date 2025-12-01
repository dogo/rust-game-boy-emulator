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
        }
    }

    /// Avança a state machine do TIMA reload
    /// NOTA: Agora usamos tima_cooldown em T-cycles, então esta função só lida com
    /// comportamentos obscuros durante o ciclo B (Reloaded)
    fn advance_tima_state_machine(&mut self, tima: &mut u8, tma: u8, _if_reg: &mut u8) {
        match self.tima_reload_state {
            TimaReloadState::Reloaded => {
                // No final do ciclo B, se TIMA não foi escrito durante o ciclo,
                // ele é recarregado com TMA. Mas se foi escrito, mantém o valor escrito.
                if !self.tima_written_this_cycle {
                    *tima = tma;
                }
                self.tima_written_this_cycle = false;
                self.tima_reload_state = TimaReloadState::Running;
            }
            TimaReloadState::Reloading => {
                // Este estado não é mais usado - cooldown é gerenciado em T-cycles
                self.tima_reload_state = TimaReloadState::Running;
            }
            TimaReloadState::Running => {}
        }
    }

    /// Incrementa TIMA, gerenciando overflow
    /// Segundo aquova.net: quando TIMA transborda, fica em 0 por 4 T-cycles antes de ser recarregado
    fn increment_tima(&mut self, tima: &mut u8, _tma: u8) {
        if *tima == 0xFF {
            // TIMA overflow! Fica 0 por 4 T-cycles, depois é recarregado com TMA
            *tima = 0x00;
            self.tima_increment_counter += 1;
            // Agenda reload após 4 T-cycles (div_counter incrementa a cada T-cycle)
            self.reload_pending = Some(self.div_counter.wrapping_add(4));
            println!(
                "[OVERFLOW] TIMA overflow at div={:04X}: set tima=0, reload_at={:04X}",
                self.div_counter,
                self.reload_pending.unwrap()
            );
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
        mut if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        // Processa T-cycles, avançando state machine no INÍCIO de cada M-cycle
        // IMPORTANTE: Segundo Pan Docs, quando TIMA transborda:
        // - Durante o M-cycle do overflow: TIMA fica em $00, state machine vai para Reloading
        // - No INÍCIO do próximo M-cycle: TMA é copiado para TIMA e IF é setado
        let mut remaining_cycles = cycles;

        while remaining_cycles > 0 {
            // Processa até 4 T-cycles (um M-cycle completo)
            let cycles_this_batch = remaining_cycles.min(4 - self.m_cycle_offset);

            for _ in 0..cycles_this_batch {
                self.div_counter = self.div_counter.wrapping_add(1);

                // Processa reload pendente (overflow de TIMA)
                if let Some(reload_at) = self.reload_pending {
                    // Usa wrapping_sub para lidar com overflow do div_counter
                    if self.div_counter.wrapping_sub(reload_at) < 0x8000 {
                        // Ainda não chegou no deadline
                    } else {
                        // Chegou no deadline: recarrega TIMA com TMA e seta interrupt
                        tima = tma;
                        if_reg |= 0x04; // Timer interrupt
                        self.reload_pending = None;
                        println!(
                            "[RELOAD] reload executed at div={:04X}: tima loaded with TMA={:02X}; IRQ raised",
                            self.div_counter, tma
                        );
                    }
                }

                // Detecta falling edge para TIMA
                if (tac & 0x04) != 0 {
                    let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
                    let cur_bit = (self.div_counter & trigger_bit) != 0;

                    // Falling edge: bit era 1, agora é 0
                    if self.prev_tima_bit && !cur_bit {
                        // Verifica se este falling edge deve ser suprimido
                        let suppressed = if let Some(deadline) = self.suppress_until {
                            // Verifica se ainda não chegou no deadline
                            // Para lidar com wrap do u16, calculamos a distância do div_counter até o deadline
                            // Se distance_to_deadline < 0x8000 e > 0, significa que ainda não chegou (sem wrap)
                            let distance_to_deadline = deadline.wrapping_sub(self.div_counter);
                            // Se distance_to_deadline < 0x8000, ainda não chegou
                            distance_to_deadline < 0x8000 && distance_to_deadline > 0
                        } else {
                            false
                        };

                        if suppressed {
                            println!(
                                "[IGNORED_EDGE] falling edge at div={:04X} ignored because suppress_until={:04X}",
                                self.div_counter,
                                self.suppress_until.unwrap()
                            );
                        } else {
                            // Limpa supressão se estava ativa
                            if self.suppress_until.is_some() {
                                self.suppress_until = None;
                            }
                            println!(
                                "[DIV] div={:04X} tac_bit={} cur_bit={} prev_bit={} falling_edge_detected=true suppressed=false -> increment TIMA -> tima=0x{:02X}",
                                self.div_counter,
                                TAC_BIT_INDICES[(tac & 0x03) as usize],
                                cur_bit,
                                self.prev_tima_bit,
                                tima
                            );
                            self.increment_tima(&mut tima, tma);
                        }
                    }

                    self.prev_tima_bit = cur_bit;
                } else {
                    // Timer desabilitado, reseta prev_bit
                    self.prev_tima_bit = false;
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

            self.m_cycle_offset = (self.m_cycle_offset + cycles_this_batch) % 4;
            // Avança state machine no FINAL de cada M-cycle
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

    /// Zera o DIV e retorna eventos
    /// IMPORTANTE: Segundo Pan Docs, resetar DIV pode causar um incremento em TIMA
    /// se o bit selecionado estava em 1 antes do reset
    pub fn reset_div(
        &mut self,
        mut tima: u8,
        tma: u8,
        tac: u8,
        if_reg: u8,
        double_speed: bool,
    ) -> (u8, u8, TimerEvents) {
        let mut events = TimerEvents::default();

        // IMPORTANTE: Antes de resetar, verifica se o bit estava em 1
        // Se estava, isso causa um falling edge (1 -> 0) que incrementa TIMA
        if (tac & 0x04) != 0 {
            let trigger_bit = TAC_TRIGGER_BITS[(tac & 0x03) as usize];
            if (self.div_counter & trigger_bit) != 0 {
                // Bit estava em 1, resetar causa falling edge -> incrementa TIMA
                self.increment_tima(&mut tima, tma);
            }
        }

        // Detecta edges para APU antes de resetar
        let apu_bit: u16 = if double_speed { 0x2000 } else { 0x1000 };
        let old_bit = (self.div_counter & apu_bit) != 0;

        if old_bit {
            events.apu_div_event = true;
        }

        // Reseta tudo
        self.div_counter = 0;
        self.last_div_bit = false;
        self.m_cycle_offset = 0;
        self.tima_written_this_cycle = false;
        self.tima_increment_counter = 0;
        self.suppress_until = None;
        self.prev_tima_bit = false;
        self.reload_pending = None;

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
                self.increment_tima(&mut tima, tma);
            }
        }

        (tima, if_reg)
    }

    /// Notifica escrita em TIMA
    /// IMPORTANTE: Segundo Pan Docs, escrever em TIMA durante o ciclo A (quando TIMA transborda)
    /// faz com que o overflow NÃO aconteça! TMA não é copiado e IF não é setado.
    /// Escrever em TIMA durante o ciclo B: "TIMA constantly copies its input, so it updates
    /// together with TMA". Isso significa que a escrita funciona imediatamente, mas será
    /// sobrescrita no final do ciclo se TMA mudar.
    pub fn notify_tima_write(&mut self, tac: u8) {
        // Cancela reload se pendente (comportamento obscuro: escrever TIMA durante reload cancela)
        if self.reload_pending.is_some() {
            self.reload_pending = None;
            println!("[WRITE] CPU wrote TIMA during reload: cancelled reload");
        }

        // Se estamos no estado Reloaded (ciclo B), marca que TIMA foi escrito
        if self.tima_reload_state == TimaReloadState::Reloaded {
            self.tima_written_this_cycle = true;
        }

        // Calcula supressão: impedir próxima falling edge até 1 full period do bit
        // Período completo = 2^(k+1) onde k é o índice do bit
        if (tac & 0x04) != 0 {
            let bit_index = TAC_BIT_INDICES[(tac & 0x03) as usize];
            let period = 1u16 << (bit_index + 1); // 2^(k+1)
            self.suppress_until = Some(self.div_counter.wrapping_add(period));
            println!(
                "[WRITE] CPU wrote TIMA at div={:04X}, suppress_until={:04X} (period={}, bit_index={})",
                self.div_counter,
                self.suppress_until.unwrap(),
                period,
                bit_index
            );
        } else {
            self.suppress_until = None;
        }
    }

    /// Verifica se estamos no ciclo B (Reloaded) - quando TIMA será recarregado
    /// Retorna true se uma escrita em TIMA durante este estado deve ser ignorada
    pub fn is_reloading_tima(&self) -> bool {
        self.tima_reload_state == TimaReloadState::Reloaded
    }
}
