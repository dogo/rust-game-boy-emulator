// Integration tests para PPU
// cargo test ppu_test

#[cfg(test)]
mod ppu_tests {
    use gb_emu::GB::PPU::PPU;

    #[test]
    fn test_decode_tile_simple() {
        let mut ppu = PPU::new();

        // Criar um tile simples: checkerboard pattern
        // Linha 0: 10101010 (0xAA em bits)
        ppu.vram[0] = 0xAA; // LSB: 10101010
        ppu.vram[1] = 0x00; // MSB: 00000000
        // Resultado esperado: cores 1,0,1,0,1,0,1,0

        // Linha 1: 01010101 (0x55 em bits)
        ppu.vram[2] = 0x55; // LSB: 01010101
        ppu.vram[3] = 0x00; // MSB: 00000000
        // Resultado esperado: cores 0,1,0,1,0,1,0,1

        let pixels = ppu.decode_tile(0);

        // Verificar linha 0
        assert_eq!(pixels[0], 1);
        assert_eq!(pixels[1], 0);
        assert_eq!(pixels[2], 1);
        assert_eq!(pixels[3], 0);

        // Verificar linha 1
        assert_eq!(pixels[8], 0);
        assert_eq!(pixels[9], 1);
        assert_eq!(pixels[10], 0);
        assert_eq!(pixels[11], 1);
    }

    #[test]
    fn test_decode_tile_all_colors() {
        let mut ppu = PPU::new();

        // Criar tile com todas as 4 cores
        // Cor 0 (00): LSB=0, MSB=0
        // Cor 1 (01): LSB=1, MSB=0
        // Cor 2 (10): LSB=0, MSB=1
        // Cor 3 (11): LSB=1, MSB=1

        // Linha 0: cores 3,2,1,0, 3,2,1,0
        ppu.vram[0] = 0b10101010; // LSB
        ppu.vram[1] = 0b11001100; // MSB
        // Pixels: cor=(MSB<<1)|LSB para cada bit
        // bit 7: MSB=1, LSB=1 -> cor 3
        // bit 6: MSB=1, LSB=0 -> cor 2
        // bit 5: MSB=0, LSB=1 -> cor 1
        // bit 4: MSB=0, LSB=0 -> cor 0

        let pixels = ppu.decode_tile(0);
        assert_eq!(pixels[0], 3); // bit 7
        assert_eq!(pixels[1], 2); // bit 6
        assert_eq!(pixels[2], 1); // bit 5
        assert_eq!(pixels[3], 0); // bit 4
        assert_eq!(pixels[4], 3); // bit 3
        assert_eq!(pixels[5], 2); // bit 2
        assert_eq!(pixels[6], 1); // bit 1
        assert_eq!(pixels[7], 0); // bit 0
    }

    #[test]
    fn test_apply_palette() {
        let mut ppu = PPU::new();

        // BGP padrão = 0xFC = 11111100 em binário
        // bits 7-6 (cor 3): 11 = 3
        // bits 5-4 (cor 2): 11 = 3
        // bits 3-2 (cor 1): 11 = 3
        // bits 1-0 (cor 0): 00 = 0

        assert_eq!(ppu.apply_palette(0), 0); // cor 0 -> 0 (branco)
        assert_eq!(ppu.apply_palette(1), 3); // cor 1 -> 3 (preto)
        assert_eq!(ppu.apply_palette(2), 3); // cor 2 -> 3 (preto)
        assert_eq!(ppu.apply_palette(3), 3); // cor 3 -> 3 (preto)

        // Testar paleta customizada: 0xE4 = 11100100
        // bits 7-6 (cor 3): 11 = 3 (preto)
        // bits 5-4 (cor 2): 10 = 2 (cinza escuro)
        // bits 3-2 (cor 1): 01 = 1 (cinza claro)
        // bits 1-0 (cor 0): 00 = 0 (branco)
        ppu.bgp = 0xE4;

        assert_eq!(ppu.apply_palette(0), 0); // cor 0 -> 0 (branco)
        assert_eq!(ppu.apply_palette(1), 1); // cor 1 -> 1 (cinza claro)
        assert_eq!(ppu.apply_palette(2), 2); // cor 2 -> 2 (cinza escuro)
        assert_eq!(ppu.apply_palette(3), 3); // cor 3 -> 3 (preto)
    }

