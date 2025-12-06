// Teste específico para sprites com X < 80 (caso que estava falhando)
use gb_emu::GB::PPU::PPU;
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
                0 => (255, 255, 255),
                1 => (192, 192, 192),
                2 => (96, 96, 96),
                3 => (0, 0, 0),
                _ => (255, 0, 255),
            };
            writeln!(f, "{} {} {}", r, g, b)?;
        }
    }
    Ok(())
}

#[test]
fn test_sprite_x_40() {
    // Teste específico para sprite em X=40 (true_x), que estava falhando antes
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Tile simples
    for row in 0..8 {
        ppu.write_vram(0x8000 + row * 2, 0xFF);
        ppu.write_vram(0x8000 + row * 2 + 1, 0x00);
    }

    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite em X=40 (true_x), OAM X = 40 + 8 = 48
    let sx: i16 = 40;
    let sy: i16 = 72;
    ppu.write_oam(0xFE00, (sy + 16) as u8);
    ppu.write_oam(0xFE01, (sx + 8) as u8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_x40_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que sprite foi renderizado
    let mut visible_lines = 0;
    for row in 0..8 {
        let py = (sy as usize).saturating_add(row);
        if py >= 144 { continue; }
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

    assert!(visible_lines >= 6, "Sprite em X=40 não renderizado: {} linhas visíveis (<6). Veja /tmp/sprite_x40_test.ppm", visible_lines);
}

#[test]
fn test_sprite_x_20() {
    // Teste para sprite em X=20 (ainda menor)
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Tile simples
    for row in 0..8 {
        ppu.write_vram(0x8000 + row * 2, 0xFF);
        ppu.write_vram(0x8000 + row * 2 + 1, 0x00);
    }

    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite em X=20 (true_x), OAM X = 20 + 8 = 28
    let sx: i16 = 20;
    let sy: i16 = 72;
    ppu.write_oam(0xFE00, (sy + 16) as u8);
    ppu.write_oam(0xFE01, (sx + 8) as u8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    // Verificar que sprite foi renderizado
    let mut visible_lines = 0;
    for row in 0..8 {
        let py = (sy as usize).saturating_add(row);
        if py >= 144 { continue; }
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

    assert!(visible_lines >= 6, "Sprite em X=20 não renderizado: {} linhas visíveis (<6)", visible_lines);
}

#[test]
fn test_sprite_x_8() {
    // Teste para sprite em X=8 (edge case)
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Tile simples
    for row in 0..8 {
        ppu.write_vram(0x8000 + row * 2, 0xFF);
        ppu.write_vram(0x8000 + row * 2 + 1, 0x00);
    }

    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite em X=8 (true_x), OAM X = 8 + 8 = 16
    let sx: i16 = 8;
    let sy: i16 = 72;
    ppu.write_oam(0xFE00, (sy + 16) as u8);
    ppu.write_oam(0xFE01, (sx + 8) as u8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    // Verificar que sprite foi renderizado
    let mut visible_lines = 0;
    for row in 0..8 {
        let py = (sy as usize).saturating_add(row);
        if py >= 144 { continue; }
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

    assert!(visible_lines >= 6, "Sprite em X=8 não renderizado: {} linhas visíveis (<6)", visible_lines);
}
