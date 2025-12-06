// Teste diagnóstico simples para entender problemas de renderização
use gb_emu::GB::PPU::PPU;

#[test]
fn diagnostic_single_sprite_center() {
    // Teste mais simples possível: um sprite no centro da tela
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

    // Sprite no centro
    ppu.write_oam(0xFE00, 72 + 16); // Y = 72
    ppu.write_oam(0xFE01, 76 + 8);  // X = 76
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    // Verificar sprite
    let line = 72;
    let mut pixels = 0;
    for x in 76..84 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            pixels += 1;
        }
    }

    println!("Diagnóstico sprite centro: {} pixels na linha {}", pixels, line);
    assert!(pixels >= 6, "Sprite no centro não renderizado: {} pixels", pixels);
}

#[test]
fn diagnostic_multiple_sprites_same_line() {
    // Teste com dois sprites na mesma linha
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

    // Sprite 1
    ppu.write_oam(0xFE00, 20 + 16);
    ppu.write_oam(0xFE01, 20 + 8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    // Sprite 2
    ppu.write_oam(0xFE04, 20 + 16);
    ppu.write_oam(0xFE05, 40 + 8);
    ppu.write_oam(0xFE06, 0);
    ppu.write_oam(0xFE07, 0x00);

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    // Verificar ambos
    let line = 20;
    let mut sprite1_pixels = 0;
    let mut sprite2_pixels = 0;

    for x in 20..28 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            sprite1_pixels += 1;
        }
    }

    for x in 40..48 {
        if ppu.framebuffer[line * 160 + x] != 0 {
            sprite2_pixels += 1;
        }
    }

    println!("Diagnóstico múltiplos sprites: sprite1={} pixels, sprite2={} pixels", sprite1_pixels, sprite2_pixels);

    // Pelo menos um deve renderizar
    assert!(sprite1_pixels >= 6 || sprite2_pixels >= 6,
            "Nenhum sprite renderizado: sprite1={}, sprite2={}", sprite1_pixels, sprite2_pixels);
}
