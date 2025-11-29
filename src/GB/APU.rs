#![allow(non_snake_case)]

/// APU (Audio Processing Unit) do Game Boy
/// 4 canais de áudio: 2 square waves, 1 wave, 1 noise
pub struct APU {
    // === Canal 1: Square wave com sweep ===
    ch1_enabled: bool,
    ch1_sweep_period: u8,         // NR10 bits 6-4: período do sweep (0-7)
    ch1_sweep_direction: bool,    // NR10 bit 3: 0=up, 1=down
    ch1_sweep_shift: u8,          // NR10 bits 2-0: shift amount (0-7)
    ch1_wave_duty: u8,            // NR11 bits 7-6: wave duty (0-3)
    ch1_length_timer: u8,         // NR11 bits 5-0: length timer (0-63)
    ch1_envelope_initial: u8,     // NR12 bits 7-4: volume inicial (0-15)
    ch1_envelope_direction: bool, // NR12 bit 3: 0=decrease, 1=increase
    ch1_envelope_period: u8,      // NR12 bits 2-0: período do envelope (0-7)
    ch1_frequency: u16,           // NR13/NR14: frequência (0-2047)
    ch1_length_enable: bool,      // NR14 bit 6: length enable

    // === Canal 2: Square wave simples ===
    ch2_enabled: bool,
    ch2_wave_duty: u8,            // NR21 bits 7-6: wave duty (0-3)
    ch2_length_timer: u8,         // NR21 bits 5-0: length timer (0-63)
    ch2_envelope_initial: u8,     // NR22 bits 7-4: volume inicial (0-15)
    ch2_envelope_direction: bool, // NR22 bit 3: 0=decrease, 1=increase
    ch2_envelope_period: u8,      // NR22 bits 2-0: período do envelope (0-7)
    ch2_frequency: u16,           // NR23/NR24: frequência (0-2047)
    ch2_length_enable: bool,      // NR24 bit 6: length enable

    // === Canal 3: Wave pattern ===
    ch3_enabled: bool,
    ch3_dac_enable: bool,    // NR30 bit 7: DAC enable
    ch3_length_timer: u8,    // NR31: length timer (0-255)
    ch3_output_level: u8,    // NR32 bits 6-5: output level (0-3)
    ch3_frequency: u16,      // NR33/NR34: frequência (0-2047)
    ch3_length_enable: bool, // NR34 bit 6: length enable
    ch3_wave_ram: [u8; 16],  // Wave RAM (0xFF30-0xFF3F): 32 samples de 4 bits

    // === Canal 4: Noise ===
    ch4_enabled: bool,
    ch4_length_timer: u8,         // NR41 bits 5-0: length timer (0-63)
    ch4_envelope_initial: u8,     // NR42 bits 7-4: volume inicial (0-15)
    ch4_envelope_direction: bool, // NR42 bit 3: 0=decrease, 1=increase
    ch4_envelope_period: u8,      // NR42 bits 2-0: período do envelope (0-7)
    ch4_clock_shift: u8,          // NR43 bits 7-4: clock shift (0-15)
    ch4_width_mode: bool,         // NR43 bit 3: 0=15bit, 1=7bit
    ch4_divisor_code: u8,         // NR43 bits 2-0: divisor code (0-7)
    ch4_length_enable: bool,      // NR44 bit 6: length enable

    // === Controle geral ===
    left_volume: u8,        // NR50 bits 6-4: volume esquerdo (0-7)
    right_volume: u8,       // NR50 bits 2-0: volume direito (0-7)
    vin_left_enable: bool,  // NR50 bit 7: VIN left enable
    vin_right_enable: bool, // NR50 bit 3: VIN right enable

    // NR51: Sound panning
    ch1_left: bool,
    ch1_right: bool,
    ch2_left: bool,
    ch2_right: bool,
    ch3_left: bool,
    ch3_right: bool,
    ch4_left: bool,
    ch4_right: bool,

    sound_enable: bool, // NR52 bit 7: master sound enable

    // === Estado interno ===
    frame_sequencer: u8, // Frame sequencer (0-7) para length/envelope/sweep
    frame_sequencer_div_bit: bool, // Estado anterior do bit 13 do DIV para edge detection

