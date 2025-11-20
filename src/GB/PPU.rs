#![allow(non_snake_case)]

// PPU (Picture Processing Unit) - Renderização de gráficos do Game Boy
// Responsável por: Background, Window, Sprites, Paletas

pub struct PPU {
    // VRAM (Video RAM) - 8KB (0x8000-0x9FFF)
    // 0x8000-0x97FF: Tile data (384 tiles × 16 bytes = 6KB)
    // 0x9800-0x9BFF: Tile map 0 (32×32 = 1KB)
    // 0x9C00-0x9FFF: Tile map 1 (32×32 = 1KB)
    pub vram: [u8; 0x2000],

    // Framebuffer - 160×144 pixels, cada pixel = 0-3 (2 bits por cor)
    pub framebuffer: [u8; 160 * 144],

    // Registradores PPU (endereços I/O)
    pub lcdc: u8,  // 0xFF40 - LCD Control
    pub stat: u8,  // 0xFF41 - LCD Status
    pub scy: u8,   // 0xFF42 - Scroll Y
    pub scx: u8,   // 0xFF43 - Scroll X
    pub ly: u8,    // 0xFF44 - Line Y (linha atual sendo renderizada)
    pub lyc: u8,   // 0xFF45 - LY Compare
    pub bgp: u8,   // 0xFF47 - Background Palette
    pub obp0: u8,  // 0xFF48 - Object Palette 0
    pub obp1: u8,  // 0xFF49 - Object Palette 1
    pub wy: u8,    // 0xFF4A - Window Y
    pub wx: u8,    // 0xFF4B - Window X

    // OAM (Object Attribute Memory) - 160 bytes (40 sprites × 4 bytes)
    pub oam: [u8; 160],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            vram: [0; 0x2000],
            framebuffer: [0; 160 * 144],
            lcdc: 0x91,  // Default pós-boot: LCD on, BG on, 8x8 sprites
            stat: 0x00,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,   // Default: cores 3,3,2,1,0 = branco,branco,cinza claro,cinza escuro
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            oam: [0; 160],
        }
    }

    // Decodifica um tile (16 bytes → 8×8 pixels, 2bpp)
    // Cada linha de pixels = 2 bytes:
    //   byte1: bit baixo da cor (LSB)
    //   byte2: bit alto da cor (MSB)
    // Cor final = (bit_msb << 1) | bit_lsb  → 0-3
    pub fn decode_tile(&self, tile_index: u16) -> [u8; 64] {
        let mut pixels = [0u8; 64];
        let tile_addr = tile_index * 16;  // Cada tile = 16 bytes

        for y in 0..8 {
            let byte1 = self.vram[(tile_addr + y * 2) as usize];
            let byte2 = self.vram[(tile_addr + y * 2 + 1) as usize];

            for x in 0..8 {
                let bit_index = 7 - x;  // Pixels são MSB first
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
            0x1C00  // Offset em VRAM (0x9C00 - 0x8000)
        } else {
            0x1800  // Offset em VRAM (0x9800 - 0x8000)
        };

        // LCDC bit 4: BG/Window tile data select
        // 0 = 0x8800-0x97FF (signed index, base 0x9000)
        // 1 = 0x8000-0x8FFF (unsigned index, base 0x8000)
        let tile_data_mode = (self.lcdc & 0x10) != 0;

        // Calcular posição Y no tile map (com scroll)
        let y = self.ly.wrapping_add(self.scy);
        let tile_y = (y / 8) as usize;  // Qual linha de tiles (0-31)
        let pixel_y = (y % 8) as usize; // Offset dentro do tile (0-7)

        let line_start = self.ly as usize * 160;

        for screen_x in 0..160 {
            // Calcular posição X no tile map (com scroll)
            let x = (screen_x as u8).wrapping_add(self.scx);
            let tile_x = (x / 8) as usize;  // Qual coluna de tiles (0-31)
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
                ((0x1000u16 as i16 + (signed as i16) * 16) as u16)
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
        if offset < 160 {
            self.oam[offset]
        } else {
            0xFF
        }
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
            0xFF41 => self.stat | 0x80,  // Bit 7 sempre 1
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
            0xFF41 => self.stat = (self.stat & 0x07) | (val & 0xF8),  // Bits 0-2 são read-only
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {},  // LY é read-only
            0xFF45 => self.lyc = val,
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {},
        }
    }
}
