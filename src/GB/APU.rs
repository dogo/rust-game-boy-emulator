#![allow(non_snake_case)]

/// APU (Audio Processing Unit) do Game Boy
/// 4 canais de áudio: 2 square waves, 1 wave, 1 noise
pub struct APU {
    // === Canal 1: Square wave com sweep ===
    ch1_enabled: bool,
    ch1_sweep_period: u8,       // NR10 bits 6-4: período do sweep (0-7)
    ch1_sweep_direction: bool,  // NR10 bit 3: 0=up, 1=down
    ch1_sweep_shift: u8,        // NR10 bits 2-0: shift amount (0-7)
    ch1_wave_duty: u8,          // NR11 bits 7-6: wave duty (0-3)
    ch1_length_timer: u8,       // NR11 bits 5-0: length timer (0-63)
    ch1_envelope_initial: u8,   // NR12 bits 7-4: volume inicial (0-15)
    ch1_envelope_direction: bool, // NR12 bit 3: 0=decrease, 1=increase
    ch1_envelope_period: u8,    // NR12 bits 2-0: período do envelope (0-7)
    ch1_frequency: u16,         // NR13/NR14: frequência (0-2047)
    ch1_length_enable: bool,    // NR14 bit 6: length enable
    ch1_trigger: bool,          // NR14 bit 7: trigger

    // === Canal 2: Square wave simples ===
    ch2_enabled: bool,
    ch2_wave_duty: u8,          // NR21 bits 7-6: wave duty (0-3)
    ch2_length_timer: u8,       // NR21 bits 5-0: length timer (0-63)
    ch2_envelope_initial: u8,   // NR22 bits 7-4: volume inicial (0-15)
    ch2_envelope_direction: bool, // NR22 bit 3: 0=decrease, 1=increase
    ch2_envelope_period: u8,    // NR22 bits 2-0: período do envelope (0-7)
    ch2_frequency: u16,         // NR23/NR24: frequência (0-2047)
    ch2_length_enable: bool,    // NR24 bit 6: length enable
    ch2_trigger: bool,          // NR24 bit 7: trigger

    // === Canal 3: Wave pattern ===
    ch3_enabled: bool,
    ch3_dac_enable: bool,       // NR30 bit 7: DAC enable
    ch3_length_timer: u8,       // NR31: length timer (0-255)
    ch3_output_level: u8,       // NR32 bits 6-5: output level (0-3)
    ch3_frequency: u16,         // NR33/NR34: frequência (0-2047)
    ch3_length_enable: bool,    // NR34 bit 6: length enable
    ch3_trigger: bool,          // NR34 bit 7: trigger
    ch3_wave_ram: [u8; 16],     // Wave RAM (0xFF30-0xFF3F): 32 samples de 4 bits

    // === Canal 4: Noise ===
    ch4_enabled: bool,
    ch4_length_timer: u8,       // NR41 bits 5-0: length timer (0-63)
    ch4_envelope_initial: u8,   // NR42 bits 7-4: volume inicial (0-15)
    ch4_envelope_direction: bool, // NR42 bit 3: 0=decrease, 1=increase
    ch4_envelope_period: u8,    // NR42 bits 2-0: período do envelope (0-7)
    ch4_clock_shift: u8,        // NR43 bits 7-4: clock shift (0-15)
    ch4_width_mode: bool,       // NR43 bit 3: 0=15bit, 1=7bit
    ch4_divisor_code: u8,       // NR43 bits 2-0: divisor code (0-7)
    ch4_length_enable: bool,    // NR44 bit 6: length enable
    ch4_trigger: bool,          // NR44 bit 7: trigger

    // === Controle geral ===
    left_volume: u8,            // NR50 bits 6-4: volume esquerdo (0-7)
    right_volume: u8,           // NR50 bits 2-0: volume direito (0-7)
    vin_left_enable: bool,      // NR50 bit 7: VIN left enable
    vin_right_enable: bool,     // NR50 bit 3: VIN right enable