    #[test]
    fn test_bg_scanline_simple() {
        let mut ppu = PPU::new();

        // Configurar tile 0 com padrão sólido (cor 3)
        for i in 0..8 {
            ppu.vram[i * 2] = 0xFF; // LSB todos 1
            ppu.vram[i * 2 + 1] = 0xFF; // MSB todos 1
        }

        // Configurar tile map para usar tile 0 em todas as posições
        for i in 0..32 * 32 {
            ppu.vram[0x1800 + i] = 0; // Tile 0
        }

        // LCDC: BG enabled, tile map 0x9800, tile data 0x8000
        ppu.lcdc = 0x91; // bit 0=1 (BG on), bit 4=1 (tile data 0x8000)

        // Paleta: 0xE4 (3,2,1,0)
        ppu.bgp = 0xE4;

        // Renderizar linha 0
        ppu.ly = 0;
        ppu.render_bg_scanline();

        // Todos os pixels devem ser cor 3 (que se torna 3 pela paleta 0xE4)
        for x in 0..160 {
            assert_eq!(ppu.framebuffer[x], 3, "Pixel {} na linha 0 deve ser 3", x);
        }
    }

    #[test]
    fn test_bg_with_scroll() {
        let mut ppu = PPU::new();

        // Criar dois tiles distintos
        // Tile 0: todos pixels cor 0
        for i in 0..8 {
            ppu.vram[i * 2] = 0x00;
            ppu.vram[i * 2 + 1] = 0x00;
        }

        // Tile 1: todos pixels cor 3
        for i in 0..8 {
            ppu.vram[16 + i * 2] = 0xFF;
            ppu.vram[16 + i * 2 + 1] = 0xFF;
        }

        // Tile map: primeira coluna usa tile 0, segunda coluna usa tile 1
        for y in 0..32 {
            ppu.vram[0x1800 + y * 32] = 0; // Coluna 0: tile 0
            ppu.vram[0x1800 + y * 32 + 1] = 1; // Coluna 1: tile 1
        }

        ppu.lcdc = 0x91;
        ppu.bgp = 0xE4;

        // Sem scroll: primeiros 8 pixels devem ser cor 0, próximos 8 devem ser cor 3
        ppu.scx = 0;
        ppu.scy = 0;
        ppu.ly = 0;
        ppu.render_bg_scanline();

        assert_eq!(ppu.framebuffer[0], 0); // Tile 0
        assert_eq!(ppu.framebuffer[7], 0); // Tile 0
        assert_eq!(ppu.framebuffer[8], 3); // Tile 1
        assert_eq!(ppu.framebuffer[15], 3); // Tile 1

        // Com scroll X=4: deve deslocar 4 pixels para esquerda
        ppu.scx = 4;
        ppu.ly = 0;
        ppu.render_bg_scanline();

        // Primeiros 4 pixels são os últimos 4 de tile 0
        assert_eq!(ppu.framebuffer[0], 0); // Ainda tile 0
        assert_eq!(ppu.framebuffer[3], 0); // Último pixel de tile 0
        assert_eq!(ppu.framebuffer[4], 3); // Primeiro pixel de tile 1
    }

