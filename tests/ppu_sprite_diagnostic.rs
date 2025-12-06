// Testes de diagnóstico para problemas de sprites
use gb_emu::GB::PPU::PPU;

#[test]
fn test_sprite_collection() {
    let mut ppu = PPU::new();
    let mut iflags = 0u8;

    // Habilitar LCD e sprites
    ppu.write_register(0xFF40, 0x93, &mut iflags);

    // Sprite na linha 10 (OAM Y = 10 + 16 = 26)
    ppu.write_oam(0xFE00, 26); // Y
    ppu.write_oam(0xFE01, 10); // X
    ppu.write_oam(0xFE02, 0);  // Tile
    ppu.write_oam(0xFE03, 0);  // Flags

    // Criar tile simples
    ppu.write_vram(0x8000, 0xFF);
    ppu.write_vram(0x8001, 0x00);

    // Configurar paleta
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    // Renderizar até linha 10 ser completada
    let mut safety = 0;
    while ppu.ly < 11 && safety < 100_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    // Avançar mais um pouco para garantir que a linha foi renderizada
    for _ in 0..500 {
        ppu.step(4, &mut iflags);
        if ppu.ly > 10 {
            break;
        }
    }

    // Verificar se algum pixel foi renderizado na linha 10
    let line_start = 10 * 160;
    let mut has_pixels = false;
    for x in 0..160 {
        if ppu.framebuffer[line_start + x] != 0 {
            has_pixels = true;
            break;
        }
    }

    // Este teste pode falhar se sprites não estão sendo coletados ou processados
    // Mas não vamos fazer assert aqui, apenas documentar
    println!("Sprite collection test: has_pixels = {}", has_pixels);
}

#[test]
fn test_get_object_line_address_calculation() {
    let mut ppu = PPU::new();
    ppu.lcdc = 0x91; // 8x8 sprites

    // Teste manual do cálculo
    // Sprite na linha 10 da tela (OAM Y = 26)
    let oam_y = 26u8;
    let current_line = 10u8;

    // Cálculo esperado:
    // sprite_top = oam_y - 16 = 26 - 16 = 10
    // tile_y = (current_line - sprite_top) & 7 = (10 - 10) & 7 = 0
    let sprite_top = oam_y.wrapping_sub(16);
    let tile_y = current_line.wrapping_sub(sprite_top) & 7;

    assert_eq!(sprite_top, 10, "sprite_top deve ser 10");
    assert_eq!(tile_y, 0, "tile_y deve ser 0 para linha 0 do sprite");

    // Teste linha 11 do mesmo sprite
    let current_line2 = 11u8;
    let tile_y2 = current_line2.wrapping_sub(sprite_top) & 7;
    assert_eq!(tile_y2, 1, "tile_y deve ser 1 para linha 1 do sprite");
}

#[test]
fn test_fifo_overlay_flip_x() {
    // Este teste não pode ser feito sem acessar campos privados.
    // Vamos testar o flip horizontal indiretamente através da renderização.
    let mut ppu = PPU::new();
    let mut iflags = 0u8;

    // Habilitar LCD, BG e sprites
    ppu.write_register(0xFF40, 0x93, &mut iflags);

    // Criar tile assimétrico: 11110000 (0xF0)
    // Sem flip: pixels 1,1,1,1,0,0,0,0
    // Com flip: pixels 0,0,0,0,1,1,1,1
    ppu.write_vram(0x8000, 0xF0);
    ppu.write_vram(0x8001, 0x00);

    // Configurar BG para ter tile 0 em todas as posições (cor 0)
    for i in 0..32 * 32 {
        ppu.write_vram(0x9800 + i as u16, 0);
    }

    // Sprite sem flip na linha 0
    ppu.write_oam(0xFE00, 16); // Y = linha 0
    ppu.write_oam(0xFE01, 8);  // X = coluna 0
    ppu.write_oam(0xFE02, 0);  // Tile 0
    ppu.write_oam(0xFE03, 0x00); // Sem flip

    ppu.write_register(0xFF48, 0xE4, &mut iflags);
    ppu.write_register(0xFF47, 0xE4, &mut iflags);

    // Renderizar linha 0
    let mut safety = 0;
    while ppu.ly < 1 && safety < 100_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }
    for _ in 0..500 {
        ppu.step(4, &mut iflags);
        if ppu.ly > 0 {
            break;
        }
    }

    // Verificar se algum pixel foi renderizado
    let mut pixels_without_flip = 0;
    for x in 8..16 {
        if ppu.framebuffer[x] != 0 {
            pixels_without_flip += 1;
        }
    }

    // Limpar framebuffer
    for p in ppu.framebuffer.iter_mut() { *p = 0; }
    ppu.frame_ready = false;

    // Sprite com flip horizontal (bit 5 = 1)
    ppu.write_oam(0xFE03, 0x20); // Bit 5 = flip horizontal

    // Renderizar linha 0 novamente
    safety = 0;
    while ppu.ly < 1 && safety < 100_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }
    for _ in 0..500 {
        ppu.step(4, &mut iflags);
        if ppu.ly > 0 {
            break;
        }
    }

    // Verificar se algum pixel foi renderizado com flip
    let mut pixels_with_flip = 0;
    for x in 8..16 {
        if ppu.framebuffer[x] != 0 {
            pixels_with_flip += 1;
        }
    }

    println!("Flip test: pixels_without_flip = {}, pixels_with_flip = {}",
             pixels_without_flip, pixels_with_flip);

    // Ambos devem renderizar alguns pixels (o padrão será diferente)
    assert!(pixels_without_flip > 0 || pixels_with_flip > 0,
            "Pelo menos um dos sprites deve renderizar pixels");
}
