// Teste simples para diagnosticar problema de sprites
use gb_emu::GB::PPU::PPU;

#[test]
fn test_sprite_visible_collection() {
    let mut ppu = PPU::new();
    ppu.lcdc = 0x93; // LCD on, sprites on
    ppu.current_line = 72; // Linha onde o sprite está

    // Sprite na linha 72 (OAM Y = 72 + 16 = 88)
    ppu.oam[0] = 88; // Y
    ppu.oam[1] = 80; // X
    ppu.oam[2] = 0;  // Tile
    ppu.oam[3] = 0;  // Flags

    // Criar tile simples
    ppu.vram[0] = 0xFF; // Todos pixels cor 1
    ppu.vram[1] = 0x00;

    // Coletar sprites visíveis
    ppu.collect_visible_sprites();

    // Verificar se sprite foi coletado
    // Como visible_sprites é privado, vamos verificar indiretamente
    // através do comportamento durante mode 3

    // Simular mode 3
    let mut iflags = 0u8;
    ppu.change_mode(3, &mut iflags);

    // Avançar alguns ciclos para processar sprite
    for _ in 0..200 {
        ppu.step(4, &mut iflags);
    }

    // Verificar se algum pixel foi renderizado na linha 72
    let line_start = 72 * 160;
    let mut has_pixels = false;
    for x in 0..160 {
        if ppu.framebuffer[line_start + x] != 0 {
            has_pixels = true;
            break;
        }
    }

    println!("Sprite visible collection test: has_pixels = {}", has_pixels);
    // Não vamos fazer assert aqui, apenas documentar o resultado
}
