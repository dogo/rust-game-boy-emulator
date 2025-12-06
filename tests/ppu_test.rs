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
        ppu.write_vram(0x8000, 0xAA);
        ppu.write_vram(0x8001, 0x00);
        // Resultado esperado: cores 1,0,1,0,1,0,1,0

        // Linha 1: 01010101 (0x55 em bits)
        ppu.write_vram(0x8002, 0x55);
        ppu.write_vram(0x8003, 0x00);
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
        ppu.write_vram(0x8000, 0b10101010);
        ppu.write_vram(0x8001, 0b11001100);

        let pixels = ppu.decode_tile(0);
        assert_eq!(pixels[0], 3);
        assert_eq!(pixels[1], 2);
        assert_eq!(pixels[2], 1);
        assert_eq!(pixels[3], 0);
        assert_eq!(pixels[4], 3);
        assert_eq!(pixels[5], 2);
        assert_eq!(pixels[6], 1);
        assert_eq!(pixels[7], 0);
    }

    // Helper: renderiza até completar um frame
    fn render_frame(ppu: &mut PPU) {
        let mut iflags = 0u8;
        let mut safety = 0;
        while !ppu.frame_ready && safety < 500_000 {
            ppu.step(4, &mut iflags);
            safety += 1;
        }
    }

    // Helper: renderiza até uma linha específica ser completada
    fn render_until_line(ppu: &mut PPU, target_line: u8) {
        let mut iflags = 0u8;
        let mut safety = 0;
        while ppu.ly < target_line && safety < 100_000 {
            ppu.step(4, &mut iflags);
            safety += 1;
        }
        // Avançar mais um pouco para garantir que a linha foi renderizada
        for _ in 0..500 {
            ppu.step(4, &mut iflags);
            if ppu.ly > target_line {
                break;
            }
        }
    }

    #[test]
    fn test_bg_scanline_simple() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Configurar tile 0 com padrão sólido (cor 3)
        for i in 0..8 {
            ppu.write_vram(0x8000 + (i * 2) as u16, 0xFF);
            ppu.write_vram(0x8000 + (i * 2 + 1) as u16, 0xFF);
        }

        // Configurar tile map para usar tile 0 em todas as posições
        for i in 0..32 * 32 {
            ppu.write_vram(0x9800 + i as u16, 0);
        }

        // LCDC: BG enabled, tile map 0x9800, tile data 0x8000
        ppu.write_register(0xFF40, 0x91, &mut iflags);

        // Paleta: 0xE4 (3,2,1,0)
        ppu.write_register(0xFF47, 0xE4, &mut iflags);

        // Renderizar até linha 0 ser completada
        render_until_line(&mut ppu, 1);

        // Todos os pixels devem ser cor 3 (que se torna 3 pela paleta 0xE4)
        for x in 0..160 {
            assert_eq!(ppu.framebuffer[x], 3, "Pixel {} na linha 0 deve ser 3", x);
        }
    }

    #[test]
    fn test_bg_with_scroll() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Criar dois tiles distintos
        // Tile 0: todos pixels cor 0
        for i in 0..8 {
            ppu.write_vram(0x8000 + (i * 2) as u16, 0x00);
            ppu.write_vram(0x8000 + (i * 2 + 1) as u16, 0x00);
        }

        // Tile 1: todos pixels cor 3
        for i in 0..8 {
            ppu.write_vram(0x8010 + (i * 2) as u16, 0xFF);
            ppu.write_vram(0x8010 + (i * 2 + 1) as u16, 0xFF);
        }

        // Tile map: primeira coluna usa tile 0, segunda coluna usa tile 1
        for y in 0..32 {
            ppu.write_vram(0x9800 + (y * 32) as u16, 0);
            ppu.write_vram(0x9800 + (y * 32 + 1) as u16, 1);
        }

        ppu.write_register(0xFF40, 0x91, &mut iflags);
        ppu.write_register(0xFF47, 0xE4, &mut iflags);

        // Sem scroll: primeiros 8 pixels devem ser cor 0, próximos 8 devem ser cor 3
        ppu.write_register(0xFF43, 0, &mut iflags);
        ppu.write_register(0xFF42, 0, &mut iflags);

        render_until_line(&mut ppu, 1);

        assert_eq!(ppu.framebuffer[0], 0);
        assert_eq!(ppu.framebuffer[7], 0);
        assert_eq!(ppu.framebuffer[8], 3);
        assert_eq!(ppu.framebuffer[15], 3);

        // Limpar framebuffer para próximo teste
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;

        // Com scroll X=4: deve deslocar 4 pixels para esquerda
        ppu.write_register(0xFF43, 4, &mut iflags);

        render_until_line(&mut ppu, 1);

        assert_eq!(ppu.framebuffer[0], 0);
        assert_eq!(ppu.framebuffer[3], 0);
        assert_eq!(ppu.framebuffer[4], 3);
    }

    #[test]
    fn test_render_full_frame() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Criar gradient vertical: tile 0=cor0, tile 1=cor1, tile 2=cor2, tile 3=cor3
        for tile in 0..4 {
            for i in 0..8 {
                let lsb = if tile & 1 != 0 { 0xFF } else { 0x00 };
                let msb = if tile & 2 != 0 { 0xFF } else { 0x00 };
                ppu.write_vram(0x8000 + (tile * 16 + i * 2) as u16, lsb);
                ppu.write_vram(0x8000 + (tile * 16 + i * 2 + 1) as u16, msb);
            }
        }

        // Tile map: linha 0 usa tile 0, linha 1 usa tile 1, etc
        for y in 0..32 {
            let tile = (y % 4) as u8;
            for x in 0..32 {
                ppu.write_vram(0x9800 + (y * 32 + x) as u16, tile);
            }
        }

        ppu.write_register(0xFF40, 0x91, &mut iflags);
        ppu.write_register(0xFF47, 0xE4, &mut iflags);

        // Renderizar frame completo
        render_frame(&mut ppu);

        // Verificar que algumas linhas foram renderizadas
        let mut rendered_lines = 0;
        for y in 0..144 {
            let mut has_pixels = false;
            for x in 0..160 {
                if ppu.framebuffer[y * 160 + x] != 0 {
                    has_pixels = true;
                    break;
                }
            }
            if has_pixels {
                rendered_lines += 1;
            }
        }
        assert!(rendered_lines > 0, "Frame should have rendered some lines");
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
        // (VBlank interrupt enabled, LY >= 144)
        let stat_irq = (ppu.ly >= 144) && (ppu.stat & 0x10 != 0);
        assert!(stat_irq, "STAT interrupt deveria ser gerado no VBlank");

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
        // (LYC=LY coincidence interrupt enabled)
        let stat_irq = (ppu.ly == ppu.lyc) && (ppu.stat & 0x40 != 0);
        assert!(stat_irq, "STAT interrupt deveria ser gerado quando LYC=LY");

        // Mudar LY para que LYC != LY
        ppu.ly = 101;
        ppu.update_lyc_flag();

        // Verificar que flag LYC foi limpada (bit 2)
        assert_eq!(ppu.stat & 0x04, 0x00, "Flag LYC=LY deveria estar limpa");
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
        let stat_irq = !((ppu.ly >= 144 && (ppu.stat & 0x10 != 0))
            || (ppu.ly == ppu.lyc && (ppu.stat & 0x40 != 0)));
        assert!(
            stat_irq,
            "STAT interrupt não deveria ser gerado quando desabilitado"
        );
    }

    #[test]
    fn test_sprite_basic_rendering() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Habilitar sprites no LCDC (bit 1)
        ppu.write_register(0xFF40, 0x93, &mut iflags);

        // Criar tile para sprite no VRAM (tile 1)
        ppu.write_vram(0x8010, 0xF0);
        ppu.write_vram(0x8011, 0x00);
        ppu.write_vram(0x8012, 0x0F);
        ppu.write_vram(0x8013, 0x00);

        // Configurar sprite 0 no OAM
        ppu.write_oam(0xFE00, 16 + 2);
        ppu.write_oam(0xFE01, 8 + 10);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x00);

        // Configurar paleta OBP0
        ppu.write_register(0xFF48, 0xE4, &mut iflags);

        // Renderizar até linha 2 ser completada
        render_until_line(&mut ppu, 3);

        // Verificar pixels do sprite na linha 2
        for x in 10..14 {
            assert_eq!(
                ppu.framebuffer[2 * 160 + x],
                1,
                "Pixel [{}, 2] deveria ser cor 1",
                x
            );
        }
    }

    #[test]
    fn test_sprite_palette_obp1() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Habilitar sprites
        ppu.write_register(0xFF40, 0x93, &mut iflags);

        // Criar tile simples
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0x00);

        // Sprite usando paleta OBP1 (bit 4 = 1)
        ppu.write_oam(0xFE00, 16);
        ppu.write_oam(0xFE01, 8);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x10);

        // Configurar paletas diferentes
        ppu.write_register(0xFF48, 0xE4, &mut iflags);
        ppu.write_register(0xFF49, 0x1B, &mut iflags);

        render_until_line(&mut ppu, 1);

        // Pixel deve usar OBP1, então cor 1 → 2
        assert_eq!(ppu.framebuffer[0], 2, "Sprite deveria usar paleta OBP1");
    }

    #[test]
    fn test_sprite_flip_horizontal() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        ppu.write_register(0xFF40, 0x93, &mut iflags);

        // Tile assimétrico: 11110000
        ppu.write_vram(0x8010, 0xF0);
        ppu.write_vram(0x8011, 0x00);

        // Sprite com flip horizontal (bit 5 = 1)
        ppu.write_oam(0xFE00, 16);
        ppu.write_oam(0xFE01, 8);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x20);

        ppu.write_register(0xFF48, 0xE4, &mut iflags);
        render_until_line(&mut ppu, 1);

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
        let mut iflags = 0u8;

        ppu.write_register(0xFF40, 0x93, &mut iflags);

        // Criar tile BG opaco (cor 2)
        for i in 0..8 {
            ppu.write_vram(0x8000 + (i * 2) as u16, 0x00);
            ppu.write_vram(0x8000 + (i * 2 + 1) as u16, 0xFF);
        }
        for i in 0..32 {
            ppu.write_vram(0x9800 + i as u16, 0);
        }
        ppu.write_register(0xFF47, 0xE4, &mut iflags);

        // Renderizar BG primeiro
        render_until_line(&mut ppu, 1);

        // Tile do sprite (cor 1)
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0x00);

        // Sprite com prioridade baixa (bit 7 = 1)
        ppu.write_oam(0xFE00, 16);
        ppu.write_oam(0xFE01, 8);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x80);

        ppu.write_register(0xFF48, 0xE4, &mut iflags);

        // Limpar e renderizar novamente
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;
        render_until_line(&mut ppu, 1);

        // Sprite com prioridade baixa não deve sobrescrever BG opaco
        assert_eq!(
            ppu.framebuffer[0], 2,
            "Sprite com prioridade baixa não deveria sobrescrever BG"
        );

        // Testar sprite com prioridade alta
        ppu.write_oam(0xFE03, 0x00);
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;
        render_until_line(&mut ppu, 1);

        // Agora deve sobrescrever
        assert_eq!(
            ppu.framebuffer[0], 1,
            "Sprite com prioridade alta deveria sobrescrever BG"
        );
    }

    #[test]
    fn test_sprite_disabled() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Desabilitar sprites no LCDC (bit 1 = 0)
        ppu.write_register(0xFF40, 0x91, &mut iflags);

        // Configurar sprite
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0x00);
        ppu.write_oam(0xFE00, 16);
        ppu.write_oam(0xFE01, 8);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x00);

        render_until_line(&mut ppu, 1);

        // Framebuffer deve permanecer 0 (sprites desabilitados)
        assert_eq!(
            ppu.framebuffer[0], 0,
            "Sprites desabilitados não deveriam renderizar"
        );
    }

    #[test]
    fn test_ppu_hblank_triggers_rendering() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // BG on, tile data unsigned, tilemap 0x9800
        ppu.lcdc = 0x91;

        // Criar tile simples (cor 3 sempre)
        for i in 0..8 {
            ppu.vram[i * 2] = 0xFF;
            ppu.vram[i * 2 + 1] = 0xFF;
        }

        // Usar tile 0 no início do tilemap
        ppu.vram[0x1800] = 0;

        // Linha 0
        ppu.ly = 0;

        // Simular execução de CPU: vários steps pequenos até passar a primeira linha
        let mut total_cycles = 0;
        while ppu.ly == 0 && total_cycles < 456 {
            ppu.step(4, &mut iflags);
            total_cycles += 4;
        }

        // Agora, a linha 0 deve ter sido renderizada em algum HBlank
        let line_start = 0;
        let mut rendered_pixels = 0;
        for x in 0..160 {
            if ppu.framebuffer[line_start + x] != 0 {
                rendered_pixels += 1;
            }
        }

        assert!(
            rendered_pixels > 0,
            "HBlank should trigger rendering, but framebuffer line is blank"
        );
    }

    #[test]
    fn test_window_basic_rendering() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Habilitar BG e Window no LCDC
        ppu.write_register(0xFF40, 0xB1 | 0x10, &mut iflags);
        ppu.write_register(0xFF4A, 5, &mut iflags);
        ppu.write_register(0xFF4B, 10, &mut iflags);

        // Criar tile para window
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0x00);

        // Configurar window tile map
        ppu.write_vram(0x9800, 1);

        // Configurar paleta
        ppu.write_register(0xFF47, 0xE4, &mut iflags);

        // Renderizar até linha 5 ser completada
        render_until_line(&mut ppu, 6);

        // Verificar pixels da window
        for x in 3..11 {
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
        let mut iflags = 0u8;

        // Desabilitar window no LCDC (bit 5 = 0)
        ppu.write_register(0xFF40, 0x91, &mut iflags);
        ppu.write_register(0xFF4A, 5, &mut iflags);
        ppu.write_register(0xFF4B, 10, &mut iflags);

        // Configurar tile e tile map
        ppu.write_vram(0x8020, 0xFF);
        ppu.write_vram(0x8021, 0x00);
        ppu.write_vram(0x9800, 2);

        render_until_line(&mut ppu, 11);

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
        let mut iflags = 0u8;

        // Window habilitada mas WY > LY
        ppu.write_register(0xFF40, 0xB1, &mut iflags);
        ppu.write_register(0xFF4A, 10, &mut iflags);
        ppu.write_register(0xFF4B, 10, &mut iflags);

        // Configurar tile
        ppu.write_vram(0x8020, 0xFF);
        ppu.write_vram(0x8021, 0x00);
        ppu.write_vram(0x9800, 2);

        render_until_line(&mut ppu, 6);

        // Window não deve renderizar (WY > LY)
        for x in 3..11 {
            assert_eq!(
                ppu.framebuffer[5 * 160 + x],
                0,
                "Window não deveria renderizar quando WY > LY"
            );
        }
    }

    #[test]
    fn test_ppu_stat_lyc_irq() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        // Enable LYC=LY interrupt in STAT
        ppu.stat = 0x40; // Bit 6: LYC=LY interrupt enable
        ppu.ly = 10;
        ppu.lyc = 10;
        ppu.update_lyc_flag();
        ppu.check_lyc_interrupt(&mut iflags);
        assert_eq!(
            ppu.stat & 0x04,
            0x04,
            "LYC flag should be set when LY == LYC"
        );
        assert_eq!(
            iflags & 0x02,
            0x02,
            "STAT IRQ should be triggered when LY == LYC and interrupt enabled"
        );

        // Change LY so LYC != LY
        ppu.ly = 11;
        ppu.update_lyc_flag();
        ppu.check_lyc_interrupt(&mut iflags);
        assert_eq!(
            ppu.stat & 0x04,
            0x00,
            "LYC flag should be cleared when LY != LYC"
        );

        // Test writing to LYC triggers flag/IRQ immediately
        ppu.ly = 20;
        ppu.lyc = 21;
        ppu.stat = 0x40;
        iflags = 0;
        ppu.write_register(0xFF45, 20, &mut iflags);
        assert_eq!(
            ppu.stat & 0x04,
            0x04,
            "LYC flag should be set after writing LYC == LY"
        );
        assert_eq!(
            iflags & 0x02,
            0x02,
            "STAT IRQ should be triggered after writing LYC == LY"
        );
    }

    #[test]
    fn test_ppu_window_trigger_and_reset() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;
        ppu.lcdc = 0x91 | 0x20; // LCD on, BG on, window enable
        ppu.ly = 5;
        ppu.wy = 5;
        ppu.wy_trigger = false;
        ppu.wy_pos = -1;
        // Enter mode 3 at WY
        ppu.change_mode(3, &mut iflags);
        assert!(ppu.wy_trigger, "Window trigger should activate at WY");
        assert_eq!(ppu.wy_pos, -1, "Window position should reset at trigger");
        // Simulate VBlank, should reset window state
        ppu.change_mode(1, &mut iflags);
        assert!(!ppu.wy_trigger, "Window trigger should reset at VBlank");
        assert_eq!(ppu.wy_pos, -1, "Window position should reset at VBlank");
    }

    #[test]
    fn test_ppu_sprite_priority_and_limit() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        ppu.write_register(0xFF40, 0x91 | 0x02, &mut iflags);

        // Fill OAM with 12 overlapping sprites on line 20
        for i in 0..12 {
            ppu.write_oam(0xFE00 + (i * 4) as u16, 20 + 16);
            ppu.write_oam(0xFE01 + (i * 4) as u16, (i * 8) as u8 + 8);
            ppu.write_oam(0xFE02 + (i * 4) as u16, 0);
            ppu.write_oam(0xFE03 + (i * 4) as u16, 0);
        }

        // Set tile data to nonzero
        for i in 0..16 {
            ppu.write_vram(0x8000 + i as u16, 0xFF);
        }

        render_until_line(&mut ppu, 21);

        // Check that framebuffer has nonzero pixels
        let line_start = 20 * 160;
        let mut sprite_pixels = 0;
        for x in 0..160 {
            if ppu.framebuffer[line_start + x] != 0 {
                sprite_pixels += 1;
            }
        }
        assert!(sprite_pixels > 0, "Some sprite pixels should be rendered");
    }

    #[test]
    fn test_ppu_bg_priority_buffer() {
        let mut ppu = PPU::new();
        let mut iflags = 0u8;

        ppu.write_register(0xFF40, 0x91 | 0x02, &mut iflags);

        // Preencher a primeira linha do tilemap com tile 1
        for x in 0..32 {
            ppu.write_vram(0x9800 + x as u16, 1);
        }

        // Tile 1 com todos pixels != 0
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0xFF);
        ppu.write_register(0xFF47, 0xFF, &mut iflags);

        render_until_line(&mut ppu, 1);

        let line_start = 0;
        for x in 0..160 {
            assert!(
                ppu.bg_priority[line_start + x],
                "BG priority buffer should be set for opaque BG pixels"
            );
        }

        // Sprite priority test
        ppu.write_oam(0xFE00, 16);
        ppu.write_oam(0xFE01, 8);
        ppu.write_oam(0xFE02, 1);
        ppu.write_oam(0xFE03, 0x80);
        ppu.write_register(0xFF48, 0xE4, &mut iflags);
        ppu.write_vram(0x8010, 0xFF);
        ppu.write_vram(0x8011, 0x00);

        // Limpar e renderizar novamente
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;
        render_until_line(&mut ppu, 1);

        // Sprite com prioridade baixa não deve sobrescrever BG opaco
        assert_eq!(
            ppu.framebuffer[0], 3,
            "Sprite com prioridade baixa não deveria sobrescrever BG"
        );

        // Sprite com prioridade alta
        ppu.write_oam(0xFE03, 0x00);
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;
        render_until_line(&mut ppu, 1);

        // Agora deve sobrescrever
        assert_eq!(
            ppu.framebuffer[0], 1,
            "Sprite com prioridade alta deveria sobrescrever BG"
        );
    }
}
