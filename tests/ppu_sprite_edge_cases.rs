// Teste para casos extremos: sprites na borda da tela, sprites parcialmente fora, etc.
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
fn test_sprite_at_left_edge() {
    // Sprite começando em X=0 (ou próximo)
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

    // Sprite em X=0 (OAM X = 0 + 8 = 8)
    ppu.write_oam(0xFE00, 20 + 16);
    ppu.write_oam(0xFE01, 0 + 8); // X = 0 na tela
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_left_edge_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que sprite foi renderizado na borda esquerda
    let line = 20;
    let mut pixels = 0;
    for x in 0..8 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            pixels += 1;
        }
    }

    assert!(pixels >= 6, "Sprite na borda esquerda não renderizado: {} pixels", pixels);
}

#[test]
fn test_sprite_at_right_edge() {
    // Sprite terminando em X=159 (ou próximo)
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

    // Sprite em X=152 (OAM X = 152 + 8 = 160, mas sprite vai até X=159)
    ppu.write_oam(0xFE00, 20 + 16);
    ppu.write_oam(0xFE01, 152 + 8); // X = 152 na tela
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_right_edge_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que sprite foi renderizado na borda direita
    let line = 20;
    let mut pixels = 0;
    for x in 152..160 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            pixels += 1;
        }
    }

    assert!(pixels >= 4, "Sprite na borda direita não renderizado: {} pixels", pixels);
}

#[test]
fn test_sprite_partially_offscreen_right() {
    // Sprite parcialmente fora da tela à direita (X > 160)
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

    // Sprite em X=156 (parte do sprite fica fora da tela)
    ppu.write_oam(0xFE00, 20 + 16);
    ppu.write_oam(0xFE01, 156 + 8); // X = 156 na tela (sprite vai até 163, mas tela só até 159)
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_offscreen_right_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que parte visível do sprite foi renderizada
    let line = 20;
    let mut pixels = 0;
    for x in 156..160 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            pixels += 1;
        }
    }

    // Esperamos pelo menos 1 pixel (parte visível de 8 pixels, mas pode ser menos devido ao clipping)
    // O sprite em X=156 vai até X=163, mas apenas pixels 156-159 são visíveis
    assert!(pixels >= 1, "Sprite parcialmente fora não renderizado: {} pixels", pixels);
}

#[test]
fn test_sprite_at_top_edge() {
    // Sprite começando em Y=0
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

    // Sprite em Y=0 (OAM Y = 0 + 16 = 16)
    ppu.write_oam(0xFE00, 0 + 16); // Y = 0 na tela
    ppu.write_oam(0xFE01, 20 + 8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_top_edge_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que sprite foi renderizado no topo
    let mut pixels = 0;
    for y in 0..8 {
        if ppu.framebuffer[y * 160 + 20] != 0 {
            pixels += 1;
        }
    }

    assert!(pixels >= 6, "Sprite no topo não renderizado: {} pixels", pixels);
}

#[test]
fn test_sprite_at_bottom_edge() {
    // Sprite terminando em Y=143
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

    // Sprite em Y=136 (sprite vai até Y=143)
    ppu.write_oam(0xFE00, 136 + 16); // Y = 136 na tela
    ppu.write_oam(0xFE01, 20 + 8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_bottom_edge_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que sprite foi renderizado no fundo
    let mut pixels = 0;
    for y in 136..144 {
        if ppu.framebuffer[y * 160 + 20] != 0 {
            pixels += 1;
        }
    }

    assert!(pixels >= 6, "Sprite no fundo não renderizado: {} pixels", pixels);
}
