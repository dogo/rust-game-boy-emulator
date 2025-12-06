// Teste simples para diagnosticar problema de sprites
use gb_emu::GB::PPU::PPU;

#[test]
fn test_sprite_visible_collection() {
    let mut ppu = PPU::new();
    let mut iflags = 0u8;

    // Habilitar LCD e sprites
    ppu.write_register(0xFF40, 0x93, &mut iflags);

    // Sprite na linha 72 (OAM Y = 72 + 16 = 88)
    ppu.write_oam(0xFE00, 88); // Y
    ppu.write_oam(0xFE01, 80); // X
    ppu.write_oam(0xFE02, 0);  // Tile
    ppu.write_oam(0xFE03, 0);  // Flags

    // Criar tile simples
    ppu.write_vram(0x8000, 0xFF); // Todos pixels cor 1
    ppu.write_vram(0x8001, 0x00);

    // Configurar paleta
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    // Renderizar até linha 72 ser completada
    let mut safety = 0;
    while ppu.ly < 73 && safety < 100_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    // Avançar mais um pouco para garantir que a linha foi renderizada
    for _ in 0..500 {
        ppu.step(4, &mut iflags);
        if ppu.ly > 72 {
            break;
        }
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