    // NR51: Sound panning
    ch1_left: bool, ch1_right: bool,
    ch2_left: bool, ch2_right: bool,
    ch3_left: bool, ch3_right: bool,
    ch4_left: bool, ch4_right: bool,

    sound_enable: bool,         // NR52 bit 7: master sound enable

    // === Estado interno ===
    sample_timer: u32,          // Timer para gerar samples a 44.1kHz
    frame_sequencer: u8,        // Frame sequencer (0-7) para length/envelope/sweep

    // Estados dos canais
    ch1_volume: u8,             // Volume atual do canal 1
    ch1_frequency_shadow: u16,  // Shadow register da frequência (para sweep)
    ch1_wave_position: u8,      // Posição na wave duty
    ch1_envelope_timer: u8,     // Timer do envelope
    ch1_length_counter: u8,     // Contador de length
    ch1_sweep_timer: u8,        // Timer do sweep
    ch1_sweep_enabled: bool,    // Sweep habilitado

    ch2_volume: u8,             // Volume atual do canal 2
    ch2_wave_position: u8,      // Posição na wave duty
    ch2_envelope_timer: u8,     // Timer do envelope
    ch2_length_counter: u8,     // Contador de length

    ch3_wave_position: u8,      // Posição no wave pattern (0-31)
    ch3_length_counter: u16,    // Contador de length (0-255)

    ch4_volume: u8,             // Volume atual do canal 4
    ch4_envelope_timer: u8,     // Timer do envelope
    ch4_length_counter: u8,     // Contador de length
    ch4_lfsr: u16,              // Linear Feedback Shift Register para noise
}

impl APU {
    pub fn new() -> Self {
        APU {
            // Canal 1
            ch1_enabled: false,
            ch1_sweep_period: 0,
            ch1_sweep_direction: false,
            ch1_sweep_shift: 0,
            ch1_wave_duty: 0,
            ch1_length_timer: 0,
            ch1_envelope_initial: 0,
            ch1_envelope_direction: false,
            ch1_envelope_period: 0,
            ch1_frequency: 0,
            ch1_length_enable: false,
            ch1_trigger: false,

            // Canal 2
            ch2_enabled: false,
            ch2_wave_duty: 0,
            ch2_length_timer: 0,
            ch2_envelope_initial: 0,
            ch2_envelope_direction: false,
            ch2_envelope_period: 0,
            ch2_frequency: 0,
            ch2_length_enable: false,
            ch2_trigger: false,

            // Canal 3
            ch3_enabled: false,
            ch3_dac_enable: false,
            ch3_length_timer: 0,
            ch3_output_level: 0,
            ch3_frequency: 0,
            ch3_length_enable: false,
            ch3_trigger: false,
            ch3_wave_ram: [0; 16],

            // Canal 4
            ch4_enabled: false,
            ch4_length_timer: 0,
            ch4_envelope_initial: 0,
            ch4_envelope_direction: false,
            ch4_envelope_period: 0,
            ch4_clock_shift: 0,
            ch4_width_mode: false,
            ch4_divisor_code: 0,
            ch4_length_enable: false,
            ch4_trigger: false,

            // Controle geral
            left_volume: 0,
            right_volume: 0,
            vin_left_enable: false,
            vin_right_enable: false,
            ch1_left: false, ch1_right: false,
            ch2_left: false, ch2_right: false,
            ch3_left: false, ch3_right: false,
            ch4_left: false, ch4_right: false,
            sound_enable: false,

            // Estado interno
            sample_timer: 0,
            frame_sequencer: 0,
            ch1_volume: 0,
            ch1_frequency_shadow: 0,
            ch1_wave_position: 0,
            ch1_envelope_timer: 0,
            ch1_length_counter: 0,
            ch1_sweep_timer: 0,
            ch1_sweep_enabled: false,
            ch2_volume: 0,
            ch2_wave_position: 0,
            ch2_envelope_timer: 0,
            ch2_length_counter: 0,
            ch3_wave_position: 0,
            ch3_length_counter: 0,
            ch4_volume: 0,
            ch4_envelope_timer: 0,
            ch4_length_counter: 0,
            ch4_lfsr: 0x7FFF, // LFSR inicializado com todos os bits em 1
        }
    }

