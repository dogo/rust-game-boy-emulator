// tests/ppu_sprite_integration.rs

// Integração: desenha 1 sprite à mão via VRAM/OAM/REG e verifica recorte.

// Usa apenas métodos públicos: PPU::new(), write_vram, write_oam, write_register, step,

// e os campos públicos framebuffer / frame_ready.

// Ajuste o path abaixo conforme o layout do seu crate.

// 1) se o tipo PPU está em crate::ppu::PPU

// use crate::ppu::PPU;

// 2) se está em crate::GB::PPU::PPU

use gb_emu::GB::PPU::PPU;

// 3) se está no root como PPU (ex.: mod ppu; pub use ppu::PPU;)

// use crate::ppu::PPU;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn write_ppm(path: &Path, framebuffer: &[u8]) -> std::io::Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    writeln!(f, "P3")?;
    writeln!(f, "160 144")?;
    writeln!(f, "255")?;
    for y in 0..144 {
        for x in 0..160 {
            let v = framebuffer[y * 160 + x];
            let (r, g, b) = match v {
                0 => (255u8, 255u8, 255u8),
                1 => (192u8, 192u8, 192u8),
                2 => (96u8, 96u8, 96u8),
                3 => (0u8, 0u8, 0u8),
                _ => (255u8, 0u8, 255u8),
            };
            writeln!(f, "{} {} {}", r, g, b)?;
        }
    }
    Ok(())
}

#[test]
fn sprite_integration_no_private_api() {
    // Cria PPU via API pública
    let mut ppu = PPU::new();

    // iflags usado nas chamadas públicas que pedem &mut u8
    let mut iflags: u8 = 0;

    // --- Configure LCD / paletas / estado ---
    // Enable LCD, sprites on, use tile data mode 0x8000 (bit4 = 1)
    // LCDC: bit7 = LCD enable, bit1 = OBJ enable, bit4 = BG & Window tile data area (1 -> 0x8000)
    let lcdc_val = 0x80 | 0x02 | 0x10;
    ppu.write_register(0xFF40, lcdc_val, &mut iflags);

    // Set OBP0 to identity mapping for clarity (public reg)
    ppu.write_register(0xFF48, 0xE4, &mut iflags); // qualquer valor razoável; adjust if needed

    // Clear framebuffer
    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // --- Write a test tile into VRAM via public write_vram ---
    // Usamos tile index 0 (simples). VRAM base 0x8000.
    let tile_index = 0usize;
    let tile_addr = 0x8000u16.wrapping_add((tile_index * 16) as u16);

    // Tile pattern: todos pixels cor 1 (para facilitar visualização)
    for row in 0..8 {
        // byte low/high per linha: todos pixels cor 1
        let b1 = 0xFFu8; // lsb: todos 1
        let b2 = 0x00u8; // msb: todos 0 -> color = 0b01 = 1
        ppu.write_vram(tile_addr + (row * 2) as u16, b1);
        ppu.write_vram(tile_addr + (row * 2 + 1) as u16, b2);
    }

    // --- Place a single sprite in OAM via public write_oam ---
    // Sprite true coords (screen): (sx, sy)
    let sx: i16 = 72;
    let sy: i16 = 72;

    // OAM address area starts at 0xFE00
    let oam_base = 0xFE00u16;

    // Each sprite is 4 bytes: Y, X, TILE, ATTR
    let spr_idx = 0usize;
    ppu.write_oam(oam_base + (spr_idx * 4) as u16 + 0, (sy + 16) as u8); // OAM Y = true_y + 16
    ppu.write_oam(oam_base + (spr_idx * 4) as u16 + 1, (sx + 8) as u8);  // OAM X = true_x + 8
    ppu.write_oam(oam_base + (spr_idx * 4) as u16 + 2, tile_index as u8);
    ppu.write_oam(oam_base + (spr_idx * 4) as u16 + 3, 0x00); // attributes: no flip, palette 0

    // Optional: disable BG to isolate sprite draw (write_register for LCDC bit0)
    let mut lcdc_read = ppu.read_register(0xFF40);
    lcdc_read &= !0x01; // clear BG enable if desired
    ppu.write_register(0xFF40, lcdc_read, &mut iflags);

    // --- Step PPU until a frame is rendered (frame_ready) ---
    let mut safety = 0usize;
    while !ppu.frame_ready && safety < 500_000 {
        ppu.step(4, &mut iflags); // 4 cycles per step is fine; se necessário use 1
        safety += 1;
    }

    assert!(ppu.frame_ready, "PPU não entrou em VBlank dentro do limite; verifique step granularity");

    // Dump PPM para inspeção manual
    let out = Path::new("/tmp/ppu_sprite_integration.ppm");
    write_ppm(out, &ppu.framebuffer).expect("Escrita PPM falhou");

    // --- Asserts básicos (somente usando framebuffer público) ---
    // Conte quantas linhas do sprite têm pixel não-zero na coluna sx..sx+7
    let mut visible_lines = 0usize;
    for row in 0..8 {
        let py = (sy as usize).saturating_add(row);
        if py >= 144 { continue; }
        // Verificar qualquer pixel na linha do sprite (não apenas a coluna esquerda)
        let mut has_pixel = false;
        for px in (sx as usize)..((sx + 8) as usize).min(160) {
            if ppu.framebuffer[py * 160 + px] != 0 {
                has_pixel = true;
                break;
            }
        }
        if has_pixel {
            visible_lines += 1;
        }
    }

    // Expectativa: a maioria das linhas do sprite deve estar visível (evita falso positivo em offscreen)
    assert!(visible_lines >= 6, "Sprite vertical recortado: linhas visíveis = {} (<6). Veja /tmp/ppu_sprite_integration.ppm", visible_lines);
}
