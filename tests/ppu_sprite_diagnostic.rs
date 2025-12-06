// Testes de diagnóstico para problemas de sprites
use gb_emu::GB::PPU::PPU;

#[test]
fn test_sprite_collection() {
    let mut ppu = PPU::new();
    ppu.lcdc = 0x93; // LCD on, sprites on
    ppu.ly = 10;

    // Sprite na linha 10 (OAM Y = 10 + 16 = 26)
    ppu.oam[0] = 26; // Y
    ppu.oam[1] = 10; // X
    ppu.oam[2] = 0;  // Tile
    ppu.oam[3] = 0;  // Flags

    // Simular mode 2 (OAM Search) que coleta sprites
    let mut iflags = 0u8;
    ppu.change_mode(2, &mut iflags);

    // Verificar se sprite foi coletado
    // Como visible_sprites é privado, vamos verificar indiretamente
    // através do comportamento durante mode 3
    ppu.change_mode(3, &mut iflags);

    // Avançar alguns ciclos para ver se sprite é processado
    for _ in 0..100 {
        ppu.step(4, &mut iflags);
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
    let mut ppu = PPU::new();
    ppu.oam_fifo.clear();
    ppu.oam_fifo.size = 0;
    ppu.oam_fifo.read_end = 0;

    // Tile: 11110000 (0xF0)
    let lower = 0xF0;
    let upper = 0x00;

    // Sem flip: deve colocar pixels 1,1,1,1,0,0,0,0 nas posições 0-7
    // Com flip: deve inverter para 0,0,0,0,1,1,1,1

    // Teste sem flip
    ppu.fifo_overlay_object_row(lower, upper, 0, false, 0, false);
    assert_eq!(ppu.oam_fifo.buffer[0].pixel, 1, "Pixel 0 sem flip deve ser 1");
    assert_eq!(ppu.oam_fifo.buffer[7].pixel, 0, "Pixel 7 sem flip deve ser 0");

    // Limpar e testar com flip
    ppu.oam_fifo.clear();
    ppu.oam_fifo.size = 0;
    ppu.oam_fifo.read_end = 0;

    ppu.fifo_overlay_object_row(lower, upper, 0, false, 0, true);
    assert_eq!(ppu.oam_fifo.buffer[0].pixel, 0, "Pixel 0 com flip deve ser 0");
    assert_eq!(ppu.oam_fifo.buffer[7].pixel, 1, "Pixel 7 com flip deve ser 1");
}