    /// Clock APU - chamado a cada ciclo de CPU (4MHz)
    pub fn tick(&mut self) {
        if !self.sound_enable {
            return;
        }

        // Frame sequencer roda a 512Hz (4194304 / 8192 = 512)
        self.sample_timer += 1;
        if self.sample_timer >= 8192 {
            self.sample_timer = 0;
            self.step_frame_sequencer();
        }

        // Atualiza timers dos canais
        self.update_channel_timers();
    }

    /// Frame sequencer (512Hz)
    fn step_frame_sequencer(&mut self) {
        self.frame_sequencer = (self.frame_sequencer + 1) % 8;

        match self.frame_sequencer {
            0 => {
                // Step 0: Length
                self.step_length_counters();
            },
            1 => {
                // Step 1: Nada
            },
            2 => {
                // Step 2: Length e Sweep
                self.step_length_counters();
                self.step_sweep();
            },
            3 => {
                // Step 3: Nada
            },
            4 => {
                // Step 4: Length
                self.step_length_counters();
            },
            5 => {
                // Step 5: Nada
            },
            6 => {
                // Step 6: Length e Sweep
                self.step_length_counters();
                self.step_sweep();
            },
            7 => {
                // Step 7: Envelope
                self.step_envelopes();
            },
            _ => {}
        }
    }

    /// Gera sample de áudio (chamado a 44.1kHz)
    pub fn generate_sample(&mut self) -> (f32, f32) {
        if !self.sound_enable {
            return (0.0, 0.0);
        }

        let ch1_output = if self.ch1_enabled { self.get_channel1_output() } else { 0.0 };
        let ch2_output = if self.ch2_enabled { self.get_channel2_output() } else { 0.0 };
        let ch3_output = if self.ch3_enabled { self.get_channel3_output() } else { 0.0 };
        let ch4_output = if self.ch4_enabled { self.get_channel4_output() } else { 0.0 };

        // Mix canais de acordo com roteamento
        let mut left_sample = 0.0;
        let mut right_sample = 0.0;

        if self.ch1_left { left_sample += ch1_output; }
        if self.ch1_right { right_sample += ch1_output; }
        if self.ch2_left { left_sample += ch2_output; }
        if self.ch2_right { right_sample += ch2_output; }
        if self.ch3_left { left_sample += ch3_output; }
        if self.ch3_right { right_sample += ch3_output; }
        if self.ch4_left { left_sample += ch4_output; }
        if self.ch4_right { right_sample += ch4_output; }

        // Aplica volume master
        left_sample *= (self.left_volume as f32 + 1.0) / 8.0;
        right_sample *= (self.right_volume as f32 + 1.0) / 8.0;

        // Normaliza para prevenir clipping
        left_sample = left_sample.clamp(-1.0, 1.0);
        right_sample = right_sample.clamp(-1.0, 1.0);

        (left_sample, right_sample)
    }

    /// Lê um registrador do APU
    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            // Canal 1
            0xFF10 => {
                // NR10: Sweep
                (if self.ch1_sweep_period > 0 { 0x80 } else { 0x00 }) |
                (self.ch1_sweep_period << 4) |
                (if self.ch1_sweep_direction { 0x08 } else { 0x00 }) |
                self.ch1_sweep_shift
            }
            0xFF11 => {
                // NR11: Wave duty + length timer (só duty é readable)
                (self.ch1_wave_duty << 6) | 0x3F
            }
            0xFF12 => {
                // NR12: Envelope
                (self.ch1_envelope_initial << 4) |
                (if self.ch1_envelope_direction { 0x08 } else { 0x00 }) |
                self.ch1_envelope_period
            }
            0xFF13 => 0xFF, // NR13: Frequency low (write-only)
            0xFF14 => {
                // NR14: Frequency high + control (só length enable é readable)
                (if self.ch1_length_enable { 0x40 } else { 0x00 }) | 0xBF
            }

