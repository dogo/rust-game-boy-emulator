use crate::GB::PPU;
use crate::GB::APU;

pub struct RAM {
    // todas as 65.536 posições endereçáveis
    memory: [u8; 65536],
    // ROM completa do cartucho (para mapeamento por banco)
    rom: Vec<u8>,
    cart_type: u8,
    // Estado MBC (MBC1 e MBC3)
    rom_bank: u8,       // banco ROM selecionado para 0x4000..=0x7FFF
    ram_enabled: bool,  // enable RAM/RTC (0x0000..=0x1FFF)
    ram_bank: u8,       // seleção de banco de RAM (MBC1: 0-3, MBC3: 0-3 ou RTC 8-C)
    // MBC1 específico
    mbc1_bank_reg1: u8, // registrador de banco 1 (bits 0-4 do ROM bank)
    mbc1_bank_reg2: u8, // registrador de banco 2 (bits 5-6 do ROM bank ou RAM bank)
    mbc1_mode: u8,      // 0=ROM banking mode, 1=RAM banking mode
    // MBC5 específico
    mbc5_rom_bank_low: u8,  // bits 0-7 do ROM bank (0x2000-0x2FFF)
    mbc5_rom_bank_high: u8, // bit 8 do ROM bank (0x3000-0x3FFF)
    mbc5_ram_bank: u8,      // bits 0-3 = RAM bank, bit 3 = rumble
    // RAM externa do cartucho (MBC3: até 4 bancos de 8KB = 32KB)
    cart_ram: Vec<u8>,
    // RTC (Real Time Clock) do MBC3
    rtc_s: u8,          // segundos (0..59)
    rtc_m: u8,          // minutos (0..59)
    rtc_h: u8,          // horas (0..23)
    rtc_dl: u8,         // dia baixo (bits 0..7 de day counter)
    rtc_dh: u8,         // dia alto + flags (bit 0 = bit 8 de day, bit 6 = halt, bit 7 = carry)
    rtc_latched_s: u8,
    rtc_latched_m: u8,
    rtc_latched_h: u8,
    rtc_latched_dl: u8,
    rtc_latched_dh: u8,
    rtc_latch_state: u8, // estado do latch (0x00 ou 0x01 para detectar transição 0->1)
    // Estado interno do temporizador
    div_counter: u16,        // contador divisor interno de 16 bits (incrementa a cada ciclo da CPU)
    timer_last_signal: bool, // último nível do sinal do timer (enable && bit selecionado de div_counter)
    tima_reload_delay: u8,   // se >0, contagem regressiva para recarregar TIMA com TMA e solicitar interrupção
    // Joypad (0xFF00)
    joypad_select_dpad: bool,   // true = selecionou leitura de direções (D-pad)
    joypad_select_buttons: bool, // true = selecionou leitura de botões de ação
    joypad_dpad: u8,            // estado do D-pad: bit0=Right, bit1=Left, bit2=Up, bit3=Down (0=pressed, 1=released)
    joypad_buttons: u8,         // estado de ação: bit0=A, bit1=B, bit2=Select, bit3=Start (0=pressed, 1=released)
    // PPU (Picture Processing Unit)
    pub ppu: PPU::PPU,
    // APU (Audio Processing Unit)
    pub apu: APU::APU,
    // Controle de trace
    pub trace_enabled: bool,    // se true, emite logs de operações (MBC, timer, joypad)
}