    #[test]
    fn test_render_full_frame() {
        let mut ppu = PPU::new();

        // Criar gradient vertical: tile 0=cor0, tile 1=cor1, tile 2=cor2, tile 3=cor3
        for tile in 0..4 {
            for i in 0..8 {
                let lsb = if tile & 1 != 0 { 0xFF } else { 0x00 };
                let msb = if tile & 2 != 0 { 0xFF } else { 0x00 };
                ppu.vram[tile * 16 + i * 2] = lsb;
                ppu.vram[tile * 16 + i * 2 + 1] = msb;
            }
        }

        // Tile map: linha 0 usa tile 0, linha 1 usa tile 1, etc
        for y in 0..32 {
            let tile = (y % 4) as u8;
            for x in 0..32 {
                ppu.vram[0x1800 + y * 32 + x] = tile;
            }
        }

        ppu.lcdc = 0x91;
        ppu.bgp = 0xE4;

        // Renderizar frame completo
        ppu.render_frame();

        // Verificar que linha 0 (8 primeiras linhas de tiles) tem cor 0
        for y in 0..8 {
            for x in 0..160 {
                assert_eq!(ppu.framebuffer[y * 160 + x], 0);
            }
        }

        // Verificar que linhas 8-15 têm cor 1
        for y in 8..16 {
            for x in 0..160 {
                assert_eq!(ppu.framebuffer[y * 160 + x], 1);
            }
        }
    }

    #[test]
    fn test_lcd_stat_interrupt_vblank() {
        let mut ppu = PPU::new();

        // Habilitar VBlank interrupt no STAT (bit 4)
        ppu.stat = 0x10;
        ppu.ly = 144; // VBlank começa na linha 144

        // Atualizar modo para VBlank
        ppu.update_stat_mode(1);

        // Verificar que STAT interrupt deve ser gerado
        assert!(
            ppu.check_stat_interrupt(),
            "STAT interrupt deveria ser gerado no VBlank"
        );

        // Verificar modo PPU
        assert_eq!(ppu.stat & 0x03, 1, "Modo PPU deveria ser 1 (VBlank)");
    }

    #[test]
    fn test_lcd_stat_interrupt_lyc_equals_ly() {
        let mut ppu = PPU::new();

        // Configurar LYC=LY coincidence
        ppu.ly = 100;
        ppu.lyc = 100;

        // Habilitar LYC=LY interrupt no STAT (bit 6)
        ppu.stat = 0x40;

        // Atualizar flag LYC=LY
        ppu.update_lyc_flag();

        // Verificar que flag LYC foi setada (bit 2)
        assert_eq!(ppu.stat & 0x04, 0x04, "Flag LYC=LY deveria estar setada");

        // Verificar que STAT interrupt deve ser gerado
        assert!(
            ppu.check_stat_interrupt(),
            "STAT interrupt deveria ser gerado quando LYC=LY"
        );
    }

    #[test]
    fn test_lcd_stat_interrupt_disabled() {
        let mut ppu = PPU::new();

        // Desabilitar todos os interrupts STAT
        ppu.stat = 0x00;
        ppu.ly = 144; // VBlank

        // Atualizar modo para VBlank
        ppu.update_stat_mode(1);

        // Verificar que STAT interrupt NÃO deve ser gerado
        assert!(
            !ppu.check_stat_interrupt(),
            "STAT interrupt não deveria ser gerado quando desabilitado"
        );
    }

    #[test]
    fn test_sprite_basic_rendering() {
        let mut ppu = PPU::new();

        // Habilitar sprites no LCDC (bit 1)
        ppu.lcdc = 0x93; // LCD on, BG on, Sprites on

        // Criar tile para sprite no VRAM (tile 1)
        // Linha 0: 11110000 (0xF0 em bits)
        ppu.vram[16] = 0xF0; // LSB
        ppu.vram[17] = 0x00; // MSB
        // Linha 1: 00001111 (0x0F em bits)
        ppu.vram[18] = 0x0F; // LSB
        ppu.vram[19] = 0x00; // MSB

        // Configurar sprite 0 no OAM
        ppu.oam[0] = 16 + 2; // Y = linha 2 (16 + 2)
        ppu.oam[1] = 8 + 10; // X = coluna 10 (8 + 10)
        ppu.oam[2] = 1; // Tile index 1
        ppu.oam[3] = 0x00; // Atributos: paleta OBP0, normal

        // Configurar paleta OBP0 (cores diferentes do BG)
        ppu.obp0 = 0xE4; // 11 10 01 00 = cores 3,2,1,0

        // Renderizar linha 2 (onde está o sprite)
        ppu.render_sprites_scanline(2);

        // Verificar pixels do sprite na linha 2 (linha 0 do tile)
        // Pixels 10-13: cor 1 do tile com paleta 0xE4 → cor 1
        for x in 10..14 {
            assert_eq!(
                ppu.framebuffer[2 * 160 + x],
                1,
                "Pixel [{}, 2] deveria ser cor 1",
                x
            );
        }
        // Pixels 14-17 devem ter cor 0 (transparente, não altera framebuffer)
        for x in 14..18 {
            assert_eq!(
                ppu.framebuffer[2 * 160 + x],
                0,
                "Pixel [{}, 2] deveria ser cor 0",
                x
            );
        }
    }