    // Estados dos canais
    ch1_volume: u8,            // Volume atual do canal 1
    ch1_frequency_shadow: u16, // Shadow register da frequência (para sweep)
    ch1_wave_position: u8,     // Posição na wave duty
    ch1_envelope_timer: u8,    // Timer do envelope
    ch1_length_counter: u8,    // Contador de length
    ch1_sweep_timer: u8,        // Timer do sweep
    ch1_sweep_enabled: bool,    // Sweep habilitado
    ch1_sweep_negate_used: bool, // True se negate foi usado em cálculo (quirk)

    ch2_volume: u8,         // Volume atual do canal 2
    ch2_wave_position: u8,  // Posição na wave duty
    ch2_envelope_timer: u8, // Timer do envelope
    ch2_length_counter: u8, // Contador de length

    ch3_wave_position: u8,   // Posição no wave pattern (0-31)
    ch3_length_counter: u16, // Contador de length (0-255)

    ch4_volume: u8,         // Volume atual do canal 4
    ch4_envelope_timer: u8, // Timer do envelope
    ch4_length_counter: u8, // Contador de length
    ch4_lfsr: u16,          // Linear Feedback Shift Register para noise

    // Timers de frequência
    ch1_frequency_timer: u32, // Timer de frequência do canal 1
    ch2_frequency_timer: u32, // Timer de frequência do canal 2
    ch3_frequency_timer: u32, // Timer de frequência do canal 3
    ch4_frequency_timer: u32, // Timer de frequência do canal 4

    // Divisor de frequência para timers (4MHz -> 1MHz)
    frequency_divider: u8,
}