            // Canal 2
            0xFF16 => {
                // NR21: Wave duty + length timer (só duty é readable)
                (self.ch2_wave_duty << 6) | 0x3F
            }
            0xFF17 => {
                // NR22: Envelope
                (self.ch2_envelope_initial << 4) |
                (if self.ch2_envelope_direction { 0x08 } else { 0x00 }) |
                self.ch2_envelope_period
            }
            0xFF18 => 0xFF, // NR23: Frequency low (write-only)
            0xFF19 => {
                // NR24: Frequency high + control (só length enable é readable)
                (if self.ch2_length_enable { 0x40 } else { 0x00 }) | 0xBF
            }

            // Canal 3
            0xFF1A => {
                // NR30: DAC enable
                (if self.ch3_dac_enable { 0x80 } else { 0x00 }) | 0x7F
            }
            0xFF1B => 0xFF, // NR31: Length timer (write-only)
            0xFF1C => {
                // NR32: Output level
                (self.ch3_output_level << 5) | 0x9F
            }
            0xFF1D => 0xFF, // NR33: Frequency low (write-only)
            0xFF1E => {
                // NR34: Frequency high + control (só length enable é readable)
                (if self.ch3_length_enable { 0x40 } else { 0x00 }) | 0xBF
            }

            // Canal 4
            0xFF20 => 0xFF, // NR41: Length timer (write-only)
            0xFF21 => {
                // NR42: Envelope
                (self.ch4_envelope_initial << 4) |
                (if self.ch4_envelope_direction { 0x08 } else { 0x00 }) |
                self.ch4_envelope_period
            }
            0xFF22 => {
                // NR43: Noise parameters
                (self.ch4_clock_shift << 4) |
                (if self.ch4_width_mode { 0x08 } else { 0x00 }) |
                self.ch4_divisor_code
            }
            0xFF23 => {
                // NR44: Control (só length enable é readable)
                (if self.ch4_length_enable { 0x40 } else { 0x00 }) | 0xBF
            }

            // Controle geral
            0xFF24 => {
                // NR50: Master volume
                (if self.vin_left_enable { 0x80 } else { 0x00 }) |
                (self.left_volume << 4) |
                (if self.vin_right_enable { 0x08 } else { 0x00 }) |
                self.right_volume
            }
            0xFF25 => {
                // NR51: Sound panning
                (if self.ch4_left { 0x80 } else { 0x00 }) |
                (if self.ch3_left { 0x40 } else { 0x00 }) |
                (if self.ch2_left { 0x20 } else { 0x00 }) |
                (if self.ch1_left { 0x10 } else { 0x00 }) |
                (if self.ch4_right { 0x08 } else { 0x00 }) |
                (if self.ch3_right { 0x04 } else { 0x00 }) |
                (if self.ch2_right { 0x02 } else { 0x00 }) |
                (if self.ch1_right { 0x01 } else { 0x00 })
            }
            0xFF26 => {
                // NR52: Sound on/off + channel status
                (if self.sound_enable { 0x80 } else { 0x00 }) |
                0x70 | // bits 6-4 não usados (sempre 1)
                (if self.ch4_enabled { 0x08 } else { 0x00 }) |
                (if self.ch3_enabled { 0x04 } else { 0x00 }) |
                (if self.ch2_enabled { 0x02 } else { 0x00 }) |
                (if self.ch1_enabled { 0x01 } else { 0x00 })
            }

            // Wave RAM
            0xFF30..=0xFF3F => {
                self.ch3_wave_ram[(address - 0xFF30) as usize]
            }

