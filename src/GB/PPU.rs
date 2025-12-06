#![allow(non_snake_case)]

// PPU (Picture Processing Unit) - Renderização de gráficos do Game Boy
// Responsável por: Background, Window, Sprites, Paletas

// Estrutura para representar um sprite do OAM
#[derive(Debug, Clone, Copy)]
struct Sprite {
    y: u8,          // Posição Y (linha + 16)
    x: u8,          // Posição X (coluna + 8)
    tile_index: u8, // Índice do tile (0-255)
    attributes: u8, // Bit 7=prioridade, 6=flip Y, 5=flip X, 4=paleta, 3-0=unused
}

// FIFO item para pixels BG e OAM
#[derive(Debug, Clone, Copy)]
struct FifoItem {
    pixel: u8,      // Cor do pixel (0-3)
    bg_priority: bool,
    palette: u8,    // Paleta (para sprites)
    priority: u8,   // Prioridade (para sprites, menor = mais prioritário)
}

// FIFO simples para pixels (8 pixels por vez)
struct Fifo {
    buffer: [FifoItem; 8],
    read_end: usize, // SameBoy: onde o próximo pixel será lido
    size: usize,
}

impl Fifo {
    fn new() -> Self {
        Self {
            buffer: [FifoItem { pixel: 0, bg_priority: false, palette: 0, priority: 0xFF }; 8],
            read_end: 0,
            size: 0,
        }
    }

    fn clear(&mut self) {
        self.read_end = 0;
        self.size = 0;
    }

    fn push_row(&mut self, byte1: u8, byte2: u8, bg_priority_base: bool) {
        // Limpa FIFO e adiciona 8 pixels de uma linha de tile
        self.clear();
        self.size = 8;
        for i in 0..8 {
            let bit_pos = 7 - i;
            let lsb = (byte1 >> bit_pos) & 1;
            let msb = (byte2 >> bit_pos) & 1;
            let pixel = (msb << 1) | lsb;
            self.buffer[i] = FifoItem {
                pixel,
                bg_priority: bg_priority_base && pixel != 0,
                palette: 0,
                priority: 0xFF,
            };
        }
        self.read_end = 0;
    }

    fn pop(&mut self) -> Option<FifoItem> {
        if self.size == 0 {
            return None;
        }
        let item = self.buffer[self.read_end];
        self.read_end += 1;
        self.read_end &= 7;
        self.size -= 1;
        Some(item)
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[allow(dead_code)]
    fn size(&self) -> usize {
        self.size
    }
}

// Estados do fetcher
#[derive(Debug, Clone, Copy, PartialEq)]
enum FetcherState {
    GetTileT1,           // T1: Determina endereço do tile map
    GetTileT2,           // T2: Lê tile index do tile map
    GetTileDataLowerT1,  // T1: Determina endereço dos dados do tile (byte baixo)
    GetTileDataLowerT2,  // T2: Lê byte baixo do tile data
    GetTileDataHighT1,   // T1: Determina endereço dos dados do tile (byte alto)
    GetTileDataHighT2,   // T2: Lê byte alto do tile data
    Push,                // Push 8 pixels para FIFO
}

use rand::Rng;
pub struct PPU {
    // VRAM (Video RAM) - 8KB (0x8000-0x9FFF)
    // 0x8000-0x97FF: Tile data (384 tiles × 16 bytes = 6KB)
    // 0x9800-0x9BFF: Tile map 0 (32×32 = 1KB)
    // 0x9C00-0x9FFF: Tile map 1 (32×32 = 1KB)
    pub vram: [u8; 0x2000],

    // Framebuffer - 160×144 pixels, cada pixel = 0-3 (2 bits por cor)
    pub framebuffer: [u8; 160 * 144],

    /// Per-pixel BG priority buffer (true = BG/window pixel is opaque)
    pub bg_priority: [bool; 160 * 144],