    #[test]
    fn test_sprite_palette_obp1() {
        let mut ppu = PPU::new();

        // Habilitar sprites
        ppu.lcdc = 0x93;

        // Criar tile simples
        ppu.vram[16] = 0xFF; // Todos os pixels cor 1
        ppu.vram[17] = 0x00;

        // Sprite usando paleta OBP1 (bit 4 = 1)
        ppu.oam[0] = 16; // Y = linha 0
        ppu.oam[1] = 8; // X = coluna 0
        ppu.oam[2] = 1; // Tile 1
        ppu.oam[3] = 0x10; // Bit 4 = paleta OBP1

        // Configurar paletas diferentes
        ppu.obp0 = 0xE4; // OBP0: cor 1 → 1
        ppu.obp1 = 0x1B; // OBP1: cor 1 → 2

        ppu.render_sprites_scanline(0);

        // Pixel deve usar OBP1, então cor 1 → 2
        assert_eq!(ppu.framebuffer[0], 2, "Sprite deveria usar paleta OBP1");
    }

    #[test]
    fn test_sprite_flip_horizontal() {
        let mut ppu = PPU::new();
        ppu.lcdc = 0x93;

        // Tile assimétrico: 11110000
        ppu.vram[16] = 0xF0;
        ppu.vram[17] = 0x00;

        // Sprite com flip horizontal (bit 5 = 1)
        ppu.oam[0] = 16; // Y = linha 0
        ppu.oam[1] = 8; // X = coluna 0
        ppu.oam[2] = 1; // Tile 1
        ppu.oam[3] = 0x20; // Bit 5 = flip horizontal

        ppu.obp0 = 0xE4;
        ppu.render_sprites_scanline(0);

        // Com flip, 11110000 vira 00001111
        // Primeiros 4 pixels devem ser cor 0 (transparente)
        for x in 0..4 {
            assert_eq!(
                ppu.framebuffer[x], 0,
                "Pixel {} deveria ser transparente",
                x
            );
        }
        // Últimos 4 pixels: cor 1 do tile com paleta 0xE4 → cor 1
        for x in 4..8 {
            assert_eq!(ppu.framebuffer[x], 1, "Pixel {} deveria ser cor 1", x);
        }
    }

    #[test]
    fn test_sprite_priority() {
        let mut ppu = PPU::new();
        ppu.lcdc = 0x93;

        // Preencher background com cor 2
        for i in 0..160 {
            ppu.framebuffer[i] = 2;
        }

        // Tile do sprite (cor 1)
        ppu.vram[16] = 0xFF;
        ppu.vram[17] = 0x00;

        // Sprite com prioridade baixa (bit 7 = 1)
        ppu.oam[0] = 16; // Y = linha 0
        ppu.oam[1] = 8; // X = coluna 0
        ppu.oam[2] = 1; // Tile 1
        ppu.oam[3] = 0x80; // Bit 7 = prioridade baixa

        ppu.obp0 = 0xE4;
        ppu.render_sprites_scanline(0);

        // Sprite com prioridade baixa não deve sobrescrever BG cor != 0
        assert_eq!(
            ppu.framebuffer[0], 2,
            "Sprite com prioridade baixa não deveria sobrescrever BG"
        );

        // Testar sprite com prioridade alta
        ppu.oam[3] = 0x00; // Bit 7 = 0 = prioridade alta
        ppu.render_sprites_scanline(0);

        // Agora deve sobrescrever
        assert_eq!(
            ppu.framebuffer[0], 1,
            "Sprite com prioridade alta deveria sobrescrever BG"
        );
    }