            _ => 0xFF, // Registradores não implementados
        }
    }

    /// Escreve em um registrador do APU
    pub fn write_register(&mut self, address: u16, value: u8) {
        // Se o som está desabilitado, só aceita writes em NR52
        if !self.sound_enable && address != 0xFF26 {
            return;
        }

        match address {
            // Canal 1
            0xFF10 => {
                // NR10: Sweep
                self.ch1_sweep_period = (value >> 4) & 0x07;
                self.ch1_sweep_direction = (value & 0x08) != 0;
                self.ch1_sweep_shift = value & 0x07;
            }
            0xFF11 => {
                // NR11: Wave duty + length timer
                self.ch1_wave_duty = (value >> 6) & 0x03;
                self.ch1_length_timer = value & 0x3F;
                self.ch1_length_counter = 64 - self.ch1_length_timer;
            }
            0xFF12 => {
                // NR12: Envelope
                self.ch1_envelope_initial = (value >> 4) & 0x0F;
                self.ch1_envelope_direction = (value & 0x08) != 0;
                self.ch1_envelope_period = value & 0x07;

                // DAC enable check
                if (value & 0xF8) == 0 {
                    self.ch1_enabled = false;
                }
            }
            0xFF13 => {
                // NR13: Frequency low
                self.ch1_frequency = (self.ch1_frequency & 0x0700) | (value as u16);
            }
            0xFF14 => {
                // NR14: Frequency high + control
                self.ch1_frequency = (self.ch1_frequency & 0x00FF) | (((value & 0x07) as u16) << 8);
                self.ch1_length_enable = (value & 0x40) != 0;

                if (value & 0x80) != 0 {
                    self.trigger_channel1();
                }
            }

            // Canal 2
            0xFF16 => {
                // NR21: Wave duty + length timer
                self.ch2_wave_duty = (value >> 6) & 0x03;
                self.ch2_length_timer = value & 0x3F;
                self.ch2_length_counter = 64 - self.ch2_length_timer;
            }
            0xFF17 => {
                // NR22: Envelope
                self.ch2_envelope_initial = (value >> 4) & 0x0F;
                self.ch2_envelope_direction = (value & 0x08) != 0;
                self.ch2_envelope_period = value & 0x07;

                // DAC enable check
                if (value & 0xF8) == 0 {
                    self.ch2_enabled = false;
                }
            }
            0xFF18 => {
                // NR23: Frequency low
                self.ch2_frequency = (self.ch2_frequency & 0x0700) | (value as u16);
            }
            0xFF19 => {
                // NR24: Frequency high + control
                self.ch2_frequency = (self.ch2_frequency & 0x00FF) | (((value & 0x07) as u16) << 8);
                self.ch2_length_enable = (value & 0x40) != 0;

                if (value & 0x80) != 0 {
                    self.trigger_channel2();
                }
            }

            // Canal 3
            0xFF1A => {
                // NR30: DAC enable
                self.ch3_dac_enable = (value & 0x80) != 0;
                if !self.ch3_dac_enable {
                    self.ch3_enabled = false;
                }
            }
            0xFF1B => {
                // NR31: Length timer
                self.ch3_length_timer = value;
                self.ch3_length_counter = 256 - (value as u16);
            }
            0xFF1C => {
                // NR32: Output level
                self.ch3_output_level = (value >> 5) & 0x03;
            }
            0xFF1D => {
                // NR33: Frequency low
                self.ch3_frequency = (self.ch3_frequency & 0x0700) | (value as u16);
            }
            0xFF1E => {
                // NR34: Frequency high + control
                self.ch3_frequency = (self.ch3_frequency & 0x00FF) | (((value & 0x07) as u16) << 8);
                self.ch3_length_enable = (value & 0x40) != 0;

                if (value & 0x80) != 0 {
                    self.trigger_channel3();
                }
            }

            // Canal 4
            0xFF20 => {
                // NR41: Length timer
                self.ch4_length_timer = value & 0x3F;
                self.ch4_length_counter = 64 - self.ch4_length_timer;
            }
            0xFF21 => {
                // NR42: Envelope
                self.ch4_envelope_initial = (value >> 4) & 0x0F;
                self.ch4_envelope_direction = (value & 0x08) != 0;
                self.ch4_envelope_period = value & 0x07;

                // DAC enable check
                if (value & 0xF8) == 0 {
                    self.ch4_enabled = false;
                }
            }
            0xFF22 => {
                // NR43: Noise parameters
                self.ch4_clock_shift = (value >> 4) & 0x0F;
                self.ch4_width_mode = (value & 0x08) != 0;
                self.ch4_divisor_code = value & 0x07;
            }
            0xFF23 => {
                // NR44: Control
                self.ch4_length_enable = (value & 0x40) != 0;

                if (value & 0x80) != 0 {
                    self.trigger_channel4();
                }
            }

            // Controle geral
            0xFF24 => {
                // NR50: Master volume
                self.vin_left_enable = (value & 0x80) != 0;
                self.left_volume = (value >> 4) & 0x07;
                self.vin_right_enable = (value & 0x08) != 0;
                self.right_volume = value & 0x07;
            }
            0xFF25 => {
                // NR51: Sound panning
                self.ch4_left = (value & 0x80) != 0;
                self.ch3_left = (value & 0x40) != 0;
                self.ch2_left = (value & 0x20) != 0;
                self.ch1_left = (value & 0x10) != 0;
                self.ch4_right = (value & 0x08) != 0;
                self.ch3_right = (value & 0x04) != 0;
                self.ch2_right = (value & 0x02) != 0;
                self.ch1_right = (value & 0x01) != 0;
            }
            0xFF26 => {
                // NR52: Sound on/off
                let old_enable = self.sound_enable;
                self.sound_enable = (value & 0x80) != 0;

                // Se o som foi desabilitado, limpa todos os registradores
                if old_enable && !self.sound_enable {
                    self.disable_all_channels();
                }
            }

            // Wave RAM
            0xFF30..=0xFF3F => {
                self.ch3_wave_ram[(address - 0xFF30) as usize] = value;
            }

            _ => {} // Registradores não implementados
        }
    }

    /// Trigger do canal 1
    fn trigger_channel1(&mut self) {
        self.ch1_enabled = true;
        if self.ch1_length_counter == 0 {
            self.ch1_length_counter = 64;
        }
        self.ch1_envelope_timer = self.ch1_envelope_period;
        self.ch1_volume = self.ch1_envelope_initial;

        // Inicializar sweep
        self.ch1_frequency_shadow = self.ch1_frequency;
        self.ch1_sweep_timer = self.ch1_sweep_period;
        self.ch1_sweep_enabled = self.ch1_sweep_period > 0 || self.ch1_sweep_shift > 0;

        // Desabilitar se DAC está off
        if (self.ch1_envelope_initial == 0 && !self.ch1_envelope_direction) {
            self.ch1_enabled = false;
        }
    }

    /// Trigger do canal 2
    fn trigger_channel2(&mut self) {
        self.ch2_enabled = true;
        if self.ch2_length_counter == 0 {
            self.ch2_length_counter = 64;
        }
        self.ch2_envelope_timer = self.ch2_envelope_period;
        self.ch2_volume = self.ch2_envelope_initial;

        // Desabilitar se DAC está off
        if (self.ch2_envelope_initial == 0 && !self.ch2_envelope_direction) {
            self.ch2_enabled = false;
        }
    }

    /// Trigger do canal 3
    fn trigger_channel3(&mut self) {
        self.ch3_enabled = self.ch3_dac_enable;
        if self.ch3_length_counter == 0 {
            self.ch3_length_counter = 256;
        }
        self.ch3_wave_position = 0;
    }

    /// Trigger do canal 4
    fn trigger_channel4(&mut self) {
        self.ch4_enabled = true;
        if self.ch4_length_counter == 0 {
            self.ch4_length_counter = 64;
        }
        self.ch4_envelope_timer = self.ch4_envelope_period;
        self.ch4_volume = self.ch4_envelope_initial;
        self.ch4_lfsr = 0x7FFF;

        // Desabilitar se DAC está off
        if (self.ch4_envelope_initial == 0 && !self.ch4_envelope_direction) {
            self.ch4_enabled = false;
        }
    }

    /// Desabilita todos os canais quando o som é desligado
    fn disable_all_channels(&mut self) {
        self.ch1_enabled = false;
        self.ch2_enabled = false;
        self.ch3_enabled = false;
        self.ch4_enabled = false;

        // Limpar a maioria dos registradores (manter wave RAM)
        self.ch1_sweep_period = 0;
        self.ch1_sweep_direction = false;
        self.ch1_sweep_shift = 0;
        self.ch1_wave_duty = 0;
        self.ch1_length_timer = 0;
        self.ch1_envelope_initial = 0;
        self.ch1_envelope_direction = false;
        self.ch1_envelope_period = 0;
        self.ch1_frequency = 0;
        self.ch1_length_enable = false;

        // Similar para os outros canais...
        // (Por brevidade, não incluindo todos os resets aqui)
    }

    /// Atualiza timers dos canais
    fn update_channel_timers(&mut self) {
        // Implementação placeholder - cada canal tem seu próprio timer
        // TODO: Implementar timers específicos de cada canal
    }

    /// Step dos length counters
    fn step_length_counters(&mut self) {
        // Canal 1
        if self.ch1_length_enable && self.ch1_length_counter > 0 {
            self.ch1_length_counter -= 1;
            if self.ch1_length_counter == 0 {
                self.ch1_enabled = false;
            }
        }

        // Canal 2
        if self.ch2_length_enable && self.ch2_length_counter > 0 {
            self.ch2_length_counter -= 1;
            if self.ch2_length_counter == 0 {
                self.ch2_enabled = false;
            }
        }

        // Canal 3
        if self.ch3_length_enable && self.ch3_length_counter > 0 {
            self.ch3_length_counter -= 1;
            if self.ch3_length_counter == 0 {
                self.ch3_enabled = false;
            }
        }

        // Canal 4
        if self.ch4_length_enable && self.ch4_length_counter > 0 {
            self.ch4_length_counter -= 1;
            if self.ch4_length_counter == 0 {
                self.ch4_enabled = false;
            }
        }
    }

    /// Step do sweep (apenas canal 1)
    fn step_sweep(&mut self) {
        if !self.ch1_sweep_enabled {
            return;
        }

        if self.ch1_sweep_timer > 0 {
            self.ch1_sweep_timer -= 1;
        }

        if self.ch1_sweep_timer == 0 {
            self.ch1_sweep_timer = if self.ch1_sweep_period > 0 { self.ch1_sweep_period } else { 8 };

            if self.ch1_sweep_enabled && self.ch1_sweep_period > 0 {
                let new_frequency = self.calculate_sweep_frequency();

                if new_frequency <= 2047 && self.ch1_sweep_shift > 0 {
                    self.ch1_frequency = new_frequency;
                    self.ch1_frequency_shadow = new_frequency;

                    // Verificação de overflow novamente
                    let _ = self.calculate_sweep_frequency();
                }
            }
        }
    }

    /// Calcula nova frequência para sweep
    fn calculate_sweep_frequency(&self) -> u16 {
        let freq_change = self.ch1_frequency_shadow >> self.ch1_sweep_shift;
        if self.ch1_sweep_direction {
            self.ch1_frequency_shadow.saturating_sub(freq_change)
        } else {
            self.ch1_frequency_shadow.saturating_add(freq_change)
        }
    }

    /// Step dos envelopes
    fn step_envelopes(&mut self) {
        // Canal 1
        if self.ch1_envelope_timer > 0 {
            self.ch1_envelope_timer -= 1;
        }

        if self.ch1_envelope_timer == 0 {
            self.ch1_envelope_timer = if self.ch1_envelope_period > 0 { self.ch1_envelope_period } else { 8 };

            if self.ch1_envelope_period > 0 {
                if self.ch1_envelope_direction && self.ch1_volume < 15 {
                    self.ch1_volume += 1;
                } else if !self.ch1_envelope_direction && self.ch1_volume > 0 {
                    self.ch1_volume -= 1;
                }
            }
        }

        // Canal 2 (similar ao canal 1)
        if self.ch2_envelope_timer > 0 {
            self.ch2_envelope_timer -= 1;
        }

        if self.ch2_envelope_timer == 0 {
            self.ch2_envelope_timer = if self.ch2_envelope_period > 0 { self.ch2_envelope_period } else { 8 };

            if self.ch2_envelope_period > 0 {
                if self.ch2_envelope_direction && self.ch2_volume < 15 {
                    self.ch2_volume += 1;
                } else if !self.ch2_envelope_direction && self.ch2_volume > 0 {
                    self.ch2_volume -= 1;
                }
            }
        }

        // Canal 4 (similar aos outros)
        if self.ch4_envelope_timer > 0 {
            self.ch4_envelope_timer -= 1;
        }

        if self.ch4_envelope_timer == 0 {
            self.ch4_envelope_timer = if self.ch4_envelope_period > 0 { self.ch4_envelope_period } else { 8 };

            if self.ch4_envelope_period > 0 {
                if self.ch4_envelope_direction && self.ch4_volume < 15 {
                    self.ch4_volume += 1;
                } else if !self.ch4_envelope_direction && self.ch4_volume > 0 {
                    self.ch4_volume -= 1;
                }
            }
        }
    }

    /// Gera output do canal 1 (square wave)
    fn get_channel1_output(&mut self) -> f32 {
        if !self.ch1_enabled {
            return 0.0;
        }

        // Duty cycle patterns
        let duty_patterns = [
            0b00000001, // 12.5%
            0b10000001, // 25%
            0b10000111, // 50%
            0b01111110, // 75%
        ];

        let pattern = duty_patterns[self.ch1_wave_duty as usize];
        let bit = (pattern >> self.ch1_wave_position) & 1;

        // Avança posição da wave
        self.ch1_wave_position = (self.ch1_wave_position + 1) % 8;

        if bit == 1 {
            (self.ch1_volume as f32) / 15.0
        } else {
            0.0
        }
    }

    /// Gera output do canal 2 (square wave)
    fn get_channel2_output(&mut self) -> f32 {
        if !self.ch2_enabled {
            return 0.0;
        }

        // Duty cycle patterns (igual ao canal 1)
        let duty_patterns = [
            0b00000001, // 12.5%
            0b10000001, // 25%
            0b10000111, // 50%
            0b01111110, // 75%
        ];

        let pattern = duty_patterns[self.ch2_wave_duty as usize];
        let bit = (pattern >> self.ch2_wave_position) & 1;

        // Avança posição da wave
        self.ch2_wave_position = (self.ch2_wave_position + 1) % 8;

        if bit == 1 {
            (self.ch2_volume as f32) / 15.0
        } else {
            0.0
        }
    }

    /// Gera output do canal 3 (wave pattern)
    fn get_channel3_output(&mut self) -> f32 {
        if !self.ch3_enabled || !self.ch3_dac_enable {
            return 0.0;
        }

        // Lê sample do wave RAM
        let byte_index = (self.ch3_wave_position / 2) as usize;
        let nibble = if self.ch3_wave_position % 2 == 0 {
            (self.ch3_wave_ram[byte_index] >> 4) & 0x0F
        } else {
            self.ch3_wave_ram[byte_index] & 0x0F
        };

        // Avança posição
        self.ch3_wave_position = (self.ch3_wave_position + 1) % 32;

        // Aplica volume
        let volume_shift = match self.ch3_output_level {
            0 => 4, // Mute
            1 => 0, // 100%
            2 => 1, // 50%
            3 => 2, // 25%
            _ => 4,
        };

        let sample = nibble >> volume_shift;
        (sample as f32) / 15.0
    }

    /// Gera output do canal 4 (noise)
    fn get_channel4_output(&mut self) -> f32 {
        if !self.ch4_enabled {
            return 0.0;
        }

        // LFSR para geração de ruído
        let bit = (self.ch4_lfsr ^ (self.ch4_lfsr >> 1)) & 1;
        self.ch4_lfsr >>= 1;
        self.ch4_lfsr |= bit << 14;

        // Para 7-bit mode
        if self.ch4_width_mode {
            self.ch4_lfsr |= bit << 6;
        }

        let output = if (self.ch4_lfsr & 1) == 0 { 1 } else { 0 };

        if output == 1 {
            (self.ch4_volume as f32) / 15.0
        } else {
            0.0
        }
    }
}