impl RAM {
    pub fn new() -> Self {
        RAM {
            memory: [0; 65536],
            rom: Vec::new(),
            cart_type: 0x00,
            rom_bank: 1,
            ram_enabled: false,
            ram_bank: 0,
            mbc1_bank_reg1: 1,  // banco 1 por padrão (0 é mapeado para 1)
            mbc1_bank_reg2: 0,  // banco alto/RAM 0 por padrão
            mbc1_mode: 0,       // ROM banking mode por padrão
            mbc5_rom_bank_low: 1,  // banco ROM baixo (1 por padrão)
            mbc5_rom_bank_high: 0, // banco ROM alto (bit 8)
            mbc5_ram_bank: 0,      // banco RAM + rumble
            cart_ram: vec![0; 128 * 1024], // 128KB = 16 bancos de 8KB (MBC5 max)
            rtc_s: 0,
            rtc_m: 0,
            rtc_h: 0,
            rtc_dl: 0,
            rtc_dh: 0,
            rtc_latched_s: 0,
            rtc_latched_m: 0,
            rtc_latched_h: 0,
            rtc_latched_dl: 0,
            rtc_latched_dh: 0,
            rtc_latch_state: 0,
            div_counter: 0,
            timer_last_signal: false,
            tima_reload_delay: 0,
            joypad_select_dpad: false,
            joypad_select_buttons: false,
            joypad_dpad: 0x0F,      // todos soltos (1111)
            joypad_buttons: 0x0F,   // todos soltos (1111)
            ppu: PPU::PPU::new(),
            apu: APU::APU::new(),
            trace_enabled: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let addr = address as usize;
        // Mapeamento de ROM (0x0000..=0x7FFF)
        if addr < 0x8000 {
            if self.is_mbc1() {
                if addr < 0x4000 {
                    // Banco 0 fixo (ou banco alto em modo RAM banking)
                    let bank = if self.mbc1_mode == 1 {
                        // Modo RAM banking: pode mapear bancos altos para 0x0000-0x3FFF
                        ((self.mbc1_bank_reg2 & 0x03) << 5) as usize
                    } else {
                        0 // Modo ROM banking: sempre banco 0
                    };
                    let idx = bank * 0x4000 + addr;
                    return self.rom_get(idx);
                } else {
                    // Banco switchable (0x4000-0x7FFF)
                    let off = addr - 0x4000;
                    let bank = self.mbc1_effective_rom_bank() as usize;
                    let idx = bank * 0x4000 + off;
                    return self.rom_get(idx);
                }
            } else if self.is_mbc3() {
                if addr < 0x4000 {
                    return self.rom_get(addr);
                } else {
                    let off = addr - 0x4000;
                    let bank = (self.rom_bank as usize).max(1);
                    let idx = bank * 0x4000 + off;
                    return self.rom_get(idx);
                }
            } else if self.is_mbc5() {
                if addr < 0x4000 {
                    return self.rom_get(addr); // Banco 0 fixo
                } else {
                    let off = addr - 0x4000;
                    let bank = self.mbc5_effective_rom_bank() as usize;
                    let idx = bank * 0x4000 + off;
                    return self.rom_get(idx);
                }
            } else {
                // ROM ONLY / sem MBC: banco 0 fixo
                return self.rom_get(addr);
            }
        }
        // VRAM (0x8000-0x9FFF)
        if address >= 0x8000 && address < 0xA000 {
            return self.ppu.read_vram(address);
        }
        // Joypad (0xFF00 - P1/JOYP)
        if addr == 0xFF00 {
            // Bits 7-6: não usados (retornam 1)
            // Bit 5: seleção de botões de ação (0=selecionado)
            // Bit 4: seleção de D-pad (0=selecionado)
            // Bits 3-0: estado dos botões (0=pressed, 1=released)
            let mut val = 0xC0; // bits 7-6 sempre 1

            // Refletir seleção nos bits 5-4
            if !self.joypad_select_buttons {
                val |= 0x20;
            }
            if !self.joypad_select_dpad {
                val |= 0x10;
            }

            // Retornar estado dos botões conforme seleção
            // Se ambos selecionados, retorna AND (qualquer botão pressionado em qualquer grupo)
            // Se nenhum selecionado, retorna 0x0F (todos soltos)
            let button_bits = if self.joypad_select_dpad && self.joypad_select_buttons {
                // Ambos selecionados: AND lógico (se qualquer estiver pressed=0, resultado é 0)
                self.joypad_dpad & self.joypad_buttons
            } else if self.joypad_select_dpad {
                // Apenas D-pad selecionado
                self.joypad_dpad
            } else if self.joypad_select_buttons {
                // Apenas botões de ação selecionados
                self.joypad_buttons
            } else {
                // Nenhum selecionado
                0x0F
            };

            val |= button_bits & 0x0F;
            return val;
        }        // Mapeamento de RAM externa do cartucho (0xA000..=0xBFFF)
        if addr >= 0xA000 && addr < 0xC000 {
            if (self.is_mbc1() || self.is_mbc3() || self.is_mbc5()) && self.ram_enabled {
                if self.is_mbc1() {
                    // MBC1: usa banco efetivo baseado no modo
                    let bank = self.mbc1_effective_ram_bank();
                    let ram_addr = (bank as usize) * 0x2000 + (addr - 0xA000);
                    if ram_addr < self.cart_ram.len() {
                        return self.cart_ram[ram_addr];
                    }
                } else if self.is_mbc3() {
                    if self.ram_bank <= 0x03 {
                        // RAM banco 0..3
                        let ram_addr = (self.ram_bank as usize) * 0x2000 + (addr - 0xA000);
                        if ram_addr < self.cart_ram.len() {
                            return self.cart_ram[ram_addr];
                        }
                    } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                        // RTC register latch read
                        return match self.ram_bank {
                            0x08 => self.rtc_latched_s,
                            0x09 => self.rtc_latched_m,
                            0x0A => self.rtc_latched_h,
                            0x0B => self.rtc_latched_dl,
                            0x0C => self.rtc_latched_dh,
                            _ => 0xFF,
                        };
                    }
                } else if self.is_mbc5() {
                    // MBC5: RAM banking simples (bits 0-3 do ram_bank)
                    let bank = (self.mbc5_ram_bank & 0x0F) as usize; // ignora rumble bit
                    let ram_addr = bank * 0x2000 + (addr - 0xA000);
                    if ram_addr < self.cart_ram.len() {
                        return self.cart_ram[ram_addr];
                    }
                }
            }
            return 0xFF; // RAM desabilitada ou endereço inválido
        }
        // OAM (Object Attribute Memory) 0xFE00-0xFE9F
        if address >= 0xFE00 && address <= 0xFE9F {
            return self.ppu.read_oam(address);
        }
        // Registradores PPU (0xFF40-0xFF4B)
        if address >= 0xFF40 && address <= 0xFF4B {
            return self.ppu.read_register(address);
        }
        // Registradores APU (0xFF10-0xFF3F)
        if address >= 0xFF10 && address <= 0xFF3F {
            return self.apu.read_register(address);
        }
        self.memory[addr]
    }

