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

pub struct PPU {
    // VRAM (Video RAM) - 8KB (0x8000-0x9FFF)
    // 0x8000-0x97FF: Tile data (384 tiles × 16 bytes = 6KB)
    // 0x9800-0x9BFF: Tile map 0 (32×32 = 1KB)
    // 0x9C00-0x9FFF: Tile map 1 (32×32 = 1KB)
    pub vram: [u8; 0x2000],

    // Framebuffer - 160×144 pixels, cada pixel = 0-3 (2 bits por cor)
    pub framebuffer: [u8; 160 * 144],

    // Registradores PPU (endereços I/O)
    pub lcdc: u8, // 0xFF40 - LCD Control
    pub stat: u8, // 0xFF41 - LCD Status
    pub scy: u8,  // 0xFF42 - Scroll Y
    pub scx: u8,  // 0xFF43 - Scroll X
    pub ly: u8,   // 0xFF44 - Line Y (linha atual sendo renderizada)
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
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: [0; 0x2000],
            framebuffer: [0; 160 * 144],
            lcdc: 0x91, // Default pós-boot: LCD on, BG on, 8x8 sprites
            stat: 0x00,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC, // Default: cores 3,3,2,1,0 = branco,branco,cinza claro,cinza escuro
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            oam: [0; 160],
            frame_ready: false,
            mode: 2, // Começa em OAM Search
            mode_clock: 0,
            wy_trigger: false,
            wy_pos: -1,
        }
    }

    // Verifica se deve gerar LCD STAT interrupt
    // STAT bits: 7=n/a, 6=LYC=LY, 5=Mode2, 4=Mode1, 3=Mode0, 2=LYC flag, 1-0=modo atual
    pub fn check_stat_interrupt(&self) -> bool {
        // Bit 4: VBlank interrupt enabled (Mode 1)
        // Verifica se estamos em VBlank (ly >= 144) e interrupt habilitado
        if self.ly >= 144 && (self.stat & 0x10) != 0 {
            return true;
        }

        // Bit 6: LYC=LY coincidence interrupt
        if self.ly == self.lyc && (self.stat & 0x40) != 0 {
            return true;
        }

        false
    }

    // Atualiza flag LYC=LY (bit 2 do STAT)
    pub fn update_lyc_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04; // Seta bit 2
        } else {
            self.stat &= !0x04; // Limpa bit 2
        }
    }

    // Lê sprite do OAM (índice 0-39)
    fn get_sprite(&self, sprite_index: u8) -> Sprite {
        let base = (sprite_index as usize) * 4;
        Sprite {
            y: self.oam[base],
            x: self.oam[base + 1],
            tile_index: self.oam[base + 2],
            attributes: self.oam[base + 3],
        }
    }

    // Aplica paleta OBP0 ou OBP1 (similar ao BGP)
    fn apply_sprite_palette(&self, color: u8, use_obp1: bool) -> u8 {
        let palette = if use_obp1 { self.obp1 } else { self.obp0 };
        let shift = color * 2;
        (palette >> shift) & 0x03
    }

    // Renderiza window layer para uma scanline específica
    pub fn render_window_scanline(&mut self) {
        // LCDC bit 5: Window enable
        if (self.lcdc & 0x20) == 0 {
            return; // Window desabilitada
        }

        // LCDC bit 0: BG/Window enable (ambos precisam estar on)
        if (self.lcdc & 0x01) == 0 {
            return;
        }

        // Window só aparece se WY <= LY (janela começou)
        if self.wy > self.ly {
            return;
        }

        // LCDC bit 6: Window tile map select
        // 0 = 0x9800-0x9BFF, 1 = 0x9C00-0x9FFF
        let tile_map_base = if (self.lcdc & 0x40) != 0 {
            0x1C00 // Offset em VRAM (0x9C00 - 0x8000)
        } else {
            0x1800 // Offset em VRAM (0x9800 - 0x8000)
        };

        // LCDC bit 4: BG/Window tile data select (mesmo que BG)
        let tile_data_mode = (self.lcdc & 0x10) != 0;

        // incrementa wy_pos se window está ativa
        let wx_trigger = self.wx <= 166;
        let win_line = if self.wy_trigger && wx_trigger {
            self.wy_pos += 1;
            self.wy_pos
        } else {
            -1
        };

        // Calcular linha da window (sem scroll)
        let window_y = if win_line >= 0 { win_line as u8 } else { self.ly - self.wy };
        let tile_y = (window_y / 8) as usize;
        let pixel_y = (window_y % 8) as usize;

        let line_start = self.ly as usize * 160;

        // WX é offset por 7, então WX=7 significa coluna 0
        let window_start_x = if self.wx >= 7 { self.wx - 7 } else { 0 };

        for screen_x in window_start_x..160 {
            // Posição dentro da window (sem offset WX)
            let window_x = screen_x - window_start_x;
            let tile_x = (window_x / 8) as usize;
            let pixel_x = (window_x % 8) as usize;

            // Obter tile index do tile map
            let tile_map_addr = tile_map_base + tile_y * 32 + tile_x;
            if tile_map_addr >= 0x2000 {
                continue;
            }
            let tile_index = self.vram[tile_map_addr];

            // Calcular endereço do tile
            let tile_addr = if tile_data_mode {
                // Modo unsigned: 0x8000 + index * 16
                (tile_index as u16) * 16
            } else {
                // Modo signed: 0x9000 + (signed_index * 16)
                let signed = tile_index as i8;
                (0x1000u16 as i16 + (signed as i16) * 16) as u16
            };

            if tile_addr + (pixel_y as u16) * 2 + 1 >= 0x2000 {
                continue;
            }

            // Ler linha do tile
            let byte1 = self.vram[(tile_addr + (pixel_y as u16) * 2) as usize];
            let byte2 = self.vram[(tile_addr + (pixel_y as u16) * 2 + 1) as usize];

            // Extrair cor do pixel
            let bit_pos = 7 - pixel_x;
            let bit1 = (byte1 >> bit_pos) & 1;
            let bit2 = (byte2 >> bit_pos) & 1;
            let color = (bit2 << 1) | bit1;

            // Aplicar paleta BGP (window usa mesma paleta que BG)
            let final_color = self.apply_palette(color);
            self.framebuffer[line_start + screen_x as usize] = final_color;
        }
    }

    // Renderiza sprites para uma scanline específica
    pub fn render_sprites_scanline(&mut self, line: u8) {
        // Verificar se sprites estão habilitados (bit 1 do LCDC)
        if (self.lcdc & 0x02) == 0 {
            return;
        }

        // Coletar sprites visíveis nesta linha (máximo 10 por linha no hardware)
        let mut visible_sprites = Vec::new();
        let sprite_height = if (self.lcdc & 0x04) != 0 { 16 } else { 8 }; // 8x8 ou 8x16

        for sprite_index in 0..40 {
            let sprite = self.get_sprite(sprite_index);

            // Sprite Y é offset por 16, então Y=16 significa linha 0
            let sprite_y = (sprite.y as i16) - 16;

            // Verificar se sprite está visível nesta linha
            if (line as i16) >= sprite_y && (line as i16) < sprite_y + sprite_height as i16 {
                visible_sprites.push((sprite, sprite_index));

                // Hardware GB limita a 10 sprites por linha
                if visible_sprites.len() >= 10 {
                    break;
                }
            }
        }

        // Renderizar sprites em ordem reversa (últimos tem prioridade)
        for &(sprite, _sprite_index) in visible_sprites.iter().rev() {
            self.render_single_sprite(sprite, line, sprite_height);
        }
    }

    // Renderiza um único sprite na scanline
    fn render_single_sprite(&mut self, sprite: Sprite, line: u8, sprite_height: u8) {
        let sprite_y = sprite.y.wrapping_sub(16);
        let sprite_x = sprite.x.wrapping_sub(8);

        // Calcular linha do tile (0-7 para 8x8, 0-15 para 8x16)
        let mut tile_line = line.wrapping_sub(sprite_y);

        // Flip vertical (bit 6)
        if (sprite.attributes & 0x40) != 0 {
            tile_line = (sprite_height - 1) - tile_line;
        }

        // Obter tile index (para 8x16, bit 0 é ignorado)
        let tile_index = if sprite_height == 16 {
            sprite.tile_index & 0xFE
        } else {
            sprite.tile_index
        };

        // Calcular endereço do tile (sprites sempre usam 0x8000-0x8FFF)
        let tile_addr = (tile_index as u16) * 16 + (tile_line as u16) * 2;

        if tile_addr + 1 >= 0x2000 {
            return;
        } // Bounds check

        let byte1 = self.vram[tile_addr as usize];
        let byte2 = self.vram[(tile_addr + 1) as usize];

        // Renderizar 8 pixels da linha do sprite
        for pixel_x in 0..8 {
            let screen_x = sprite_x.wrapping_add(pixel_x);

            // Verificar bounds horizontais
            if screen_x >= 160 {
                continue;
            }

            // Calcular bit position (flip horizontal se bit 5 setado)
            let bit_pos = if (sprite.attributes & 0x20) != 0 {
                pixel_x // Flip horizontal
            } else {
                7 - pixel_x // Normal
            };

            // Extrair cor do pixel (2 bits por pixel)
            let bit1 = (byte1 >> bit_pos) & 1;
            let bit2 = (byte2 >> bit_pos) & 1;
            let color = (bit2 << 1) | bit1;

            // Cor 0 é transparente para sprites
            if color == 0 {
                continue;
            }

            // Verificar prioridade (bit 7 do atributo)
            let bg_priority = (sprite.attributes & 0x80) != 0;
            let framebuffer_pos = (line as usize) * 160 + (screen_x as usize);

            // Se sprite tem prioridade baixa, só desenha sobre cor 0 do BG
            if bg_priority && self.framebuffer[framebuffer_pos] != 0 {
                continue;
            }

            // Aplicar paleta (bit 4 escolhe OBP0 ou OBP1)
            let use_obp1 = (sprite.attributes & 0x10) != 0;
            let final_color = self.apply_sprite_palette(color, use_obp1);

            self.framebuffer[framebuffer_pos] = final_color;
        }
    }

    // Atualiza modo PPU no registrador STAT (bits 1-0)
    pub fn update_stat_mode(&mut self, mode: u8) {
        self.stat = (self.stat & 0xFC) | (mode & 0x03);
    }

    // Leitura de STAT (FF41)
    pub fn read_stat(&self) -> u8 {
        0x80 |
        (if (self.stat & 0x40) != 0 { 0x40 } else { 0 }) | // LYC=LY enable
        (if (self.stat & 0x20) != 0 { 0x20 } else { 0 }) | // Mode 2 enable
        (if (self.stat & 0x10) != 0 { 0x10 } else { 0 }) | // Mode 1 enable
        (if (self.stat & 0x08) != 0 { 0x08 } else { 0 }) | // Mode 0 enable
        (if self.ly == self.lyc { 0x04 } else { 0 }) |     // LYC coincidence
        (self.mode & 0x03)                                 // bits 0-1: modo atual
    }

    // Escrita de STAT (FF41) - só atualiza bits de enable
    pub fn write_stat(&mut self, val: u8) {
        self.stat = (self.stat & 0x07) | (val & 0xF8); // bits 0-2 são read-only
    }

    // Decodifica um tile (16 bytes → 8×8 pixels, 2bpp)
    // Cada linha de pixels = 2 bytes:
    //   byte1: bit baixo da cor (LSB)
    //   byte2: bit alto da cor (MSB)
    // Cor final = (bit_msb << 1) | bit_lsb  → 0-3
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

    // Aplica paleta BGP (0xFF47) a um valor de cor 0-3
    // BGP format: bits 7-6 = cor 3, 5-4 = cor 2, 3-2 = cor 1, 1-0 = cor 0
    // Retorna: 0-3 (intensidade final para display)
    pub fn apply_palette(&self, color: u8) -> u8 {
        let shift = color * 2;
        (self.bgp >> shift) & 0x03
    }

    // Renderiza uma scanline (linha) do background
    // ly = linha atual (0-143)
    // Escreve 160 pixels no framebuffer na posição correta
    pub fn render_bg_scanline(&mut self) {
        // LCDC bit 0: BG/Window enable
        if (self.lcdc & 0x01) == 0 {
            // BG desabilitado, preencher com branco (cor 0)
            let line_start = self.ly as usize * 160;
            for x in 0..160 {
                self.framebuffer[line_start + x] = 0;
            }
            return;
        }

        // LCDC bit 3: BG tile map select
        // 0 = 0x9800-0x9BFF, 1 = 0x9C00-0x9FFF
        let tile_map_base = if (self.lcdc & 0x08) != 0 {
            0x1C00 // Offset em VRAM (0x9C00 - 0x8000)
        } else {
            0x1800 // Offset em VRAM (0x9800 - 0x8000)
        };

        // LCDC bit 4: BG/Window tile data select
        // 0 = 0x8800-0x97FF (signed index, base 0x9000)
        // 1 = 0x8000-0x8FFF (unsigned index, base 0x8000)
        let tile_data_mode = (self.lcdc & 0x10) != 0;

        // Calcular posição Y no tile map (com scroll)
        let y = self.ly.wrapping_add(self.scy);
        let tile_y = (y / 8) as usize; // Qual linha de tiles (0-31)
        let pixel_y = (y % 8) as usize; // Offset dentro do tile (0-7)

        let line_start = self.ly as usize * 160;

        for screen_x in 0..160 {
            // Calcular posição X no tile map (com scroll)
            let x = (screen_x as u8).wrapping_add(self.scx);
            let tile_x = (x / 8) as usize; // Qual coluna de tiles (0-31)
            let pixel_x = (x % 8) as usize; // Offset dentro do tile (0-7)

            // Ler tile number do tile map
            let tile_map_addr = tile_map_base + tile_y * 32 + tile_x;
            let tile_number = self.vram[tile_map_addr];

            // Converter tile number para endereço em VRAM
            let tile_addr = if tile_data_mode {
                // Unsigned: 0-255 → tiles 0-255
                (tile_number as u16) * 16
            } else {
                // Signed: -128 a +127, base em 0x9000 (offset 0x1000 na VRAM)
                let signed = tile_number as i8;
                (0x1000u16 as i16 + (signed as i16) * 16) as u16
            };

            // Ler 2 bytes da linha do tile
            let byte1 = self.vram[(tile_addr + pixel_y as u16 * 2) as usize];
            let byte2 = self.vram[(tile_addr + pixel_y as u16 * 2 + 1) as usize];

            // Extrair pixel
            let bit_index = 7 - pixel_x;
            let lsb = (byte1 >> bit_index) & 1;
            let msb = (byte2 >> bit_index) & 1;
            let color = (msb << 1) | lsb;

            // Aplicar paleta e escrever no framebuffer
            let final_color = self.apply_palette(color);
            self.framebuffer[line_start + screen_x] = final_color;
        }
    }

    // Renderiza frame completo (144 scanlines)
    pub fn render_frame(&mut self) {
        for line in 0..144 {
            self.ly = line;
            self.render_bg_scanline();
        }
    }

    // Lê byte da VRAM (endereço 0x8000-0x9FFF)
    pub fn read_vram(&self, addr: u16) -> u8 {
        let offset = (addr - 0x8000) as usize;
        if offset < 0x2000 {
            self.vram[offset]
        } else {
            0xFF
        }
    }

    // Escreve byte na VRAM
    pub fn write_vram(&mut self, addr: u16, val: u8) {
        let offset = (addr - 0x8000) as usize;
        if offset < 0x2000 {
            self.vram[offset] = val;
        }
    }

    // Lê byte da OAM (endereço 0xFE00-0xFE9F)
    pub fn read_oam(&self, addr: u16) -> u8 {
        let offset = (addr - 0xFE00) as usize;
        if offset < 160 { self.oam[offset] } else { 0xFF }
    }

    // Escreve byte na OAM
    pub fn write_oam(&mut self, addr: u16, val: u8) {
        let offset = (addr - 0xFE00) as usize;
        if offset < 160 {
            self.oam[offset] = val;
        }
    }

    // Lê registrador PPU (0xFF40-0xFF4B)
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.read_stat(), // Usar função de leitura de STAT
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

    // Escreve registrador PPU
    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.lcdc = val,
            0xFF41 => self.write_stat(val),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {}, // LY é read-only
            0xFF45 => {
                self.lyc = val;
                // Dispara STAT IRQ se necessário
                // Precisa de acesso ao iflags, então pode ser ajustado para receber &mut u8 se necessário
                // Aqui, só marca flag interna, IRQ é disparado em step
            },
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {}
        }
    }

    /// Avança PPU em `cycles` ciclos de CPU (4MHz → 456 ciclos por linha, 154 linhas)
    pub fn step(&mut self, cycles: u32, iflags: &mut u8) {
        // Nova lógica baseada em mode_clock/mode
        if (self.lcdc & 0x80) == 0 {
            // LCD off: reset PPU state
            self.mode = 0;
            self.mode_clock = 0;
            self.ly = 0;
            self.frame_ready = false;
            return;
        }

        self.mode_clock += cycles;

        if self.ly < 144 {
            if self.mode_clock <= 80 {
                if self.mode != 2 { self.change_mode(2, iflags); }
            } else if self.mode_clock <= 252 {
                if self.mode != 3 { self.change_mode(3, iflags); }
            } else if self.mode_clock < 456 {
                if self.mode != 0 { self.change_mode(0, iflags); }
            }
        } else {
            if self.mode != 1 { self.change_mode(1, iflags); }
        }

        if self.mode_clock >= 456 {
            self.mode_clock -= 456;
            self.ly = (self.ly + 1) % 154;
            self.update_lyc_flag();
            self.check_lyc_interrupt(iflags);
            if self.ly >= 144 && self.mode != 1 {
                self.change_mode(1, iflags);
            }
        }
    }

    // Centraliza mudança de modo, IRQs e ações do PPU
    pub fn change_mode(&mut self, new_mode: u8, iflags: &mut u8) {
        self.mode = new_mode;
        self.update_stat_mode(new_mode);

        let stat_irq = match new_mode {
            0 => {
                // HBlank: renderiza scanline
                self.render_bg_scanline();
                self.render_window_scanline();
                self.render_sprites_scanline(self.ly);
                (self.stat & 0x08) != 0
            }
            1 => {
                self.frame_ready = true;
                *iflags |= 0x01;
                (self.stat & 0x10) != 0
            }
            2 => {
                (self.stat & 0x20) != 0
            }
            3 => {
                // Window trigger: ativa ao entrar em modo 3 na linha wy
                if (self.lcdc & 0x20) != 0 && !self.wy_trigger && self.ly == self.wy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }
                false
            }
            _ => false,
        };

        if stat_irq {
            *iflags |= 0x02; // LCD STAT
        }
    }

    /// Dispara STAT IRQ se lyc_inte estiver setado e ly == lyc
    pub fn check_lyc_interrupt(&mut self, iflags: &mut u8) {
        // Bit 6: LYC=LY coincidence interrupt enable
        let lyc_inte = (self.stat & 0x40) != 0;
        if lyc_inte && self.ly == self.lyc {
            *iflags |= 0x02; // LCD STAT
        }
    }
}
