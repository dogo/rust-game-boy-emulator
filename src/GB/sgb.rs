//! Super Game Boy commands used for joypad detection and color output.

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 224;
const GAME_WIDTH: usize = 160;
const GAME_HEIGHT: usize = 144;

#[derive(Clone, Copy)]
enum Transfer {
    Palettes,
    Tiles(usize),
    Border,
}

pub struct Sgb {
    commands_enabled: bool,
    receiving: bool,
    pulse: Option<u8>,
    bits: usize,
    packets: usize,
    expected_packets: usize,
    packet: [u8; 112],
    select: u8,
    players: u8,
    player: u8,
    pending_transfer: Option<Transfer>,
    system_palettes: [[u16; 4]; 512],
    palettes: [[u16; 4]; 4],
    attributes: [u8; 20 * 18],
    border_tiles: [[u8; 32]; 256],
    border_map: [u16; 32 * 28],
    border_palettes: [[u16; 16]; 3],
    border_ready: bool,
    border_rgb: Vec<u8>,
    border_opaque: Vec<bool>,
    game_rgb: Vec<u8>,
    frame_rgb: Vec<u8>,
    mask: u8,
}

impl Sgb {
    pub fn new(commands_enabled: bool) -> Self {
        Self {
            commands_enabled,
            receiving: false,
            pulse: None,
            bits: 0,
            packets: 0,
            expected_packets: 0,
            packet: [0; 112],
            select: 0x30,
            players: 1,
            player: 0,
            pending_transfer: None,
            system_palettes: [[0; 4]; 512],
            palettes: [[0x7fff, 0x56b5, 0x294a, 0]; 4],
            attributes: [0; 20 * 18],
            border_tiles: [[0; 32]; 256],
            border_map: [0; 32 * 28],
            border_palettes: [[0; 16]; 3],
            border_ready: false,
            border_rgb: vec![0; WIDTH * HEIGHT * 3],
            border_opaque: vec![false; WIDTH * HEIGHT],
            game_rgb: vec![0; GAME_WIDTH * GAME_HEIGHT * 3],
            frame_rgb: vec![0; WIDTH * HEIGHT * 3],
            mask: 0,
        }
    }

    pub fn read_joyp(&self, value: u8) -> u8 {
        if self.commands_enabled && self.players > 1 && self.select == 0x30 {
            (value & 0xf0) | (0x0f - self.player)
        } else {
            value
        }
    }

