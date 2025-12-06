// tests/ppu_sprite_test.rs

// Test harness para reproduzir visualmente / por assert o problema de sprites recortados.

use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

use gb_emu::GB::PPU::PPU;

fn write_ppm(path: &Path, framebuffer: &[u8]) -> std::io::Result<()> {
    // framebuffer: 160*144, valores 0..3.
    let mut f = BufWriter::new(File::create(path)?);

    // PPM P3 header (ASCII). Map colors simples para tons de cinza.
    writeln!(f, "P3")?;
    writeln!(f, "160 144")?;
    writeln!(f, "255")?;

    for y in 0..144 {
        for x in 0..160 {
            let v = framebuffer[y * 160 + x];
            let rgb = match v {
                0 => (255u8, 255u8, 255u8), // branco
                1 => (192u8, 192u8, 192u8),
                2 => (96u8, 96u8, 96u8),
                3 => (0u8, 0u8, 0u8),
                _ => (255u8, 0u8, 255u8),
            };
            writeln!(f, "{} {} {}", rgb.0, rgb.1, rgb.2)?;
        }
    }
    Ok(())
}

#[test]
fn sprite_render_basic_visual_test() {
    // Cria PPU
    let mut ppu = PPU::new();

    // Habilita LCD e sprites (LCDC): bit7 = LCD on, bit1 = sprites on, bit4 tiles unsigned
    ppu.lcdc = 0x80 | 0x02 | 0x10; // LCD on, sprites on, tile data = 0x8000 mode

    // Paletas (OBP0): vamos usar valores não triviais para visualizar
    ppu.obp0 = 0b11_10_01_00; // map 0->0,1->1,2->2,3->3 (identidade mas explícita)

    // Limpa framebuffer
    for v in ppu.framebuffer.iter_mut() { *v = 0; }

    // Escreve um tile simples em VRAM (tile index 5, por exemplo)
    // Tile pattern: para facilitar ver recorte vertical, vamos desenhar uma coluna vertical à esquerda.
    // Cada linha do tile (8 linhas) = 2 bytes.
    // Vamos montar um tile onde os 4 pixels à esquerda são cor 3 (bits = 11), os outros 4 zeros.
    let tile_index: usize = 5;
    let tile_base = tile_index * 16;

    for row in 0..8 {
        // bits 7..0 -> vamos definir pixels 7..4 = 1, pixels 3..0 = 0
        // Para color 3 (binary 11) por pixel, byte1 (lsb) tem 1s e byte2 (msb) tem 1s nas mesmas posições.
        // Exemplo: left 4 pixels set -> bits 7..4 = 1111 -> lsb row = 0b1111_0000 = 0xF0
        let byte1 = 0xF0u8; // lsb bits
        let byte2 = 0xF0u8; // msb bits -> color = 0b11 on those pixels
        ppu.vram[tile_base + row * 2] = byte1;
        ppu.vram[tile_base + row * 2 + 1] = byte2;
    }

    // Coloca um sprite usando esse tile.
    // OAM: cada sprite = 4 bytes: Y, X, tile_index, attributes
    // Lembre: OAM Y = true_y + 16, X = true_x + 8.
    let sprite_y_true: i16 = 72; // linha vertical central da tela
    let sprite_x_true: i16 = 72;
    let oam_index = 0; // primeiro sprite

    ppu.oam[oam_index * 4 + 0] = (sprite_y_true + 16) as u8;
    ppu.oam[oam_index * 4 + 1] = (sprite_x_true + 8) as u8;
    ppu.oam[oam_index * 4 + 2] = tile_index as u8;
    ppu.oam[oam_index * 4 + 3] = 0x00; // sem flips, paleta 0, prioridade baixa

    // Desabilita BG para isolar sprite (opcional)
    ppu.lcdc &= !0x01; // clear bit0 (BG) se quiser testar sprite sozinho

    // Avança PPU até um frame terminar (loop até frame_ready)
    let mut iter = 0usize;
    while !ppu.frame_ready && iter < 1_000_000 {
        // passo de 4 cycles por vez (M-cycles) — ajuste se seu step usa outra granularidade
        ppu.step(4, &mut 0u8);
        iter += 1;
    }

    assert!(ppu.frame_ready, "PPU não sinalizou frame_ready dentro do limite de iterações");

    // Escreve PPM para /tmp
    let out = Path::new("/tmp/ppu_sprite_test.ppm");
    write_ppm(out, &ppu.framebuffer).expect("failed to write ppm");

    // Verificações básicas (asserts)
    // Esperamos encontrar cor != 0 em várias linhas na coluna do sprite (recorte vertical detectável)
    // Convertemos coordenadas do sprite para índices de framebuffer:
    let sx = sprite_x_true as usize;
    let sy = sprite_y_true as usize;
    let mut nonzero_count = 0usize;

    for y in 0..8 {
        let px = sx;
        let py = sy + y;
        if py >= 144 || px >= 160 { continue; }
        let val = ppu.framebuffer[py * 160 + px];
        if val != 0 { nonzero_count += 1; }
    }

    // Esperamos ao menos 6/8 linhas visíveis (se houver recorte vertical significativo isso falha)
    assert!(nonzero_count >= 6, "sprite vertical recortado: apenas {} linhas com pixels não-zero (esperado >=6). Ver /tmp/ppu_sprite_test.ppm", nonzero_count);
}
