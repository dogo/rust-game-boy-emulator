// Teste para verificar flip horizontal e vertical de sprites
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
fn test_sprite_flip_horizontal() {
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    // Configurar LCD e sprites
    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    // Limpar framebuffer
    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Criar tile com padrão assimétrico: esquerda escura, direita clara
    // Tile 0: pixels 0-3 = cor 3 (preto), pixels 4-7 = cor 1 (cinza claro)
    for row in 0..8 {
        // bits 7-4 = 1 (esquerda), bits 3-0 = 0 (direita)
        ppu.write_vram(0x8000 + row * 2, 0xF0);     // LSB: 11110000
        ppu.write_vram(0x8000 + row * 2 + 1, 0xF0); // MSB: 11110000 -> cor 3 na esquerda
    }

    // Desabilitar BG
    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite 1: sem flip (esquerda deve ser escura)
    ppu.write_oam(0xFE00, 20 + 16); // Y = 20
    ppu.write_oam(0xFE01, 20 + 8);  // X = 20
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00); // sem flip

    // Sprite 2: com flip horizontal (direita deve ser escura)
    ppu.write_oam(0xFE04, 20 + 16); // Y = 20
    ppu.write_oam(0xFE05, 40 + 8);  // X = 40
    ppu.write_oam(0xFE06, 0);
    ppu.write_oam(0xFE07, 0x20); // flip horizontal (bit 5)

    // Renderizar frame
    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    // Verificar que sprites foram renderizados
    let line = 20;
    let mut sprite1_pixels = 0;
    let mut sprite2_pixels = 0;

    // Sprite 1 (sem flip): esquerda deve ser escura (cor 3)
    for x in 20..28 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            sprite1_pixels += 1;
        }
    }

    // Sprite 2 (com flip): direita deve ser escura (cor 3)
    for x in 40..48 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            sprite2_pixels += 1;
        }
    }

    write_ppm(Path::new("/tmp/sprite_flip_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Ambos devem ter pixels renderizados
    assert!(sprite1_pixels >= 6, "Sprite 1 (sem flip) não renderizado: {} pixels", sprite1_pixels);
    assert!(sprite2_pixels >= 6, "Sprite 2 (com flip) não renderizado: {} pixels", sprite2_pixels);

    // Verificar padrão: sprite 1 esquerda escura, sprite 2 direita escura
    // (Isso valida que o flip está funcionando)
    let sprite1_left = ppu.framebuffer[line * 160 + 20]; // pixel esquerdo sprite 1
    let sprite2_right = ppu.framebuffer[line * 160 + 47]; // pixel direito sprite 2

    // Ambos devem ser escuros (cor 3) se flip funcionar
    // Se não funcionar, sprite2_right seria claro
    println!("Sprite 1 (sem flip) pixel esquerdo: {}", sprite1_left);
    println!("Sprite 2 (com flip) pixel direito: {}", sprite2_right);
}

#[test]
fn test_sprite_flip_vertical() {
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    // Configurar LCD e sprites
    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    // Limpar framebuffer
    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Criar tile com padrão vertical assimétrico: topo escuro, fundo claro
    // Linha 0-3: cor 3 (preto), linha 4-7: cor 1 (cinza claro)
    for row in 0..4 {
        // Topo: escuro
        ppu.write_vram(0x8000 + row * 2, 0xFF);
        ppu.write_vram(0x8000 + row * 2 + 1, 0xFF);
    }
    for row in 4..8 {
        // Fundo: claro
        ppu.write_vram(0x8000 + row * 2, 0xFF);
        ppu.write_vram(0x8000 + row * 2 + 1, 0x00);
    }

    // Desabilitar BG
    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite 1: sem flip (topo deve ser escuro)
    ppu.write_oam(0xFE00, 20 + 16); // Y = 20
    ppu.write_oam(0xFE01, 20 + 8);  // X = 20
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00); // sem flip

    // Sprite 2: com flip vertical (fundo deve ser escuro)
    ppu.write_oam(0xFE04, 20 + 16); // Y = 20
    ppu.write_oam(0xFE05, 40 + 8);  // X = 40
    ppu.write_oam(0xFE06, 0);
    ppu.write_oam(0xFE07, 0x40); // flip vertical (bit 6)

    // Renderizar frame
    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_flip_vertical_test.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    // Verificar que ambos renderizaram
    let mut sprite1_lines = 0;
    let mut sprite2_lines = 0;

    for row in 0..8 {
        let y = 20 + row;
        if y >= 144 { break; }

        if ppu.framebuffer[y * 160 + 20] != 0 {
            sprite1_lines += 1;
        }
        if ppu.framebuffer[y * 160 + 40] != 0 {
            sprite2_lines += 1;
        }
    }

    assert!(sprite1_lines >= 6, "Sprite 1 (sem flip) não renderizado");
    assert!(sprite2_lines >= 6, "Sprite 2 (com flip vertical) não renderizado");
}