const DUTY_TABLE: [[u8; 8]; 4] = [
    // 12.5%
    [0, 1, 0, 0, 0, 0, 0, 0],
    // 25%
    [0, 1, 1, 0, 0, 0, 0, 0],
    // 50%
    [0, 1, 1, 1, 1, 0, 0, 0],
    // 75%
    [1, 0, 0, 1, 1, 1, 1, 1],
];


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

            // Canal 2
            ch2_enabled: false,
            ch2_wave_duty: 0,
            ch2_length_timer: 0,
            ch2_envelope_initial: 0,
            ch2_envelope_direction: false,
            ch2_envelope_period: 0,
            ch2_frequency: 0,
            ch2_length_enable: false,

            // Canal 3
            ch3_enabled: false,
            ch3_dac_enable: false,
            ch3_length_timer: 0,
            ch3_output_level: 0,
            ch3_frequency: 0,
            ch3_length_enable: false,
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

            // Controle geral – pós-boot
            left_volume: 7,  // NR50
            right_volume: 7, // NR50
            vin_left_enable: false,
            vin_right_enable: false,
            ch1_left: true, // NR51
            ch1_right: true,
            ch2_left: true,
            ch2_right: true,
            ch3_left: true,
            ch3_right: true,
            ch4_left: true,
            ch4_right: true,
            sound_enable: true, // NR52

            // Estado interno
            frame_sequencer: 0,
            frame_sequencer_div_bit: false,
            ch1_volume: 0,
            ch1_frequency_shadow: 0,
            ch1_wave_position: 0,
            ch1_envelope_timer: 0,
            ch1_length_counter: 0,
            ch1_sweep_timer: 0,
            ch1_sweep_enabled: false,
            ch1_sweep_negate_used: false,
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

            // Timers de frequência
            ch1_frequency_timer: 0,
            ch2_frequency_timer: 0,
            ch3_frequency_timer: 0,
            ch4_frequency_timer: 0,

            // Divisor de frequência
            frequency_divider: 0,
        }
    }

    /// Clock APU - chamado a cada ciclo de CPU (4MHz)
    /// div_counter é o contador interno de 16 bits do DIV
    pub fn tick(&mut self, div_counter: u16) {
        // Frame sequencer é clockado pelo bit 12 do DIV (falling edge)
        // Bit 12 alterna a cada 4096 ciclos, falling edge a cada 8192 = 512Hz
        let div_bit = (div_counter >> 12) & 1 != 0;
        if self.frame_sequencer_div_bit && !div_bit {
            // Falling edge do bit 13 - clocka frame sequencer
            // Frame sequencer roda SEMPRE, mesmo com APU desligada
            // Os length counters continuam decrementando
            self.step_frame_sequencer();
        }
        self.frame_sequencer_div_bit = div_bit;

        if !self.sound_enable {
            return;
        }

        // Timers de frequência dos canais (4MHz -> 1MHz)
        self.frequency_divider += 1;
        if self.frequency_divider >= 4 {
            self.frequency_divider = 0;
            self.update_channel_timers();
        }
    }

    /// Retorna true se o próximo step do frame sequencer vai clockar length counters
    /// Isso acontece nos steps 0, 2, 4, 6 (steps pares)
    fn is_length_clock_next(&self) -> bool {
        // Length é clockado em steps pares (0, 2, 4, 6)
        self.frame_sequencer % 2 == 0
    }

    /// Frame sequencer (512Hz)
    fn step_frame_sequencer(&mut self) {
        self.frame_sequencer = (self.frame_sequencer + 1) % 8;

        // Length counters rodam SEMPRE (mesmo com APU desligada)
        match self.frame_sequencer {
            0 | 2 | 4 | 6 => {
                self.step_length_counters();
            }
            _ => {}
        }

        // Sweep e envelopes só rodam com APU ligada
        if self.sound_enable {
            match self.frame_sequencer {
                2 | 6 => {
                    self.step_sweep();
                }
                7 => {
                    self.step_envelopes();
                }
                _ => {}
            }
        }
    }

    /// Gera sample de áudio (chamado a 44.1kHz)
    pub fn generate_sample(&mut self) -> (f32, f32) {
        // Frame sequencer agora é executado em tick() baseado em ciclos de CPU

        if !self.sound_enable {
            return (0.0, 0.0);
        }

        let mut left_sample = 0.0;
        let mut right_sample = 0.0;

        // Canal 1: Square Wave (usando wave_position de hardware)
        if self.ch1_enabled && self.ch1_volume > 0 {
            let duty = (self.ch1_wave_duty & 0x03) as usize;
            let step = (self.ch1_wave_position & 0x07) as usize;
            let bit = DUTY_TABLE[duty][step];

            let wave_out = if bit != 0 { 1.0 } else { -1.0 };
            let volume = self.ch1_volume as f32 / 15.0;
            let final_output = wave_out * volume * 0.25;

            if self.ch1_left {
                left_sample += final_output;
            }
            if self.ch1_right {
                right_sample += final_output;
            }
        }

        // Canal 2: Square Wave (usando wave_position)
        if self.ch2_enabled && self.ch2_volume > 0 {
            let duty = (self.ch2_wave_duty & 0x03) as usize;
            let step = (self.ch2_wave_position & 0x07) as usize;
            let bit = DUTY_TABLE[duty][step];

            let wave_out = if bit != 0 { 1.0 } else { -1.0 };
            let volume = self.ch2_volume as f32 / 15.0;
            let final_output = wave_out * volume * 0.25;

            if self.ch2_left {
                left_sample += final_output;
            }
            if self.ch2_right {
                right_sample += final_output;
            }
        }

        // Canal 3: Wave Channel
        if self.ch3_enabled && self.ch3_dac_enable {
            let wave_output = self.generate_wave();
            let final_output = wave_output * 0.25;

            if self.ch3_left {
                left_sample += final_output;
            }
            if self.ch3_right {
                right_sample += final_output;
            }
        }

        // Canal 4: Noise Channel
        if self.ch4_enabled && self.ch4_volume > 0 {
            let noise_output = self.generate_noise();
            let volume = self.ch4_volume as f32 / 15.0;
            let final_output = noise_output * volume * 0.25;

            if self.ch4_left {
                left_sample += final_output;
            }
            if self.ch4_right {
                right_sample += final_output;
            }
        }

        // Master volume simplificado (0-7 -> 0.0-1.0)
        let left_master_vol = self.left_volume as f32 / 7.0;
        let right_master_vol = self.right_volume as f32 / 7.0;

        left_sample *= left_master_vol;
        right_sample *= right_master_vol;

        // Clamp final para evitar distorção
        left_sample = left_sample.clamp(-1.0, 1.0);
        right_sample = right_sample.clamp(-1.0, 1.0);

        (left_sample, right_sample)
    }

    /// === FASE 4: Geração de Noise usando LFSR ===
    fn generate_noise(&mut self) -> f32 {
        // LFSR é avançado apenas via update_channel_timers()
        // Aqui apenas lemos o bit atual do LFSR

        // Gerar output baseado no bit 0 do LFSR
        if (self.ch4_lfsr & 1) == 0 { 1.0 } else { -1.0 }
    }

    /// === FASE 5: Geração de Wave usando Wave RAM ===
    fn generate_wave(&mut self) -> f32 {
        // Wave position é avançada apenas via update_channel_timers()
        // Aqui apenas lemos o sample da posição atual

        // Ler sample da Wave RAM (32 samples de 4 bits)
        let byte_index = (self.ch3_wave_position / 2) as usize;
        let nibble = if self.ch3_wave_position & 1 == 0 {
            // Nibble superior (bits 7-4)
            (self.ch3_wave_ram[byte_index] >> 4) & 0x0F
        } else {
            // Nibble inferior (bits 3-0)
            self.ch3_wave_ram[byte_index] & 0x0F
        };

        // Converter 4-bit sample para float (-1.0 a 1.0)
        let raw_sample = (nibble as f32 / 7.5) - 1.0;

        // Aplicar volume shift (NR32)
        let volume_shift = match self.ch3_output_level {
            0 => 0.0,  // Mute
            1 => 1.0,  // 100% volume
            2 => 0.5,  // 50% volume
            3 => 0.25, // 25% volume
            _ => 0.0,
        };

        raw_sample * volume_shift
    }

    /// Lê um registrador do APU
    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            // Canal 1
            0xFF10 => {
                // NR10: Sweep (bit 7 não usado, sempre 1)
                0x80 | (self.ch1_sweep_period << 4)
                    | (if self.ch1_sweep_direction { 0x08 } else { 0x00 })
                    | self.ch1_sweep_shift
            }
            0xFF11 => {
                // NR11: Wave duty + length timer (só duty é readable)
                (self.ch1_wave_duty << 6) | 0x3F
            }
            0xFF12 => {
                // NR12: Envelope
                (self.ch1_envelope_initial << 4)
                    | (if self.ch1_envelope_direction {
                        0x08
                    } else {
                        0x00
                    })
                    | self.ch1_envelope_period
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
                (self.ch2_envelope_initial << 4)
                    | (if self.ch2_envelope_direction {
                        0x08
                    } else {
                        0x00
                    })
                    | self.ch2_envelope_period
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
                (self.ch4_envelope_initial << 4)
                    | (if self.ch4_envelope_direction {
                        0x08
                    } else {
                        0x00
                    })
                    | self.ch4_envelope_period
            }
            0xFF22 => {
                // NR43: Noise parameters
                (self.ch4_clock_shift << 4)
                    | (if self.ch4_width_mode { 0x08 } else { 0x00 })
                    | self.ch4_divisor_code
            }
            0xFF23 => {
                // NR44: Control (só length enable é readable)
                (if self.ch4_length_enable { 0x40 } else { 0x00 }) | 0xBF
            }

            // Controle geral
            0xFF24 => {
                // NR50: Master volume
                (if self.vin_left_enable { 0x80 } else { 0x00 })
                    | (self.left_volume << 4)
                    | (if self.vin_right_enable { 0x08 } else { 0x00 })
                    | self.right_volume
            }
            0xFF25 => {
                // NR51: Sound panning
                (if self.ch4_left { 0x80 } else { 0x00 })
                    | (if self.ch3_left { 0x40 } else { 0x00 })
                    | (if self.ch2_left { 0x20 } else { 0x00 })
                    | (if self.ch1_left { 0x10 } else { 0x00 })
                    | (if self.ch4_right { 0x08 } else { 0x00 })
                    | (if self.ch3_right { 0x04 } else { 0x00 })
                    | (if self.ch2_right { 0x02 } else { 0x00 })
                    | (if self.ch1_right { 0x01 } else { 0x00 })
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

            // Wave RAM - HARDWARE QUIRK: durante playback, retorna byte sendo acessado
            0xFF30..=0xFF3F => {
                if self.ch3_enabled && self.ch3_dac_enable {
                    // Durante playback, retorna o byte que o canal está acessando
                    let byte_index = (self.ch3_wave_position / 2) as usize;
                    self.ch3_wave_ram[byte_index]
                } else {
                    // Normal: Wave RAM acessível
                    self.ch3_wave_ram[(address - 0xFF30) as usize]
                }
            }

            _ => 0xFF, // Registradores não implementados
        }
    }

    /// Escreve em um registrador do APU
    pub fn write_register(&mut self, address: u16, value: u8) {
        // No DMG, quando o som está desabilitado (NR52 bit 7 = 0),
        // escritas em NR10-NR51 são ignoradas (exceto NR52, NR41 e Wave RAM)
        // NR41 (0xFF20) é especial: pode ser escrito mesmo com APU desligada
        if !self.sound_enable
            && address != 0xFF26
            && address != 0xFF20
            && !(0xFF30..=0xFF3F).contains(&address)
        {
            return;
        }

        match address {
            // Canal 1
            0xFF10 => {
                // NR10: Sweep
                let old_negate = self.ch1_sweep_direction;
                self.ch1_sweep_period = (value >> 4) & 0x07;
                self.ch1_sweep_direction = (value & 0x08) != 0;
                self.ch1_sweep_shift = value & 0x07;

                // Quirk: se estava em negate, fez cálculo, e agora mudou para add -> desabilita
                if old_negate && !self.ch1_sweep_direction && self.ch1_sweep_negate_used {
                    self.ch1_enabled = false;
                }
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

                // Extra length clocking: habilitando length na primeira metade do frame sequencer
                let new_length_enable = (value & 0x40) != 0;
                if new_length_enable && !self.ch1_length_enable && self.is_length_clock_next() {
                    if self.ch1_length_counter > 0 {
                        self.ch1_length_counter -= 1;
                        if self.ch1_length_counter == 0 && (value & 0x80) == 0 {
                            self.ch1_enabled = false;
                        }
                    }
                }
                self.ch1_length_enable = new_length_enable;

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

                // Extra length clocking
                let new_length_enable = (value & 0x40) != 0;
                if new_length_enable && !self.ch2_length_enable && self.is_length_clock_next() {
                    if self.ch2_length_counter > 0 {
                        self.ch2_length_counter -= 1;
                        if self.ch2_length_counter == 0 && (value & 0x80) == 0 {
                            self.ch2_enabled = false;
                        }
                    }
                }
                self.ch2_length_enable = new_length_enable;

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

                // Extra length clocking
                let new_length_enable = (value & 0x40) != 0;
                if new_length_enable && !self.ch3_length_enable && self.is_length_clock_next() {
                    if self.ch3_length_counter > 0 {
                        self.ch3_length_counter -= 1;
                        if self.ch3_length_counter == 0 && (value & 0x80) == 0 {
                            self.ch3_enabled = false;
                        }
                    }
                }
                self.ch3_length_enable = new_length_enable;

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
                // Extra length clocking
                let new_length_enable = (value & 0x40) != 0;
                if new_length_enable && !self.ch4_length_enable && self.is_length_clock_next() {
                    if self.ch4_length_counter > 0 {
                        self.ch4_length_counter -= 1;
                        if self.ch4_length_counter == 0 && (value & 0x80) == 0 {
                            self.ch4_enabled = false;
                        }
                    }
                }
                self.ch4_length_enable = new_length_enable;

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

                // Se o som foi habilitado, reseta o frame sequencer
                // O frame sequencer começa em 7, então o primeiro step será 0
                if !old_enable && self.sound_enable {
                    self.frame_sequencer = 7;
                }
            }

            // Wave RAM - HARDWARE QUIRK: write bloqueado durante playback
            0xFF30..=0xFF3F => {
                if !(self.ch3_enabled && self.ch3_dac_enable) {
                    // Só permite write quando canal 3 não está tocando
                    self.ch3_wave_ram[(address - 0xFF30) as usize] = value;
                }
                // Durante playback: write ignorado (quirk do hardware)
            }

            _ => {} // Registradores não implementados
        }
    }

    /// Trigger do canal 1
    fn trigger_channel1(&mut self) {
        self.ch1_enabled = true;

        // Se length counter era 0, seta para máximo
        // E se length_enable + primeira metade do frame seq, faz clock extra
        if self.ch1_length_counter == 0 {
            self.ch1_length_counter = 64;
            if self.ch1_length_enable && self.is_length_clock_next() {
                self.ch1_length_counter -= 1;
            }
        }

        self.ch1_envelope_timer = self.ch1_envelope_period;
        self.ch1_volume = self.ch1_envelope_initial;

        // Inicializar sweep
        self.ch1_frequency_shadow = self.ch1_frequency;
        self.ch1_sweep_timer = if self.ch1_sweep_period > 0 {
            self.ch1_sweep_period
        } else {
            8
        };
        self.ch1_sweep_enabled = self.ch1_sweep_period > 0 || self.ch1_sweep_shift > 0;
        self.ch1_sweep_negate_used = false; // Reset da flag no trigger

        // Se shift > 0, calcula frequência imediatamente (overflow check)
        if self.ch1_sweep_shift > 0 {
            let new_freq = self.calculate_sweep_frequency();
            if new_freq > 2047 {
                self.ch1_enabled = false;
            }
        }

        // Inicializar timer de frequência
        self.ch1_frequency_timer = 2048 - self.ch1_frequency as u32;
        self.ch1_wave_position = 0;

        // Desabilitar se DAC está off
        if self.ch1_envelope_initial == 0 && !self.ch1_envelope_direction {
            self.ch1_enabled = false;
        }
    }

    /// Trigger do canal 2
    fn trigger_channel2(&mut self) {
        self.ch2_enabled = true;

        if self.ch2_length_counter == 0 {
            self.ch2_length_counter = 64;
            if self.ch2_length_enable && self.is_length_clock_next() {
                self.ch2_length_counter -= 1;
            }
        }

        self.ch2_envelope_timer = self.ch2_envelope_period;
        self.ch2_volume = self.ch2_envelope_initial;

        // Inicializar timer de frequência
        self.ch2_frequency_timer = 2048 - self.ch2_frequency as u32;
        self.ch2_wave_position = 0;

        // Desabilitar se DAC está off
        if self.ch2_envelope_initial == 0 && !self.ch2_envelope_direction {
            self.ch2_enabled = false;
        }
    }

    /// Trigger do canal 3
    fn trigger_channel3(&mut self) {
        self.ch3_enabled = self.ch3_dac_enable;

        if self.ch3_length_counter == 0 {
            self.ch3_length_counter = 256;
            if self.ch3_length_enable && self.is_length_clock_next() {
                self.ch3_length_counter -= 1;
            }
        }

        // Inicializar timer de frequência
        self.ch3_frequency_timer = (2048 - self.ch3_frequency as u32) / 2;
        self.ch3_wave_position = 0;
    }

    /// Trigger do canal 4
    fn trigger_channel4(&mut self) {
        self.ch4_enabled = true;

        if self.ch4_length_counter == 0 {
            self.ch4_length_counter = 64;
            if self.ch4_length_enable && self.is_length_clock_next() {
                self.ch4_length_counter -= 1;
            }
        }
        self.ch4_envelope_timer = self.ch4_envelope_period;
        self.ch4_volume = self.ch4_envelope_initial;
        self.ch4_lfsr = 0x7FFF;

        // Inicializar timer de frequência usando tabela oficial DMG
        const NOISE_DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];
        let divisor = NOISE_DIVISORS[self.ch4_divisor_code as usize] as u32;
        self.ch4_frequency_timer = divisor << self.ch4_clock_shift;

        // Desabilitar se DAC está off
        if self.ch4_envelope_initial == 0 && !self.ch4_envelope_direction {
            self.ch4_enabled = false;
        }
    }

    /// Desabilita todos os canais quando o som é desligado
    fn disable_all_channels(&mut self) {
        self.ch1_enabled = false;
        self.ch2_enabled = false;
        self.ch3_enabled = false;
        self.ch4_enabled = false;

        // Limpar registradores de todos os canais
        // NÃO limpa: length timers (NRx1) e wave RAM - quirk do hardware DMG

        // Canal 1 (mantém ch1_length_timer)
        self.ch1_sweep_period = 0;
        self.ch1_sweep_direction = false;
        self.ch1_sweep_shift = 0;
        self.ch1_wave_duty = 0;
        // self.ch1_length_timer = 0; // NR11 bits 0-5 preservado
        self.ch1_envelope_initial = 0;
        self.ch1_envelope_direction = false;
        self.ch1_envelope_period = 0;
        self.ch1_frequency = 0;
        self.ch1_length_enable = false;

        // Canal 2 (mantém ch2_length_timer)
        self.ch2_wave_duty = 0;
        // self.ch2_length_timer = 0; // NR21 bits 0-5 preservado
        self.ch2_envelope_initial = 0;
        self.ch2_envelope_direction = false;
        self.ch2_envelope_period = 0;
        self.ch2_frequency = 0;
        self.ch2_length_enable = false;

        // Canal 3 (mantém ch3_length_timer e wave RAM)
        self.ch3_dac_enable = false;
        // self.ch3_length_timer = 0; // NR31 preservado
        self.ch3_output_level = 0;
        self.ch3_frequency = 0;
        self.ch3_length_enable = false;

        // Canal 4 (mantém ch4_length_timer e ch4_length_counter)
        // self.ch4_length_timer = 0; // NR41 bits 0-5 preservado
        // ch4_length_counter também preservado
        self.ch4_envelope_initial = 0;
        self.ch4_envelope_direction = false;
        self.ch4_envelope_period = 0;
        self.ch4_clock_shift = 0;
        self.ch4_width_mode = false;
        self.ch4_divisor_code = 0;
        self.ch4_length_enable = false;

        // Reset controles gerais
        self.left_volume = 0;
        self.right_volume = 0;
        self.vin_left_enable = false;
        self.vin_right_enable = false;
        self.ch1_left = false;
        self.ch1_right = false;
        self.ch2_left = false;
        self.ch2_right = false;
        self.ch3_left = false;
        self.ch3_right = false;
        self.ch4_left = false;
        self.ch4_right = false;
    }

    /// Atualiza timers dos canais
    fn update_channel_timers(&mut self) {
        // Atualiza timers de frequência dos canais (executado a cada ciclo APU)

        // Canal 1 - Timer de frequência
        if self.ch1_enabled {
            if self.ch1_frequency_timer > 0 {
                self.ch1_frequency_timer -= 1;
            } else {
                // Reset timer baseado na frequência (agora já em 1MHz)
                self.ch1_frequency_timer = 2048 - self.ch1_frequency as u32;
                self.ch1_wave_position = (self.ch1_wave_position + 1) % 8;
            }
        }

        // Canal 2 - Timer de frequência
        if self.ch2_enabled {
            if self.ch2_frequency_timer > 0 {
                self.ch2_frequency_timer -= 1;
            } else {
                self.ch2_frequency_timer = 2048 - self.ch2_frequency as u32;
                self.ch2_wave_position = (self.ch2_wave_position + 1) % 8;
            }
        }

        // Canal 3 - Timer de frequência (2x mais rápido que canais 1/2)
        // No hardware real: período do canal 3 = (2048 - freq) * 2 / 4.194304MHz.
        // Como update_channel_timers() roda a 4MHz/4 = ~1MHz,
        // usar (2048 - freq) / 2 dá a mesma frequência final.
        if self.ch3_enabled {
            if self.ch3_frequency_timer > 0 {
                self.ch3_frequency_timer -= 1;
            } else {
                self.ch3_frequency_timer = (2048 - self.ch3_frequency as u32) / 2;
                self.ch3_wave_position = (self.ch3_wave_position + 1) % 32;
            }
        } // Canal 4 - Timer de frequência (mais complexo)
        if self.ch4_enabled {
            if self.ch4_frequency_timer > 0 {
                self.ch4_frequency_timer -= 1;
            } else {
                // Timer baseado no divisor e clock shift
                const NOISE_DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];
                let divisor = NOISE_DIVISORS[self.ch4_divisor_code as usize] as u32;
                self.ch4_frequency_timer = divisor << self.ch4_clock_shift;

                // Atualiza LFSR
                let bit = (self.ch4_lfsr ^ (self.ch4_lfsr >> 1)) & 1;
                self.ch4_lfsr >>= 1;
                self.ch4_lfsr |= bit << 14;

                if self.ch4_width_mode {
                    // limpa o bit 6, depois escreve o novo bit
                    self.ch4_lfsr = (self.ch4_lfsr & !(1 << 6)) | (bit << 6);
                }
            }
        }
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

    /// Step do sweep (apenas canal 1) com hardware precision
    fn step_sweep(&mut self) {
        if !self.ch1_sweep_enabled {
            return;
        }

        if self.ch1_sweep_timer > 0 {
            self.ch1_sweep_timer -= 1;
        }

        if self.ch1_sweep_timer == 0 {
            self.ch1_sweep_timer = if self.ch1_sweep_period > 0 {
                self.ch1_sweep_period
            } else {
                8
            };

            if self.ch1_sweep_enabled && self.ch1_sweep_period > 0 {
                let new_frequency = self.calculate_sweep_frequency();

                // HARDWARE PRECISION: overflow check ANTES de aplicar
                if new_frequency > 2047 {
                    self.ch1_enabled = false; // Disable channel imediatamente
                } else if self.ch1_sweep_shift > 0 {
                    self.ch1_frequency = new_frequency;
                    self.ch1_frequency_shadow = new_frequency;

                    // HARDWARE PRECISION: segundo overflow check
                    let next_freq = self.calculate_sweep_frequency();
                    if next_freq > 2047 {
                        self.ch1_enabled = false;
                    }
                }
            }
        }
    }

    /// Calcula nova frequência para sweep com hardware precision
    fn calculate_sweep_frequency(&mut self) -> u16 {
        let freq_change = self.ch1_frequency_shadow >> self.ch1_sweep_shift;
        if self.ch1_sweep_direction {
            // HARDWARE PRECISION: subtração usa complemento de um
            // Marca que negate foi usado (quirk do hardware)
            self.ch1_sweep_negate_used = true;
            self.ch1_frequency_shadow.wrapping_sub(freq_change)
        } else {
            // HARDWARE PRECISION: pode overflow além de 2047
            self.ch1_frequency_shadow.wrapping_add(freq_change)
        }
    }

    /// Step dos envelopes com hardware edge cases
    fn step_envelopes(&mut self) {
        // Canal 1
        if self.ch1_envelope_timer > 0 {
            self.ch1_envelope_timer -= 1;
        }

        if self.ch1_envelope_timer == 0 {
            self.ch1_envelope_timer = if self.ch1_envelope_period > 0 {
                self.ch1_envelope_period
            } else {
                8
            };

            if self.ch1_envelope_period > 0 {
                if self.ch1_envelope_direction && self.ch1_volume < 15 {
                    self.ch1_volume += 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge máximo
                    if self.ch1_volume == 15 {
                        self.ch1_envelope_period = 0;
                    }
                } else if !self.ch1_envelope_direction && self.ch1_volume > 0 {
                    self.ch1_volume -= 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge zero
                    if self.ch1_volume == 0 {
                        self.ch1_envelope_period = 0;
                    }
                }
            }
        }

        // Canal 2 (com mesmas edge cases)
        if self.ch2_envelope_timer > 0 {
            self.ch2_envelope_timer -= 1;
        }

        if self.ch2_envelope_timer == 0 {
            self.ch2_envelope_timer = if self.ch2_envelope_period > 0 {
                self.ch2_envelope_period
            } else {
                8
            };

            if self.ch2_envelope_period > 0 {
                if self.ch2_envelope_direction && self.ch2_volume < 15 {
                    self.ch2_volume += 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge máximo
                    if self.ch2_volume == 15 {
                        self.ch2_envelope_period = 0;
                    }
                } else if !self.ch2_envelope_direction && self.ch2_volume > 0 {
                    self.ch2_volume -= 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge zero
                    if self.ch2_volume == 0 {
                        self.ch2_envelope_period = 0;
                    }
                }
            }
        }

        // Canal 4 (com mesmas edge cases)
        if self.ch4_envelope_timer > 0 {
            self.ch4_envelope_timer -= 1;
        }

        if self.ch4_envelope_timer == 0 {
            self.ch4_envelope_timer = if self.ch4_envelope_period > 0 {
                self.ch4_envelope_period
            } else {
                8
            };

            if self.ch4_envelope_period > 0 {
                if self.ch4_envelope_direction && self.ch4_volume < 15 {
                    self.ch4_volume += 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge máximo
                    if self.ch4_volume == 15 {
                        self.ch4_envelope_period = 0;
                    }
                } else if !self.ch4_envelope_direction && self.ch4_volume > 0 {
                    self.ch4_volume -= 1;
                    // HARDWARE EDGE CASE: para envelope quando atinge zero
                    if self.ch4_volume == 0 {
                        self.ch4_envelope_period = 0;
                    }
                }
            }
        }
    }
}
