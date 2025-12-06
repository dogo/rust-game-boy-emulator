// Teste para verificar sprites em múltiplas linhas
use gb_emu::GB::PPU::PPU;

#[test]
fn test_sprite_multiple_lines() {
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    // Configurar LCD e sprites
    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    // Limpar framebuffer
    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Criar tile simples (tile 0)
    for row in 0..8 {
        ppu.write_vram(0x8000 + (row * 2) as u16, 0xFF);
        ppu.write_vram(0x8000 + (row * 2 + 1) as u16, 0x00);
    }

    // Desabilitar BG
    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Testar sprites em diferentes linhas
    let test_lines = vec![10, 50, 72, 100];
    let mut results = Vec::new();

    for &sy in &test_lines {
        // Limpar OAM
        for i in 0..160 {
            ppu.write_oam(0xFE00 + i as u16, 0);
        }

        // Sprite na linha sy
        ppu.write_oam(0xFE00, (sy + 16) as u8);
        ppu.write_oam(0xFE01, 80); // X = 80
        ppu.write_oam(0xFE02, 0);
        ppu.write_oam(0xFE03, 0x00);

        // Limpar framebuffer
        for p in ppu.framebuffer.iter_mut() { *p = 0; }
        ppu.frame_ready = false;

        // Avançar até completar um frame
        let mut cycles = 0u32;
        while !ppu.frame_ready && cycles < 100000 {
            ppu.step(4, &mut iflags);
            cycles += 4;
        }

        // Verificar se sprite foi renderizado
        let line_start = sy * 160;
        let mut sprite_pixels = 0;
        for x in 0..160 {
            if ppu.framebuffer[line_start + x] != 0 {
                sprite_pixels += 1;
            }
        }

        results.push((sy, sprite_pixels));
        println!("Linha {}: {} pixels do sprite", sy, sprite_pixels);
    }

    // Verificar que pelo menos algumas linhas renderizaram sprites
    let mut success_count = 0;
    for (sy, pixels) in &results {
        if *pixels >= 6 {
            success_count += 1;
        }
    }

    println!("Resultados: {:?}", results);
    assert!(success_count >= 2, "Pelo menos 2 linhas devem renderizar sprites corretamente. Resultados: {:?}", results);
}