    // Registradores PPU (endereços I/O)
    pub lcdc: u8, // 0xFF40 - LCD Control
    pub stat: u8, // 0xFF41 - LCD Status
    pub scy: u8,  // 0xFF42 - Scroll Y
    pub scx: u8,  // 0xFF43 - Scroll X
    pub ly: u8,   // 0xFF44 - Line Y (registrador LY, atualizado em momentos específicos)
    current_line: u8,
    pub lyc: u8,  // 0xFF45 - LY Compare
    pub bgp: u8,  // 0xFF47 - Background Palette
    pub obp0: u8, // 0xFF48 - Object Palette 0
    pub obp1: u8, // 0xFF49 - Object Palette 1
    pub wy: u8,   // 0xFF4A - Window Y
    pub wx: u8,   // 0xFF4B - Window X

    // OAM (Object Attribute Memory) - 160 bytes (40 sprites × 4 bytes)
    pub oam: [u8; 160],

    // Controle de window: início e linha da window
    pub wy_trigger: bool,
    pub wy_pos: i32,

    // Flag para indicar quando um frame foi completado (VBlank)
    pub frame_ready: bool,

    // Ciclos acumulados na linha atual (456 ciclos por linha)
    pub mode: u8,        // 0=HBlank, 1=VBlank, 2=OAM, 3=Transfer
    pub mode_clock: u32, // Acumula ciclos para controle de modo

    // Estado interno para STAT/LYC
    pub ly_eq_lyc_prev: bool,


    pub last_accessed_oam_row: usize,

    // Sprites visíveis coletados durante mode 2 (OAM Search)
    visible_sprites: Vec<(Sprite, u8)>, // (sprite, sprite_index)

    bg_fifo: Fifo,
    oam_fifo: Fifo,
    fetcher_state: FetcherState,
    fetcher_tile_index: u8,
    fetcher_tile_data: [u8; 2],
    position_in_line: i16,
    lcd_x: u8,
    window_tile_x: u8,
    wx_triggered: bool,
    window_is_being_fetched: bool,
    during_object_fetch: bool,
    cycles_for_line: u16,
}

impl PPU {
    /// Atualiza LCDC e trata ON/OFF conforme hardware
    fn set_lcdc(&mut self, new_val: u8, iflags: &mut u8) {
        let was_on = (self.lcdc & 0x80) != 0;
        let now_on = (new_val & 0x80) != 0;
        self.lcdc = new_val;

        // LCD ligado -> desligado
        if was_on && !now_on {
            self.mode = 0;
            self.mode_clock = 0;
            self.ly = 0;
            self.frame_ready = false;
            self.wy_trigger = false;
            self.wy_pos = -1;
            self.update_stat_mode(0);
            self.update_lyc_flag();
            self.ly_eq_lyc_prev = self.ly == self.lyc;
            *iflags &= !0x02; // limpa bit de LCD STAT
        }

        // LCD desligado -> ligado
        if !was_on && now_on {
            self.mode = 2;
            self.mode_clock = 0;
            self.ly = 0;
            self.frame_ready = false;
            self.wy_trigger = false;
            self.wy_pos = -1;
            self.last_accessed_oam_row = 0xFF; // Reset para nova scanline
            self.update_stat_mode(2);
            self.update_lyc_flag();
            self.ly_eq_lyc_prev = self.ly == self.lyc;
        }
    }
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        // VRAM com lixo de power-on
        let mut vram = [0u8; 0x2000];
        rng.fill(&mut vram[..]);

        // OAM com lixo de power-on
        let mut oam = [0u8; 160];
        rng.fill(&mut oam[..]);

        PPU {
            vram,
            framebuffer: [0; 160 * 144],
            bg_priority: [false; 160 * 144],
            lcdc: 0x91, // Default pós-boot: LCD on, BG on, 8x8 sprites
            stat: 0x00,
            scy: 0,
            scx: 0,
            ly: 0,
            current_line: 0,
            lyc: 0,
            bgp: 0xFC, // Default: cores 3,3,2,1,0 = branco,branco,cinza claro,cinza escuro
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            oam,
            frame_ready: false,
            mode: 2, // Começa em OAM Search
            mode_clock: 0,
            wy_trigger: false,
            wy_pos: -1,
            ly_eq_lyc_prev: false,
            last_accessed_oam_row: 0xFF, // 0xFF = nenhuma row escaneada ainda
            visible_sprites: Vec::new(),
            bg_fifo: Fifo::new(),
            oam_fifo: Fifo::new(),
            fetcher_state: FetcherState::GetTileT1,
            fetcher_tile_index: 0,
            fetcher_tile_data: [0; 2],
            position_in_line: -16,
            lcd_x: 0,
            window_tile_x: 0,
            wx_triggered: false,
            window_is_being_fetched: false,
            during_object_fetch: false,
            cycles_for_line: 0,
        }
    }