    #[test]
    fn test_sprite_disabled() {
        let mut ppu = PPU::new();

        // Desabilitar sprites no LCDC (bit 1 = 0)
        ppu.lcdc = 0x91; // LCD on, BG on, Sprites OFF

        // Configurar sprite
        ppu.vram[16] = 0xFF;
        ppu.vram[17] = 0x00;
        ppu.oam[0] = 16;
        ppu.oam[1] = 8;
        ppu.oam[2] = 1;
        ppu.oam[3] = 0x00;

        ppu.render_sprites_scanline(0);

        // Framebuffer deve permanecer 0 (sprites desabilitados)
        assert_eq!(
            ppu.framebuffer[0], 0,
            "Sprites desabilitados não deveriam renderizar"
        );
    }

    #[test]
    fn test_window_basic_rendering() {
        let mut ppu = PPU::new();

        // Habilitar BG e Window no LCDC (bits 0, 4 e 5)
        // Bit 4 = 1 para usar modo unsigned (0x8000-0x8FFF)
        ppu.lcdc = 0xB1 | 0x10; // LCD on, BG on, Window on, unsigned mode
        ppu.ly = 5; // Linha atual
        ppu.wy = 5; // Window começa na linha 5 (window_y = 0)
        ppu.wx = 10; // Window começa na coluna 3 (10-7)

        // Criar tile para window (tile index 1, endereço 0x10 em VRAM)
        ppu.vram[16] = 0xFF; // Linha 0: todos pixels cor 1
        ppu.vram[17] = 0x00;

        // Configurar window tile map (usar 0x9800, bit 6 do LCDC = 0)
        // Window linha 5 (ly=10, wy=5, então window_y=5, tile_y=0)
        ppu.vram[0x1800] = 1; // Tile index 1 na posição (0,0) da window

        // Configurar paleta
        ppu.bgp = 0xE4; // 11 10 01 00

        ppu.render_window_scanline();

        // Verificar pixels da window (começam na coluna 3, tile tem pixels cor 1→paleta→1)
        for x in 3..11 {
            // 8 pixels do tile
            assert_eq!(
                ppu.framebuffer[5 * 160 + x],
                1,
                "Pixel window [{}, 5] deveria ser cor 1",
                x
            );
        }
    }

    #[test]
    fn test_window_disabled() {
        let mut ppu = PPU::new();

        // Desabilitar window no LCDC (bit 5 = 0)
        ppu.lcdc = 0x91; // LCD on, BG on, Window OFF
        ppu.ly = 10;
        ppu.wy = 5;
        ppu.wx = 10;

        // Configurar tile e tile map
        ppu.vram[32] = 0xFF;
        ppu.vram[33] = 0x00;
        ppu.vram[0x1800] = 2;

        ppu.render_window_scanline();

        // Framebuffer deve permanecer 0 (window desabilitada)
        for x in 3..11 {
            assert_eq!(
                ppu.framebuffer[10 * 160 + x],
                0,
                "Window desabilitada não deveria renderizar"
            );
        }
    }

    #[test]
    fn test_window_wy_condition() {
        let mut ppu = PPU::new();

        // Window habilitada mas WY > LY
        ppu.lcdc = 0xB1;
        ppu.ly = 5; // Linha atual
        ppu.wy = 10; // Window só começa na linha 10
        ppu.wx = 10;

        // Configurar tile
        ppu.vram[32] = 0xFF;
        ppu.vram[33] = 0x00;
        ppu.vram[0x1800] = 2;

        ppu.render_window_scanline();

        // Window não deve renderizar (WY > LY)
        for x in 3..11 {
            assert_eq!(
                ppu.framebuffer[5 * 160 + x],
                0,
                "Window não deveria renderizar quando WY > LY"
            );
        }
    }
}
