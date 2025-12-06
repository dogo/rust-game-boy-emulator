// Teste de debug para entender por que sprites não aparecem
use gb_emu::GB::PPU::PPU;

#[test]
fn debug_sprite_rendering() {
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

    // Sprite na linha 10, coluna 10
    let sy = 10i16;
    let sx = 10i16;
    ppu.write_oam(0xFE00, (sy + 16) as u8);
    ppu.write_oam(0xFE01, (sx + 8) as u8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00);

    // Desabilitar BG
    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Avançar até completar um frame
    let mut cycles = 0u32;
    while !ppu.frame_ready && cycles < 100000 {
        ppu.step(4, &mut iflags);
        cycles += 4;
    }

    // Verificar se sprite foi renderizado na linha 10
    let line_start = 10 * 160;
    let mut sprite_pixels = 0;
    for x in 0..160 {
        if ppu.framebuffer[line_start + x] != 0 {
            sprite_pixels += 1;
        }
    }

    println!("Debug: sprite_pixels na linha 10 = {}", sprite_pixels);
    println!("Debug: frame_ready = {}", ppu.frame_ready);
    println!("Debug: cycles executados = {}", cycles);

    // Este teste não faz assert, apenas imprime informações para debug
}