    pub fn update_lyc_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
    }

    fn get_sprite(&self, sprite_index: u8) -> Sprite {
        let base = (sprite_index as usize) * 4;
        Sprite {
            y: self.oam[base],
            x: self.oam[base + 1],
            tile_index: self.oam[base + 2],
            attributes: self.oam[base + 3],
        }
    }

    /// Coleta sprites visíveis durante mode 2 (OAM Search)
    fn collect_visible_sprites(&mut self) {
        self.visible_sprites.clear();

        // Verificar se sprites estão habilitados (bit 1 do LCDC)
        if (self.lcdc & 0x02) == 0 {
            return;
        }

        let sprite_height = if (self.lcdc & 0x04) != 0 { 16 } else { 8 };

        for sprite_index in 0..40 {
            let sprite = self.get_sprite(sprite_index);
            let sprite_y = (sprite.y as i16) - 16;
            if (self.current_line as i16) >= sprite_y && (self.current_line as i16) < sprite_y + sprite_height as i16 {
                self.visible_sprites.push((sprite, sprite_index));
            }
        }
        // Limita a 10 sprites por linha
        if self.visible_sprites.len() > 10 {
            self.visible_sprites.truncate(10);
        }
        // Ordena por prioridade DMG: x menor primeiro, depois OAM menor
        self.visible_sprites.sort_by(|a, b| {
            let ax = a.0.x;
            let bx = b.0.x;
            if ax != bx { ax.cmp(&bx) } else { a.1.cmp(&b.1) }
        });
    }

    pub fn update_stat_mode(&mut self, mode: u8) {
        self.stat = (self.stat & 0xFC) | (mode & 0x03);
    }

    pub fn read_stat(&self) -> u8 {
        0x80 |
        (if (self.stat & 0x40) != 0 { 0x40 } else { 0 }) |
        (if (self.stat & 0x20) != 0 { 0x20 } else { 0 }) |
        (if (self.stat & 0x10) != 0 { 0x10 } else { 0 }) |
        (if (self.stat & 0x08) != 0 { 0x08 } else { 0 }) |
        (if self.ly == self.lyc { 0x04 } else { 0 }) |
        (self.mode & 0x03)
    }

    pub fn write_stat(&mut self, val: u8) {
        self.stat = (self.stat & 0x07) | (val & 0xF8);
    }
    pub fn decode_tile(&self, tile_index: u16) -> [u8; 64] {
        let mut pixels = [0u8; 64];
        let tile_addr = tile_index * 16; // Cada tile = 16 bytes

        for y in 0..8 {
            let byte1 = self.vram[(tile_addr + y * 2) as usize];
            let byte2 = self.vram[(tile_addr + y * 2 + 1) as usize];

            for x in 0..8 {
                let bit_index = 7 - x; // Pixels são MSB first
                let lsb = (byte1 >> bit_index) & 1;
                let msb = (byte2 >> bit_index) & 1;
                let color = (msb << 1) | lsb;
                pixels[(y * 8 + x) as usize] = color;
            }
        }

        pixels
    }

    fn apply_palette(&self, color: u8, palette_reg: u8) -> u8 {
        let shift = color * 2;
        (palette_reg >> shift) & 0x03
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        let offset = (addr - 0x8000) as usize;
        if offset < 0x2000 {
            self.vram[offset]
        } else {
            0xFF
        }
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        let offset = (addr - 0x8000) as usize;
        if offset < 0x2000 {
            self.vram[offset] = val;
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let offset = (addr - 0xFE00) as usize;
        if offset < 160 { self.oam[offset] } else { 0xFF }
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        let offset = (addr - 0xFE00) as usize;
        if offset < 160 {
            self.oam[offset] = val;
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.read_stat(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8, iflags: &mut u8) {
        match addr {
            0xFF40 => self.set_lcdc(val, iflags),
            0xFF41 => self.write_stat(val),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {}
            0xFF45 => {
                self.lyc = val;
                self.update_lyc_flag();
                self.check_lyc_interrupt(iflags);
            }
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {}
        }
    }

    /// Avança PPU em `cycles` ciclos de CPU
    pub fn step(&mut self, cycles: u32, iflags: &mut u8) {
        if (self.lcdc & 0x80) == 0 {
            self.mode = 0;
            self.mode_clock = 0;
            self.ly = 0;
            self.current_line = 0;
            self.frame_ready = false;
            self.wy_trigger = false;
            self.wy_pos = -1;
            self.ly_eq_lyc_prev = self.ly == self.lyc;
            return;
        }

        self.mode_clock += cycles;

        // Atualizar current_line ANTES de verificar os modes, para que collect_visible_sprites use a linha correta
        if self.mode_clock >= 456 {
            self.mode_clock -= 456;
            self.current_line = (self.current_line + 1) % 154;
            self.ly = self.current_line;
            self.cycles_for_line = 0;
            if self.current_line < 144 {
                self.position_in_line = -15;
            } else {
                self.position_in_line = -16;
            }
            self.update_lyc_flag();
            self.check_lyc_interrupt(iflags);
            if self.current_line >= 144 && self.mode != 1 {
                self.change_mode(1, iflags);
            }
        }

        if self.current_line < 144 {
            if self.mode_clock < 80 {
                if self.mode != 2 {
                    self.change_mode(2, iflags);
                }
            } else if self.mode_clock < 252 {
                if self.mode != 3 {
                    self.change_mode(3, iflags);
                }
                for _ in 0..cycles {
                    if self.position_in_line == 160 {
                        break;
                    }
                    self.step_mode3_rendering(iflags);
                }
            } else if self.mode_clock < 456 {
                if self.mode != 0 {
                    self.change_mode(0, iflags);
                }
                if (self.lcdc & 0x20) == 0 {
                    self.wy_trigger = false;
                    self.wy_pos = -1;
                    self.wx_triggered = false;
                }
            }
        } else {
            if self.mode != 1 {
                self.change_mode(1, iflags);
            }
        }

    }

    pub fn change_mode(&mut self, new_mode: u8, iflags: &mut u8) {
        self.mode = new_mode;
        self.update_stat_mode(new_mode);

        let stat_irq = match new_mode {
            0 => (self.stat & 0x08) != 0,
            1 => {
                self.frame_ready = true;
                *iflags |= 0x01;
                self.wy_trigger = false;
                self.wy_pos = -1;
                (self.stat & 0x10) != 0
            }
            2 => {
                self.collect_visible_sprites();
                (self.stat & 0x20) != 0
            }
            3 => {
                if (self.lcdc & 0x20) != 0 && !self.wy_trigger && self.current_line == self.wy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }
                self.init_mode3_rendering();
                false
            }
            _ => false,
        };

        if stat_irq {
            *iflags |= 0x02;
        }
    }

    pub fn check_lyc_interrupt(&mut self, iflags: &mut u8) {
        let lyc_inte = (self.stat & 0x40) != 0;
        let now_eq = self.ly == self.lyc;
        if lyc_inte && now_eq && !self.ly_eq_lyc_prev {
            *iflags |= 0x02;
        }
        self.ly_eq_lyc_prev = now_eq;
    }

    pub fn is_oam_scan_mode(&self) -> bool {
        let lcd_on = (self.lcdc & 0x80) != 0;
        lcd_on && self.mode == 2 && self.mode_clock < 80
    }

    pub fn is_oam_write_blocked_mode(&self) -> bool {
        let lcd_on = (self.lcdc & 0x80) != 0;
        lcd_on && (self.mode == 2 || self.mode == 3)
    }

    fn get_current_oam_row(&self) -> usize {
        let m_cycles = self.mode_clock / 4;
        (m_cycles as usize).min(19)
    }

    fn read_oam_word(&self, row: usize, word_index: usize) -> u16 {
        let addr = row * 8 + word_index * 2;
        if addr + 1 < 160 {
            let lo = self.oam[addr] as u16;
            let hi = self.oam[addr + 1] as u16;
            lo | (hi << 8)
        } else {
            0
        }
    }

    fn write_oam_word(&mut self, row: usize, word_index: usize, value: u16) {
        let addr = row * 8 + word_index * 2;
        if addr + 1 < 160 {
            self.oam[addr] = (value & 0xFF) as u8;
            self.oam[addr + 1] = ((value >> 8) & 0xFF) as u8;
        }
    }

    fn apply_write_corruption(&mut self, row: usize) {
        if row == 0 {
            return;
        }
        let prev_row = row - 1;
        let a = self.read_oam_word(row, 0);
        let b = self.read_oam_word(prev_row, 0);
        let c = self.read_oam_word(prev_row, 2);
        let corrupted = ((a ^ c) & (b ^ c)) ^ c;
        self.write_oam_word(row, 0, corrupted);
        for word_idx in 1..4 {
            let value = self.read_oam_word(prev_row, word_idx);
            self.write_oam_word(row, word_idx, value);
        }
    }

    fn apply_read_corruption(&mut self, row: usize) {
        if row == 0 {
            return;
        }
        let prev_row = row - 1;
        let a = self.read_oam_word(row, 0);
        let b = self.read_oam_word(prev_row, 0);
        let c = self.read_oam_word(prev_row, 2);
        let corrupted = b | (a & c);
        self.write_oam_word(row, 0, corrupted);
        for word_idx in 1..4 {
            let value = self.read_oam_word(prev_row, word_idx);
            self.write_oam_word(row, word_idx, value);
        }
    }

    fn apply_read_inc_dec_corruption(&mut self, row: usize) {
        if row >= 4 && row < 19 {
            let prev_row = row - 1;
            let prev_prev_row = row - 2;
            let a = self.read_oam_word(prev_prev_row, 0);
            let b = self.read_oam_word(prev_row, 0);
            let c = self.read_oam_word(row, 0);
            let d = self.read_oam_word(prev_row, 2);
            let corrupted = (b & (a | c | d)) | (a & c & d);
            self.write_oam_word(prev_row, 0, corrupted);
            for word_idx in 0..4 {
                let value = self.read_oam_word(prev_row, word_idx);
                self.write_oam_word(row, word_idx, value);
                self.write_oam_word(prev_prev_row, word_idx, value);
            }
        }
        self.apply_read_corruption(row);
    }

    pub fn trigger_oam_bug_write(&mut self) {
        if !self.is_oam_write_blocked_mode() {
            return;
        }
        let row = if self.mode == 2 {
            let row = self.get_current_oam_row();
            if row < 20 {
                self.last_accessed_oam_row = row;
            }
            row
        } else {
            if self.last_accessed_oam_row != 0xFF && self.last_accessed_oam_row >= 8 {
                self.last_accessed_oam_row
            } else {
                return;
            }
        };
        self.apply_write_corruption(row);
    }

    pub fn trigger_oam_bug_read(&mut self) {
        if !self.is_oam_write_blocked_mode() {
            return;
        }
        let row = if self.mode == 2 {
            let row = self.get_current_oam_row();
            if row < 20 {
                self.last_accessed_oam_row = row;
            }
            row
        } else {
            if self.last_accessed_oam_row != 0xFF && self.last_accessed_oam_row >= 8 {
                self.last_accessed_oam_row
            } else {
                return;
            }
        };
        self.apply_read_corruption(row);
    }

    pub fn trigger_oam_bug_read_inc_dec(&mut self) {
        if !self.is_oam_scan_mode() {
            return;
        }
        let row = self.get_current_oam_row();
        self.apply_read_inc_dec_corruption(row);
    }

    fn init_mode3_rendering(&mut self) {
        self.bg_fifo.clear();
        self.oam_fifo.clear();
        self.bg_fifo.push_row(0, 0, false);
        self.lcd_x = 0;
        self.fetcher_state = FetcherState::GetTileT1;
        self.window_tile_x = 0;
        self.wx_triggered = false;
        self.window_is_being_fetched = false;
        self.during_object_fetch = false;
        self.cycles_for_line = 0;

        let line_start = self.current_line as usize * 160;
        for x in 0..160 {
            self.bg_priority[line_start + x] = false;
        }
    }

    fn step_mode3_rendering(&mut self, _iflags: &mut u8) {
        if self.position_in_line == 160 {
            return;
        }

        self.check_window_activation();
        self.handle_objects_during_mode3();
        self.render_pixel_if_possible();
        self.advance_fetcher();
        self.cycles_for_line += 1;
    }

    fn check_window_activation(&mut self) {
        if !self.wx_triggered && self.wy_trigger && (self.lcdc & 0x20) != 0 {
            let wx = self.wx;
            let should_activate = if wx == 0 {
                self.position_in_line == -7 ||
                (self.position_in_line == -16 && (self.scx & 7) != 0) ||
                (self.position_in_line >= -15 && self.position_in_line <= -8)
            } else if wx < 166 {
                (self.position_in_line + 7) as u8 == wx
            } else {
                false
            };

            if should_activate {
                if self.wy_pos < 0 {
                    self.wy_pos = 0;
                } else {
                    self.wy_pos += 1;
                }
                self.window_tile_x = 0;
                self.bg_fifo.clear();
                self.wx_triggered = true;
                self.fetcher_state = FetcherState::GetTileT1;
                self.window_is_being_fetched = true;
            } else if wx == 166 && (self.position_in_line + 7) as u8 == wx {
                if self.wy_pos < 0 {
                    self.wy_pos = 0;
                } else {
                    self.wy_pos += 1;
                }
            }
        }
    }

    fn advance_fetcher(&mut self) {
        match self.fetcher_state {
            FetcherState::GetTileT1 => {
                if (self.lcdc & 0x20) == 0 {
                    self.wx_triggered = false;
                }
                let map = if (self.lcdc & 0x08) != 0 && !self.wx_triggered {
                    0x1C00
                } else if (self.lcdc & 0x40) != 0 && self.wx_triggered {
                    0x1C00
                } else {
                    0x1800
                };

                let y = if self.wx_triggered {
                    if self.wy_pos < 0 { 0 } else { self.wy_pos as u8 }
                } else {
                    self.current_line.wrapping_add(self.scy)
                };

                let x = if self.wx_triggered {
                    self.window_tile_x
                } else if (self.position_in_line as u8).wrapping_add(16) < 8 {
                    self.scx >> 3
                } else {
                    let is_cgb = false;
                    let offset = if is_cgb && !self.during_object_fetch { 1 } else { 0 };
                    (((self.scx as i16) + self.position_in_line + 8 - offset) / 8 & 0x1F) as u8
                };

                let tile_map_addr = (map as usize) + (x as usize) + ((y as usize / 8) * 32);
                self.fetcher_tile_index = self.vram[tile_map_addr.min(0x1FFF)];
                self.fetcher_state = FetcherState::GetTileT2;
            }
            FetcherState::GetTileT2 => {
                self.fetcher_state = FetcherState::GetTileDataLowerT1;
            }
            FetcherState::GetTileDataLowerT1 => {
                let tile_addr = self.get_tile_data_address(self.fetcher_tile_index, false);
                self.fetcher_tile_data[0] = self.vram[tile_addr.min(0x1FFF)];
                self.fetcher_state = FetcherState::GetTileDataLowerT2;
            }
            FetcherState::GetTileDataLowerT2 => {
                self.fetcher_state = FetcherState::GetTileDataHighT1;
            }
            FetcherState::GetTileDataHighT1 => {
                let tile_addr = self.get_tile_data_address(self.fetcher_tile_index, true);
                self.fetcher_tile_data[1] = self.vram[tile_addr.min(0x1FFF)];
                self.fetcher_state = FetcherState::GetTileDataHighT2;
            }
            FetcherState::GetTileDataHighT2 => {
                self.fetcher_state = FetcherState::Push;
            }
            FetcherState::Push => {
                if !self.bg_fifo.is_empty() {
                    return;
                }

                let bg_priority_base = (self.lcdc & 0x01) != 0;
                self.bg_fifo.push_row(self.fetcher_tile_data[0], self.fetcher_tile_data[1], bg_priority_base);

                if self.wx_triggered {
                    self.window_tile_x = (self.window_tile_x.wrapping_add(1)) & 0x1F;
                }
                self.fetcher_state = FetcherState::GetTileT1;
            }
        }
    }

    fn get_tile_data_address(&self, tile_index: u8, high_byte: bool) -> usize {
        let y = if self.wx_triggered {
            if self.wy_pos < 0 { 0 } else { self.wy_pos as u8 }
        } else {
            self.current_line.wrapping_add(self.scy)
        };
        let line_in_tile = y % 8;

        let base_addr = if (self.lcdc & 0x10) != 0 {
            (tile_index as usize) * 16
        } else {
            let signed = tile_index as i8;
            (0x1000 + (signed as i16) * 16) as usize
        };

        base_addr + (line_in_tile as usize * 2) + if high_byte { 1 } else { 0 }
    }

    fn render_pixel_if_possible(&mut self) {
        if self.bg_fifo.is_empty() {
            return;
        }

        if !self.visible_sprites.is_empty() && (self.lcdc & 0x02) != 0 {
            if let Some((sprite, _)) = self.visible_sprites.last() {
                if sprite.x == 0 {
                    return;
                }
            }
        }

        let fifo_item = match self.bg_fifo.pop() {
            Some(item) => item,
            None => return,
        };
        let bg_priority = fifo_item.bg_priority;

        let mut sprite_behind_bg = false;
        let mut draw_oam = false;
        let mut oam_pixel = 0;
        let mut oam_palette = 0;
        if !self.oam_fifo.is_empty() {
            if let Some(oam_item) = self.oam_fifo.pop() {
                if oam_item.pixel != 0 && (self.lcdc & 0x02) != 0 {
                    draw_oam = true;
                    oam_pixel = oam_item.pixel;
                    oam_palette = oam_item.palette;
                    sprite_behind_bg = oam_item.bg_priority;
                }
            }
        }

        if (self.position_in_line as u8).wrapping_add(16) < 8 {
            if self.position_in_line == -17 {
                self.position_in_line = -16;
            } else if (self.position_in_line & 7) == (self.scx & 7) as i16 {
                self.position_in_line = -8;
            } else if self.window_is_being_fetched && (self.position_in_line & 7) == 6 && (self.scx & 7) == 7 {
                self.position_in_line = -8;
            } else if self.position_in_line == -9 {
                self.position_in_line = -16;
                return;
            }
        }

        self.window_is_being_fetched = false;

        if self.position_in_line >= 160 {
            self.position_in_line += 1;
            return;
        }

        if fifo_item.pixel != 0 && sprite_behind_bg {
            draw_oam = false;
        }

        let bg_enabled = (self.lcdc & 0x01) != 0;
        let mut pixel = if bg_enabled {
            fifo_item.pixel
        } else {
            0
        };

        pixel = self.apply_palette(pixel, self.bgp);

        if self.position_in_line >= 0 && self.position_in_line < 160 {
            let pixel_idx = (self.current_line as usize * 160) + self.position_in_line as usize;
            if pixel_idx < 160 * 144 {
                if draw_oam && oam_pixel != 0 {
                    let palette_reg = if oam_palette != 0 { self.obp1 } else { self.obp0 };
                    let sprite_pixel = self.apply_palette(oam_pixel, palette_reg);
                    self.framebuffer[pixel_idx] = sprite_pixel;
                } else {
                    self.framebuffer[pixel_idx] = pixel;
                }
                self.bg_priority[pixel_idx] = bg_priority;
            }
        }

        // SameBoy linha 798-799: sempre incrementa ambos
        self.position_in_line += 1;
        self.lcd_x += 1;
    }

    fn handle_objects_during_mode3(&mut self) {
        // Compute x_match robustly (signed math to avoid wraps)
        let x_match: u8 = {
            let xm = self.position_in_line + 8;
            if xm < 0 {
                0u8
            } else if xm > 255 {
                0u8
            } else {
                xm as u8
            }
        };

        // Drop sprites that are left of x_match (already passed)
        // Process from front since we sort ascending (smallest X first)
        while !self.visible_sprites.is_empty() {
            let first_x = self.visible_sprites[0].0.x;
            if first_x < x_match {
                self.visible_sprites.remove(0);
            } else {
                break;
            }
        }

        self.during_object_fetch = true;

        // Process sprites whose x == x_match, from the front
        while !self.visible_sprites.is_empty() {
            let sprite_x = self.visible_sprites[0].0.x;

            if (self.lcdc & 0x02) == 0 {
                break;
            }

            if sprite_x != x_match {
                break;
            }

            while (self.fetcher_state as u8) < (FetcherState::GetTileDataHighT2 as u8) || self.bg_fifo.is_empty() {
                self.advance_fetcher();
                self.cycles_for_line += 1;
            }

            self.advance_fetcher();
            self.cycles_for_line += 1;

            let (sprite, sprite_idx) = self.visible_sprites[0];
            let sprite_tile = sprite.tile_index;
            let sprite_flags = sprite.attributes;
            let sprite_y = sprite.y;

            let object_line_addr = self.get_object_line_address(sprite_y, sprite_tile, sprite_flags);
            let object_tile_data_0 = self.vram[object_line_addr.min(0x1FFF)];

            self.during_object_fetch = false;
            self.cycles_for_line += 1;
            let object_line_addr2 = self.get_object_line_address(sprite_y, sprite_tile, sprite_flags);
            let object_tile_data_1 = self.vram[(object_line_addr2 + 1).min(0x1FFF)];

            let palette = if (sprite_flags & 0x10) != 0 { 1 } else { 0 };
            let bg_priority_sprite = (sprite_flags & 0x80) != 0;
            let priority = sprite_idx;
            let flip_x = (sprite_flags & 0x20) != 0;

            self.fifo_overlay_object_row(object_tile_data_0, object_tile_data_1, palette, bg_priority_sprite, priority, flip_x);

            self.visible_sprites.remove(0);
        }

        self.during_object_fetch = false;
    }

    fn fifo_overlay_object_row(&mut self, lower: u8, upper: u8, palette: u8, bg_priority: bool, priority: u8, flip_x: bool) {
        while self.oam_fifo.size < 8 {
            self.oam_fifo.buffer[(self.oam_fifo.read_end + self.oam_fifo.size) & 7] = FifoItem {
                pixel: 0,
                bg_priority: false,
                palette: 0,
                priority: 0xFF,
            };
            self.oam_fifo.size += 1;
        }

        let flip_xor = if flip_x { 0 } else { 7 };
        let mut lower_work = lower;
        let mut upper_work = upper;
        let read_end = self.oam_fifo.read_end;

        for i in (0..8).rev() {
            let pixel = (lower_work >> 7) | ((upper_work >> 7) << 1);
            let target_idx = (read_end + (i ^ flip_xor)) & 7;
            let target = &mut self.oam_fifo.buffer[target_idx];

            if pixel != 0 && (target.pixel == 0 || target.priority > priority) {
                target.pixel = pixel;
                target.palette = palette;
                target.bg_priority = bg_priority;
                target.priority = priority;
            }

            lower_work <<= 1;
            upper_work <<= 1;
        }
    }

    fn get_object_line_address(&self, oam_y: u8, tile: u8, flags: u8) -> usize {
        let height_16 = (self.lcdc & 0x04) != 0;
        // OAM y = true_y + 16, então sprite_top = oam_y - 16
        let sprite_top = oam_y.wrapping_sub(16);
        // linha dentro do sprite (0..7 ou 0..15)
        let mut tile_y = self.current_line.wrapping_sub(sprite_top) & if height_16 { 0xF } else { 7 };

        if (flags & 0x40) != 0 {
            tile_y ^= if height_16 { 0xF } else { 7 };
        }

        let tile_index = if height_16 { (tile & 0xFE) as usize } else { tile as usize };
        (tile_index * 16) + (tile_y as usize * 2)
    }
}