    pub fn write_joyp(&mut self, value: u8) {
        if !self.commands_enabled {
            return;
        }
        let select = value & 0x30;
        if select == 0x30 && self.select != 0x30 && !self.receiving && self.players > 1 {
            self.player = (self.player + 1) % self.players;
        }
        self.select = select;
        match select {
            0x00 => {
                if self.packets >= 7 {
                    self.packets = 0;
                }
                self.packet[self.packets * 16..self.packets * 16 + 16].fill(0);
                self.receiving = true;
                self.bits = 0;
                self.pulse = None;
            }
            0x10 | 0x20 if self.receiving => self.pulse = Some(u8::from(select == 0x10)),
            0x30 if self.receiving => {
                if let Some(bit) = self.pulse.take() {
                    if self.bits < 128 {
                        let byte = self.packets * 16 + self.bits / 8;
                        self.packet[byte] |= bit << (self.bits % 8);
                        self.bits += 1;
                    } else {
                        self.receiving = false;
                        if bit == 0 {
                            self.finish_packet();
                        } else {
                            self.packets = 0;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn finish_packet(&mut self) {
        self.packets += 1;
        if self.packets == 1 {
            self.expected_packets = (self.packet[0] & 7) as usize;
        }
        if self.expected_packets == 0 || self.expected_packets > 7 {
            self.packets = 0;
        } else if self.packets == self.expected_packets {
            let packet = self.packet;
            self.execute(&packet[..self.packets * 16]);
            self.packets = 0;
        }
    }

    fn execute(&mut self, packet: &[u8]) {
        match packet[0] >> 3 {
            command @ 0x00..=0x03 => {
                let (a, b) = [(0, 1), (2, 3), (0, 3), (1, 2)][command as usize];
                let color = |i: usize| u16::from_le_bytes([packet[1 + i * 2], packet[2 + i * 2]]);
                // A cor 0 é uma só para todas as paletas: vale a última enviada
                for palette in &mut self.palettes {
                    palette[0] = color(0);
                }
                for c in 1..4 {
                    self.palettes[a][c] = color(c);
                    self.palettes[b][c] = color(c + 3);
                }
                if self.border_ready {
                    self.render_border();
                }
            }
            0x04 => self.attribute_block(packet),
            0x0a => {
                for palette in 0..4 {
                    let offset = 1 + palette * 2;
                    let index = u16::from_le_bytes([packet[offset], packet[offset + 1]]) as usize;
                    if let Some(colors) = self.system_palettes.get(index) {
                        self.palettes[palette] = *colors;
                    }
                }
                if packet[9] & 0x40 != 0 {
                    self.mask = 0;
                }
                if self.border_ready {
                    self.render_border();
                }
            }
            0x0b => self.pending_transfer = Some(Transfer::Palettes),
            0x11 => {
                self.players = match packet[1] & 3 {
                    1 => 2,
                    3 => 4,
                    _ => 1,
                };
                self.player = u8::from(self.players > 1);
            }
            0x13 => self.pending_transfer = Some(Transfer::Tiles((packet[1] & 1) as usize)),
            0x14 => self.pending_transfer = Some(Transfer::Border),
            0x17 => self.mask = packet[1] & 3,
            // shortcut: ignora ATTR_LIN, ATTR_DIV, ATTR_CHR, ATTR_TRN, ATTR_SET (e o ATF do PAL_SET) e os
            // comandos do lado SNES (som, DATA_SND/DATA_TRN, JUMP); emular se um jogo depender deles.
            _ => {}
        }
    }

    fn attribute_block(&mut self, packet: &[u8]) {
        for block in packet[2..].chunks_exact(6).take(packet[1] as usize) {
            let (x1, y1, x2, y2) = (
                block[2] as usize,
                block[3] as usize,
                block[4] as usize,
                block[5] as usize,
            );
            let control = block[0] & 7;
            // Mudando só o interior ou só o exterior, a linha recebe a mesma paleta
            let (line_changes, line_palette) = match control {
                1 => (true, block[1]),
                4 => (true, block[1] >> 4),
                _ => (control & 2 != 0, block[1] >> 2),
            };
            for y in 0..18 {
                for x in 0..20 {
                    let (changes, palette) = if x < x1 || x > x2 || y < y1 || y > y2 {
                        (control & 4 != 0, block[1] >> 4)
                    } else if x == x1 || x == x2 || y == y1 || y == y2 {
                        (line_changes, line_palette)
                    } else {
                        (control & 1 != 0, block[1])
                    };
                    if changes {
                        self.attributes[y * 20 + x] = palette & 3;
                    }
                }
            }
        }
    }

    /// Completes a transfer from the Game Boy video signal at VBlank.
    pub fn on_frame(&mut self, shades: &[u8; GAME_WIDTH * GAME_HEIGHT]) {
        let Some(transfer) = self.pending_transfer.take() else {
            return;
        };
        // shortcut: usa um quadro, modelar os cinco quadros de transferência se o jogo mudar a imagem durante o envio.
        // O SGB reconstrói 256 tiles de 2 bpp a partir da imagem visível, 20 tiles por linha.
        let mut data = [0u8; 0x1000];
        for tile in 0..256 {
            let (tile_x, tile_y) = (tile % 20, tile / 20);
            for row in 0..8 {
                let mut low = 0;
                let mut high = 0;
                for x in 0..8 {
                    let shade = shades[(tile_y * 8 + row) * GAME_WIDTH + tile_x * 8 + x];
                    low |= (shade & 1) << (7 - x);
                    high |= ((shade >> 1) & 1) << (7 - x);
                }
                data[tile * 16 + row * 2] = low;
                data[tile * 16 + row * 2 + 1] = high;
            }
        }
        match transfer {
            Transfer::Palettes => {
                for (i, colors) in self.system_palettes.iter_mut().enumerate() {
                    for (j, color) in colors.iter_mut().enumerate() {
                        let offset = i * 8 + j * 2;
                        *color = u16::from_le_bytes([data[offset], data[offset + 1]]);
                    }
                }
            }
            Transfer::Tiles(half) => {
                for i in 0..128 {
                    self.border_tiles[half * 128 + i].copy_from_slice(&data[i * 32..i * 32 + 32]);
                }
                if self.border_ready {
                    self.render_border();
                }
            }
            Transfer::Border => {
                for (i, entry) in self.border_map.iter_mut().enumerate() {
                    *entry = u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
                }
                for p in 0..3 {
                    for c in 0..16 {
                        let offset = 0x800 + (p * 16 + c) * 2;
                        self.border_palettes[p][c] =
                            u16::from_le_bytes([data[offset], data[offset + 1]]);
                    }
                }
                self.border_ready = true;
                self.render_border();
            }
        }
    }

    fn render_border(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let entry = self.border_map[(y / 8) * 32 + x / 8];
                let tile = &self.border_tiles[(entry & 0xff) as usize];
                let row = if entry & 0x8000 != 0 {
                    7 - y % 8
                } else {
                    y % 8
                };
                let bit = if entry & 0x4000 != 0 {
                    x % 8
                } else {
                    7 - x % 8
                };
                let color = ((tile[row * 2] >> bit) & 1)
                    | (((tile[row * 2 + 1] >> bit) & 1) << 1)
                    | (((tile[16 + row * 2] >> bit) & 1) << 2)
                    | (((tile[17 + row * 2] >> bit) & 1) << 3);
                let palette = ((entry >> 10) & 7).saturating_sub(4).min(2) as usize;
                let offset = (y * WIDTH + x) * 3;
                let rgb = if color == 0 {
                    rgb555(self.palettes[0][0])
                } else {
                    rgb555(self.border_palettes[palette][color as usize])
                };
                self.border_rgb[offset..offset + 3].copy_from_slice(&rgb);
                self.border_opaque[y * WIDTH + x] = color != 0;
            }
        }
    }

    pub fn render(&mut self, shades: &[u8; GAME_WIDTH * GAME_HEIGHT]) -> &[u8] {
        if self.mask == 0 {
            for y in 0..GAME_HEIGHT {
                for x in 0..GAME_WIDTH {
                    let palette = self.attributes[(y / 8) * 20 + x / 8] as usize;
                    let shade = (shades[y * GAME_WIDTH + x] & 3) as usize;
                    let offset = (y * GAME_WIDTH + x) * 3;
                    // A cor 0 de toda paleta mostra o fundo comum
                    let color = self.palettes[if shade == 0 { 0 } else { palette }][shade];
                    self.game_rgb[offset..offset + 3].copy_from_slice(&rgb555(color));
                }
            }
        }
        self.frame_rgb.copy_from_slice(&self.border_rgb);
        let masked = rgb555(self.palettes[0][0]);
        for y in 0..GAME_HEIGHT {
            for x in 0..GAME_WIDTH {
                let target = (y + 40) * WIDTH + x + 48;
                if self.border_ready && self.border_opaque[target] {
                    continue;
                }
                let dst = target * 3;
                let src = (y * GAME_WIDTH + x) * 3;
                match self.mask {
                    2 => self.frame_rgb[dst..dst + 3].fill(0),
                    3 => self.frame_rgb[dst..dst + 3].copy_from_slice(&masked),
                    _ => self.frame_rgb[dst..dst + 3].copy_from_slice(&self.game_rgb[src..src + 3]),
                }
            }
        }
        &self.frame_rgb
    }
}

fn rgb555(color: u16) -> [u8; 3] {
    let expand = |channel: u16| ((channel * 255 + 15) / 31) as u8;
    [
        expand(color & 31),
        expand((color >> 5) & 31),
        expand((color >> 10) & 31),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(sgb: &mut Sgb, packet: &[u8; 16]) {
        sgb.write_joyp(0);
        sgb.write_joyp(0x30);
        for &byte in packet {
            for bit in 0..8 {
                sgb.write_joyp(if byte & (1 << bit) != 0 { 0x10 } else { 0x20 });
                sgb.write_joyp(0x30);
            }
        }
        sgb.write_joyp(0x20);
        sgb.write_joyp(0x30);
    }

    fn transfer(sgb: &mut Sgb, data: &[u8]) {
        let mut shades = [0; GAME_WIDTH * GAME_HEIGHT];
        for tile in 0..256 {
            for row in 0..8 {
                let low = data[tile * 16 + row * 2];
                let high = data[tile * 16 + row * 2 + 1];
                for x in 0..8 {
                    let bit = 7 - x;
                    shades[((tile / 20) * 8 + row) * GAME_WIDTH + (tile % 20) * 8 + x] =
                        ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
                }
            }
        }
        sgb.on_frame(&shades);
    }

    #[test]
    fn multiplayer_packet_answers_detection() {
        let mut sgb = Sgb::new(true);
        let mut packet = [0; 16];
        packet[0] = 0x89;
        packet[1] = 1;
        send(&mut sgb, &packet);
        assert_eq!(sgb.read_joyp(0xff), 0xfe);
        sgb.write_joyp(0x20);
        sgb.write_joyp(0x30);
        assert_eq!(sgb.read_joyp(0xff), 0xff);

        let mut disabled = Sgb::new(false);
        send(&mut disabled, &packet);
        assert_eq!(disabled.read_joyp(0xff), 0xff);
    }

    #[test]
    fn sgb_model_connects_packet_receiver_to_bus() {
        use crate::GB::CPU::{BootModel, CPU};

        let mut rom = vec![0; 0x8000];
        rom[0x0146] = 0x03;
        rom[0x014b] = 0x33;
        let mut cpu = CPU::new_with_model(rom, BootModel::Sgb);
        cpu.bus.load_boot_rom(vec![0; 0x100]);
        let mut packet = [0; 16];
        packet[0] = 0x89;
        packet[1] = 1;
        cpu.bus.write(0xff00, 0);
        cpu.bus.write(0xff00, 0x30);
        for &byte in &packet {
            for bit in 0..8 {
                cpu.bus
                    .write(0xff00, if byte & (1 << bit) != 0 { 0x10 } else { 0x20 });
                cpu.bus.write(0xff00, 0x30);
            }
        }
        cpu.bus.write(0xff00, 0x20);
        cpu.bus.write(0xff00, 0x30);
        assert_eq!(cpu.bus.read(0xff00), 0xfe);
    }

    #[test]
    fn multi_packet_attribute_command_crosses_packet_boundary() {
        let mut sgb = Sgb::new(true);
        let mut bytes = [0; 32];
        bytes[0] = 0x22;
        bytes[1] = 3;
        bytes[14..20].copy_from_slice(&[4, 0x30, 1, 1, 18, 16]);
        send(&mut sgb, bytes[..16].try_into().unwrap());
        assert_eq!(sgb.attributes[0], 0);
        send(&mut sgb, bytes[16..].try_into().unwrap());
        assert_eq!(sgb.attributes[0], 3);
    }

    #[test]
    fn attribute_block_changing_one_side_also_changes_the_line() {
        let mut sgb = Sgb::new(true);
        let mut packet = [0; 16];
        packet[..8].copy_from_slice(&[0x21, 1, 1, 0x01, 2, 2, 5, 5]); // só dentro = paleta 1
        send(&mut sgb, &packet);
        assert_eq!(sgb.attributes[2 * 20 + 2], 1, "linha");
        assert_eq!(sgb.attributes[3 * 20 + 3], 1, "dentro");
        assert_eq!(sgb.attributes[0], 0, "fora não muda");

        packet[..8].copy_from_slice(&[0x21, 1, 4, 0x20, 2, 2, 5, 5]); // só fora = paleta 2
        send(&mut sgb, &packet);
        assert_eq!(sgb.attributes[0], 2, "fora");
        assert_eq!(sgb.attributes[2 * 20 + 2], 2, "linha");
        assert_eq!(sgb.attributes[3 * 20 + 3], 1, "dentro não muda");

        packet[..8].copy_from_slice(&[0x21, 1, 3, 0x03, 2, 2, 5, 5]); // dentro e linha = paleta 3
        send(&mut sgb, &packet);
        assert_eq!(
            sgb.attributes[2 * 20 + 2],
            0,
            "linha com sua própria paleta"
        );
        assert_eq!(sgb.attributes[3 * 20 + 3], 3, "dentro");
    }

    #[test]
    fn pal_commands_set_two_palettes_and_share_color_zero() {
        let mut sgb = Sgb::new(true);
        let mut packet = [0; 16];
        packet[0] = 0x09; // PAL23
        packet[1..3].copy_from_slice(&0x001fu16.to_le_bytes()); // cor 0: vermelho
        packet[3..5].copy_from_slice(&0x03e0u16.to_le_bytes()); // paleta 2, cor 1: verde
        packet[9..11].copy_from_slice(&0x7c00u16.to_le_bytes()); // paleta 3, cor 1: azul
        send(&mut sgb, &packet);
        packet = [0; 16];
        packet[..8].copy_from_slice(&[0x21, 1, 4, 0x20, 1, 1, 18, 16]); // fora = paleta 2
        send(&mut sgb, &packet);

        let pixel = |frame: &[u8], x: usize, y: usize| {
            let offset = ((y + 40) * WIDTH + x + 48) * 3;
            frame[offset..offset + 3].to_vec()
        };
        let frame = sgb.render(&[1; GAME_WIDTH * GAME_HEIGHT]);
        assert_eq!(pixel(frame, 0, 0), [0, 255, 0], "paleta 2, cor 1");
        assert_eq!(
            pixel(frame, 80, 72),
            [173, 173, 173],
            "paleta 0 mantém a cor 1"
        );
        let frame = sgb.render(&[0; GAME_WIDTH * GAME_HEIGHT]);
        assert_eq!(pixel(frame, 0, 0), [255, 0, 0], "cor 0 compartilhada");
        assert_eq!(pixel(frame, 80, 72), [255, 0, 0], "cor 0 compartilhada");
        assert_eq!(sgb.palettes[3][1], 0x7c00);

        // PAL_SET traz uma cor 0 por paleta, mas a tela mostra só a da paleta 0
        sgb.system_palettes[1][0] = 0x7c00;
        packet = [0; 16];
        packet[0] = 0x51; // PAL_SET: paleta 0 = sistema 0, paleta 1 = sistema 1
        packet[3] = 1;
        send(&mut sgb, &packet);
        packet = [0; 16];
        packet[..8].copy_from_slice(&[0x21, 1, 4, 0x10, 1, 1, 18, 16]); // fora = paleta 1
        send(&mut sgb, &packet);
        let frame = sgb.render(&[0; GAME_WIDTH * GAME_HEIGHT]);
        assert_eq!(pixel(frame, 0, 0), [0, 0, 0], "cor 0 da paleta 0");
    }

    #[test]
    fn palette_and_border_transfers_color_the_frame() {
        let mut sgb = Sgb::new(true);
        let mut vram = [0; 0x2000];
        vram[2..4].copy_from_slice(&0x001fu16.to_le_bytes());
        vram[10..12].copy_from_slice(&0x7c00u16.to_le_bytes());
        let mut packet = [0; 16];
        packet[0] = 0x59; // PAL_TRN
        send(&mut sgb, &packet);
        transfer(&mut sgb, &vram);

        packet = [0; 16];
        packet[0] = 0x51; // PAL_SET
        packet[3] = 1; // palette 1 uses system palette 1
        send(&mut sgb, &packet);
        packet = [0; 16];
        packet[..8].copy_from_slice(&[0x21, 1, 4, 0x10, 1, 1, 18, 16]); // outside = palette 1
        send(&mut sgb, &packet);
        let shades = [1; GAME_WIDTH * GAME_HEIGHT];
        let frame = sgb.render(&shades);
        assert_eq!(
            &frame[(40 * WIDTH + 48) * 3..(40 * WIDTH + 48) * 3 + 3],
            &[0, 0, 255]
        );
        assert_eq!(
            &frame[(56 * WIDTH + 64) * 3..(56 * WIDTH + 64) * 3 + 3],
            &[255, 0, 0]
        );

        vram.fill(0);
        for row in 0..8 {
            vram[row * 2] = 0xff;
        }
        packet = [0; 16];
        packet[0] = 0x99; // CHR_TRN, first 128 border tiles
        send(&mut sgb, &packet);
        transfer(&mut sgb, &vram);

        vram.fill(0);
        for entry in 0..32 * 28 {
            vram[entry * 2 + 1] = 0x10; // palette 4, tile 0
        }
        vram[0x802..0x804].copy_from_slice(&0x03e0u16.to_le_bytes());
        packet[0] = 0xa1; // PCT_TRN
        send(&mut sgb, &packet);
        transfer(&mut sgb, &vram);
        assert_eq!(&sgb.render(&shades)[..3], &[0, 255, 0]);
    }

    #[test]
    fn border_color_zero_follows_the_game_backdrop() {
        let mut sgb = Sgb::new(true);
        let mut vram = [0; 0x1000];
        for entry in 0..32 * 28 {
            vram[entry * 2 + 1] = 0x10; // palette 4, blank tile
        }
        let mut packet = [0; 16];
        packet[0] = 0xa1; // PCT_TRN, palette 4 color 0 stays black
        send(&mut sgb, &packet);
        transfer(&mut sgb, &vram);
        let shades = [0; GAME_WIDTH * GAME_HEIGHT];
        assert_eq!(&sgb.render(&shades)[..3], &[255, 255, 255]);

        sgb.system_palettes[1][0] = 0x001f;
        packet[0] = 0x51; // PAL_SET
        packet[1] = 1;
        send(&mut sgb, &packet);
        assert_eq!(&sgb.render(&shades)[..3], &[255, 0, 0]);
    }
}