    pub fn write(&mut self, address: u16, byte: u8) {

        // Tratamento de registradores MBC1
        if self.is_mbc1() {
            match address {
                0x0000..=0x1FFF => {
                    // Enable/disable RAM (0x0A habilita, outros desabilitam)
                    let old = self.ram_enabled;
                    self.ram_enabled = (byte & 0x0F) == 0x0A;
                    if self.trace_enabled && old != self.ram_enabled {
                        crate::GB::trace::trace_mbc_ram_enable(self.ram_enabled);
                    }
                    return;
                }
                0x2000..=0x3FFF => {
                    // Registrador de banco 1 (bits 0-4 do ROM bank)
                    let new_reg1 = byte & 0x1F; // 5 bits
                    if self.mbc1_bank_reg1 != new_reg1 {
                        if self.trace_enabled {
                            let old_reg1 = self.mbc1_bank_reg1;
                            let old_rom = self.mbc1_effective_rom_bank();
                            self.mbc1_bank_reg1 = new_reg1;
                            let new_rom = self.mbc1_effective_rom_bank();
                            crate::GB::trace::trace_mbc1_reg1_write(old_reg1, new_reg1, old_rom, new_rom);
                        } else {
                            self.mbc1_bank_reg1 = new_reg1;
                        }
                    }
                    return;
                }
                0x4000..=0x5FFF => {
                    // Registrador de banco 2 (bits 5-6 do ROM ou RAM bank)
                    let new_reg2 = byte & 0x03; // 2 bits
                    if self.mbc1_bank_reg2 != new_reg2 {
                        if self.trace_enabled {
                            let old_reg2 = self.mbc1_bank_reg2;
                            let old_rom = self.mbc1_effective_rom_bank();
                            self.mbc1_bank_reg2 = new_reg2;
                            let new_rom = self.mbc1_effective_rom_bank();
                            let new_ram = self.mbc1_effective_ram_bank();
                            crate::GB::trace::trace_mbc1_reg2_write(old_reg2, new_reg2, old_rom, new_rom, new_ram);
                        } else {
                            self.mbc1_bank_reg2 = new_reg2;
                        }
                    }
                    return;
                }
                0x6000..=0x7FFF => {
                    // Mode select (0=ROM banking, 1=RAM banking)
                    let new_mode = byte & 0x01;
                    if self.mbc1_mode != new_mode {
                        if self.trace_enabled {
                            let old_mode = self.mbc1_mode;
                            self.mbc1_mode = new_mode;
                            crate::GB::trace::trace_mbc1_mode_switch(old_mode, new_mode);
                        } else {
                            self.mbc1_mode = new_mode;
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        // Tratamento de registradores MBC5
        if self.is_mbc5() {
            match address {
                0x0000..=0x1FFF => {
                    // Enable/disable RAM (0x0A habilita, outros desabilitam)
                    let old = self.ram_enabled;
                    self.ram_enabled = (byte & 0x0F) == 0x0A;
                    if self.trace_enabled && old != self.ram_enabled {
                        crate::GB::trace::trace_mbc_ram_enable(self.ram_enabled);
                    }
                    return;
                }
                0x2000..=0x2FFF => {
                    // ROM bank bits 0-7
                    if self.mbc5_rom_bank_low != byte {
                        if self.trace_enabled {
                            let old_bank = self.mbc5_effective_rom_bank();
                            self.mbc5_rom_bank_low = byte;
                            let new_bank = self.mbc5_effective_rom_bank();
                            crate::GB::trace::trace_mbc5_rom_bank(old_bank, new_bank);
                        } else {
                            self.mbc5_rom_bank_low = byte;
                        }
                    } else {
                        self.mbc5_rom_bank_low = byte;
                    }
                    return;
                }
                0x3000..=0x3FFF => {
                    // ROM bank bit 8 (só bit 0 é usado)
                    let new_high = byte & 0x01;
                    if self.mbc5_rom_bank_high != new_high {
                        if self.trace_enabled {
                            let old_bank = self.mbc5_effective_rom_bank();
                            self.mbc5_rom_bank_high = new_high;
                            let new_bank = self.mbc5_effective_rom_bank();
                            crate::GB::trace::trace_mbc5_rom_bank(old_bank, new_bank);
                        } else {
                            self.mbc5_rom_bank_high = new_high;
                        }
                    } else {
                        self.mbc5_rom_bank_high = new_high;
                    }
                    return;
                }
                0x4000..=0x5FFF => {
                    // RAM bank (bits 0-3) + rumble (bit 3)
                    let new_ram_bank = byte & 0x0F; // 4 bits
                    if (self.mbc5_ram_bank & 0x0F) != new_ram_bank {
                        if self.trace_enabled {
                            let rumble = (new_ram_bank & 0x08) != 0;
                            let bank = new_ram_bank & 0x07;
                            println!("[MBC5] RAM banco: {} | Rumble: {}", bank, if rumble { "ON" } else { "OFF" });
                        }
                    }
                    self.mbc5_ram_bank = new_ram_bank;
                    return;
                }
                _ => {}
            }
        }

        // Tratamento de registradores MBC3 (mínimo)
        if self.is_mbc3() {
            match address {
                0x0000..=0x1FFF => {
                    // Enable/disable RAM/RTC (0x0A habilita, outros desabilitam)
                    let old = self.ram_enabled;
                    self.ram_enabled = (byte & 0x0F) == 0x0A;
                    if self.trace_enabled && old != self.ram_enabled {
                        crate::GB::trace::trace_mbc_ram_enable(self.ram_enabled);
                    }
                    return;
                }
                0x2000..=0x3FFF => {
                    // Seleção de banco ROM (7 bits, 0 => 1)
                    let mut bank = byte & 0x7F;
                    if bank == 0 { bank = 1; }
                    if self.rom_bank != bank {
                        if self.trace_enabled {
                            crate::GB::trace::trace_mbc_rom_bank(self.rom_bank, bank);
                        }
                        self.rom_bank = bank;
                    } else {
                        self.rom_bank = bank;
                    }
                    return;
                }
                0x4000..=0x5FFF => {
                    // Seleção de banco RAM (00-03) ou registrador RTC (08-0C)
                    if self.ram_bank != byte {
                        if self.trace_enabled {
                            crate::GB::trace::trace_mbc_ram_rtc_select(byte);
                        }
                        self.ram_bank = byte;
                    } else {
                        self.ram_bank = byte;
                    }
                    return;
                }
                0x6000..=0x7FFF => {
                    // Latch clock: transição 0x00 -> 0x01 captura RTC atual
                    if self.rtc_latch_state == 0x00 && byte == 0x01 {
                        self.rtc_latched_s = self.rtc_s;
                        self.rtc_latched_m = self.rtc_m;
                        self.rtc_latched_h = self.rtc_h;
                        self.rtc_latched_dl = self.rtc_dl;
                        self.rtc_latched_dh = self.rtc_dh;
                        if self.trace_enabled {
                            crate::GB::trace::trace_mbc_rtc_latch(
                                self.rtc_h, self.rtc_m, self.rtc_s, self.rtc_dh, self.rtc_dl
                            );
                        }
                    }
                    self.rtc_latch_state = byte;
                    return;
                }
                _ => {}
            }
        }

        // VRAM (0x8000-0x9FFF)
        if address >= 0x8000 && address < 0xA000 {
            self.ppu.write_vram(address, byte);
            return;
        }

        // OAM (0xFE00-0xFE9F)
        if address >= 0xFE00 && address <= 0xFE9F {
            self.ppu.write_oam(address, byte);
            return;
        }

        // DMA (Direct Memory Access) DEVE ser tratado ANTES dos registradores PPU gerais!
        // DMA permite transferir dados automaticamente entre áreas da memória sem envolver a CPU
        if address == 0xFF46 {
            self.memory[address as usize] = byte;
            let source_base = (byte as u16) * 0x100;

            // DMA copia 160 bytes usando o bus normal
            for i in 0..160 {
                let source_addr = source_base + i;
                let data = self.read(source_addr); // usa o bus normal
                self.ppu.write_oam(0xFE00 + i, data); // alimenta a OAM que o renderer usa
            }
            return;
        }

        // Registradores APU (0xFF10-0xFF3F)
        if address >= 0xFF10 && address <= 0xFF3F {
            self.apu.write_register(address, byte);
            return;
        }

        // Registradores PPU (0xFF40-0xFF4B, exceto 0xFF46 que é DMA)
        if address >= 0xFF40 && address <= 0xFF4B {
            if address == 0xFF40 { // LCDC register
                eprintln!("*** LCDC Write *** value={:02X} (sprites={}, bg={}, window={})",
                    byte,
                    (byte & 0x02) != 0,
                    (byte & 0x01) != 0,
                    (byte & 0x20) != 0);
            }
            self.ppu.write_register(address, byte);
            return;
        }

        match address {
            0xFF00 => { // Joypad (P1/JOYP)
                // Bits 5-4 controlam qual grupo de botões é lido (0=selecionado)
                let old_sel_buttons = self.joypad_select_buttons;
                let old_sel_dpad = self.joypad_select_dpad;

                self.joypad_select_buttons = (byte & 0x20) == 0;
                self.joypad_select_dpad = (byte & 0x10) == 0;

                if self.trace_enabled && (old_sel_buttons != self.joypad_select_buttons || old_sel_dpad != self.joypad_select_dpad) {
                    crate::GB::trace::trace_joypad_selection(
                        self.joypad_select_dpad, self.joypad_select_buttons
                    );
                }

                // Bits 3-0 são apenas leitura (estado dos botões), ignoramos escrita neles
            }
            0xFF04 => { // escrita em DIV: zera o divisor interno e o registrador DIV
                self.div_counter = 0;
                self.memory[address as usize] = 0;
                // Recalcula o último sinal do timer após o reset para evitar bordas espúrias
                self.timer_last_signal = self.current_timer_signal();
                if self.trace_enabled {
                    crate::GB::trace::trace_timer_div_reset();
                }
            }
            0xFF07 => { // TAC
                self.memory[address as usize] = byte & 0x07; // apenas os 3 bits menos significativos são usados
                // Atualiza o último sinal conforme a nova configuração do TAC
                self.timer_last_signal = self.current_timer_signal();
                if self.trace_enabled {
                    crate::GB::trace::trace_timer_tac(byte);
                }
            }
            0xFF05 | 0xFF06 => { // TIMA, TMA
                self.memory[address as usize] = byte;
                if self.trace_enabled {
                    if address == 0xFF05 {
                        crate::GB::trace::trace_timer_tima(byte);
                    } else {
                        crate::GB::trace::trace_timer_tma(byte);
                    }
                }
            }

            0xA000..=0xBFFF => {
                // Escrita em RAM externa ou RTC
                if (self.is_mbc1() || self.is_mbc3() || self.is_mbc5()) && self.ram_enabled {
                    if self.is_mbc1() {
                        // MBC1: usa banco efetivo baseado no modo
                        let bank = self.mbc1_effective_ram_bank();
                        let ram_addr = (bank as usize) * 0x2000 + (address as usize - 0xA000);
                        if ram_addr < self.cart_ram.len() {
                            self.cart_ram[ram_addr] = byte;
                        }
                    } else if self.is_mbc3() {
                        if self.ram_bank <= 0x03 {
                            // RAM banco 0..3
                            let ram_addr = (self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000);
                            if ram_addr < self.cart_ram.len() {
                                self.cart_ram[ram_addr] = byte;
                            }
                        } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                            // Escrita nos registradores RTC
                            match self.ram_bank {
                                0x08 => self.rtc_s = byte & 0x3F,   // 0..59
                                0x09 => self.rtc_m = byte & 0x3F,   // 0..59
                                0x0A => self.rtc_h = byte & 0x1F,   // 0..23
                                0x0B => self.rtc_dl = byte,
                                0x0C => self.rtc_dh = byte,
                                _ => {}
                            }
                        }
                    } else if self.is_mbc5() {
                        // MBC5: RAM banking simples (bits 0-3 do ram_bank)
                        let bank = (self.mbc5_ram_bank & 0x0F) as usize; // ignora rumble bit
                        let ram_addr = bank * 0x2000 + (address as usize - 0xA000);
                        if ram_addr < self.cart_ram.len() {
                            self.cart_ram[ram_addr] = byte;
                        }
                    }
                }
                return;
            }
            _ => {
                // Evita escrita em ROM (0x0000..=0x7FFF)
                if address < 0x8000 { return; }
                self.memory[address as usize] = byte;
            }
        }
    }

    pub fn load_bytes(&mut self, data: &[u8]) {
        // Armazena ROM completa e define tipo de cartucho
        self.rom = data.to_vec();
        self.cart_type = if self.rom.len() > 0x0147 { self.rom[0x0147] } else { 0x00 };
        self.rom_bank = 1;
        // Limpa a RAM interna (áreas não-ROM)
        self.memory = [0; 65536];
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
                    if self.trace_enabled {
                        crate::GB::trace::trace_timer_interrupt(tma);
                    }
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

    // Utilidades MBC/ROM
    fn is_mbc1(&self) -> bool {
        matches!(self.cart_type, 0x01..=0x03)
    }

    fn is_mbc3(&self) -> bool {
        matches!(self.cart_type, 0x0F..=0x13)
    }

    fn is_mbc5(&self) -> bool {
        matches!(self.cart_type, 0x19..=0x1E)
    }

    fn rom_get(&self, idx: usize) -> u8 {
        if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
    }

    /// Calcula o banco ROM efetivo para MBC1 baseado nos registradores e modo atual
    fn mbc1_effective_rom_bank(&self) -> u8 {
        if !self.is_mbc1() { return self.rom_bank; }

        let mut bank = self.mbc1_bank_reg1;
        if bank == 0 { bank = 1; } // banco 0 é mapeado para 1

        // No modo ROM banking, reg2 afeta os bits altos do ROM bank
        if self.mbc1_mode == 0 {
            bank |= (self.mbc1_bank_reg2 & 0x03) << 5;
        }

        bank
    }

    /// Calcula o banco RAM efetivo para MBC1 baseado no modo atual
    fn mbc1_effective_ram_bank(&self) -> u8 {
        if !self.is_mbc1() { return self.ram_bank; }

        // No modo RAM banking, reg2 seleciona o banco RAM
        if self.mbc1_mode == 1 {
            self.mbc1_bank_reg2 & 0x03
        } else {
            0 // modo ROM banking usa sempre RAM banco 0
        }
    }

    /// Calcula o banco ROM efetivo para MBC5 (9 bits)
    fn mbc5_effective_rom_bank(&self) -> u16 {
        if !self.is_mbc5() { return self.rom_bank as u16; }

        // MBC5: 9-bit ROM bank (0-511)
        let bank = (self.mbc5_rom_bank_low as u16) | ((self.mbc5_rom_bank_high as u16) << 8);
        if bank == 0 { 1 } else { bank } // banco 0 mapeado para 1
    }

    // === API pública para manipular joypad ===

    pub fn press_joypad_button(&mut self, button: u8, is_dpad: bool) {
        if is_dpad {
            self.joypad_dpad &= !(1 << button);
        } else {
            self.joypad_buttons &= !(1 << button);
        }
    }

    pub fn release_joypad_button(&mut self, button: u8, is_dpad: bool) {
        if is_dpad {
            self.joypad_dpad |= 1 << button;
        } else {
            self.joypad_buttons |= 1 << button;
        }
    }

    // === Save/Load persistence para arquivos .sav ===

    /// Salva a RAM do cartucho para arquivo .sav
    pub fn save_cart_ram(&self, sav_path: &str) -> Result<(), std::io::Error> {
        use std::fs;

        // Só salva se há RAM e é MBC com suporte a saves
        if !(self.is_mbc1() || self.is_mbc3() || self.is_mbc5()) || !self.has_cart_ram() || self.cart_ram.is_empty() {
            return Ok(()); // Nada para salvar
        }

        // Salva apenas a RAM usada (não os 32KB completos se o jogo usa menos)
        let ram_size = self.get_cart_ram_size();
        if ram_size > 0 {
            let data_to_save = &self.cart_ram[..ram_size.min(self.cart_ram.len())];
            fs::write(sav_path, data_to_save)?;
            println!("💾 Save criado: {} ({} bytes)", sav_path, data_to_save.len());
        }

        Ok(())
    }

    /// Carrega a RAM do cartucho de arquivo .sav
    pub fn load_cart_ram(&mut self, sav_path: &str) -> Result<(), std::io::Error> {
        use std::fs;

        // Só tenta carregar se há RAM no cartucho e é MBC com suporte
        if !(self.is_mbc1() || self.is_mbc3() || self.is_mbc5()) || !self.has_cart_ram() {
            return Ok(());
        }

        // Verifica se arquivo existe
        if !std::path::Path::new(sav_path).exists() {
            println!("📁 Nenhum save encontrado: {}", sav_path);
            return Ok(());
        }

        // Carrega dados do arquivo
        let save_data = fs::read(sav_path)?;
        if save_data.is_empty() {
            return Ok(());
        }

        // Copia dados para cart_ram (limitado ao tamanho disponível)
        let copy_size = save_data.len().min(self.cart_ram.len());
        self.cart_ram[..copy_size].copy_from_slice(&save_data[..copy_size]);

        println!("💾 Save carregado: {} ({} bytes)", sav_path, copy_size);
        Ok(())
    }

    /// Verifica se o cartucho tem RAM baseado no header
    fn has_cart_ram(&self) -> bool {
        // Lê o código de RAM size do header (0x0149)
        if self.rom.len() > 0x0149 {
            let ram_code = self.rom[0x0149];
            ram_code != 0x00 // 0x00 = sem RAM
        } else {
            false
        }
    }

    /// Retorna o tamanho da RAM do cartucho baseado no header
    fn get_cart_ram_size(&self) -> usize {
        if self.rom.len() <= 0x0149 {
            return 0;
        }

        match self.rom[0x0149] {
            0x00 => 0,          // Sem RAM
            0x01 => 2 * 1024,   // 2KB
            0x02 => 8 * 1024,   // 8KB
            0x03 => 32 * 1024,  // 32KB (4 bancos × 8KB)
            0x04 => 128 * 1024, // 128KB (16 bancos × 8KB)
            0x05 => 64 * 1024,  // 64KB (8 bancos × 8KB)
            _ => 0,
        }
    }
}