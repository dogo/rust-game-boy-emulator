// Teste de debug para flip horizontal - verifica pixel por pixel
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
fn test_flip_horizontal_debug() {
    let mut ppu = PPU::new();
    let mut iflags: u8 = 0;

    ppu.write_register(0xFF40, 0x80 | 0x02 | 0x10, &mut iflags);
    ppu.write_register(0xFF48, 0xE4, &mut iflags);

    for p in ppu.framebuffer.iter_mut() { *p = 0; }

    // Tile com padrão muito claro: apenas pixel 0 (esquerda) = cor 3
    // lower = 0x80 (10000000), upper = 0x80 (10000000) -> pixel 0 = cor 3
    for row in 0..8 {
        ppu.write_vram(0x8000 + row * 2, 0x80);     // LSB: apenas bit 7
        ppu.write_vram(0x8000 + row * 2 + 1, 0x80); // MSB: apenas bit 7 -> cor 3
    }

    let mut lcdc = ppu.read_register(0xFF40);
    lcdc &= !0x01;
    ppu.write_register(0xFF40, lcdc, &mut iflags);

    // Sprite 1: SEM flip - pixel 0 (esquerda) deve ser cor 3
    ppu.write_oam(0xFE00, 20 + 16);
    ppu.write_oam(0xFE01, 20 + 8);
    ppu.write_oam(0xFE02, 0);
    ppu.write_oam(0xFE03, 0x00); // sem flip

    // Sprite 2: COM flip - pixel 7 (direita) deve ser cor 3
    ppu.write_oam(0xFE04, 20 + 16);
    ppu.write_oam(0xFE05, 40 + 8);
    ppu.write_oam(0xFE06, 0);
    ppu.write_oam(0xFE07, 0x20); // flip horizontal

    let mut safety = 0;
    while !ppu.frame_ready && safety < 600_000 {
        ppu.step(4, &mut iflags);
        safety += 1;
    }

    assert!(ppu.frame_ready, "Frame não completou");

    write_ppm(Path::new("/tmp/sprite_flip_debug.ppm"), &ppu.framebuffer)
        .expect("Erro ao escrever PPM");

    let line = 20;

    // Sprite 1 (sem flip): pixel 0 (x=20) deve ser cor 3
    let sprite1_pixel0 = ppu.framebuffer[line * 160 + 20];
    let sprite1_pixel7 = ppu.framebuffer[line * 160 + 27];

    // Sprite 2 (com flip): pixel 7 (x=47) deve ser cor 3
    let sprite2_pixel0 = ppu.framebuffer[line * 160 + 40];
    let sprite2_pixel7 = ppu.framebuffer[line * 160 + 47];

    println!("Sprite 1 (sem flip): pixel[0]={}, pixel[7]={}", sprite1_pixel0, sprite1_pixel7);
    println!("Sprite 2 (com flip): pixel[0]={}, pixel[7]={}", sprite2_pixel0, sprite2_pixel7);

    // Sem flip: pixel 0 deve ser cor 3, pixel 7 deve ser 0
    assert_eq!(sprite1_pixel0, 3, "Sprite 1 (sem flip): pixel 0 deve ser cor 3");
    assert_eq!(sprite1_pixel7, 0, "Sprite 1 (sem flip): pixel 7 deve ser 0");

    // Com flip: pixel 7 deve ser cor 3, pixel 0 deve ser 0
    assert_eq!(sprite2_pixel7, 3, "Sprite 2 (com flip): pixel 7 deve ser cor 3");
    assert_eq!(sprite2_pixel0, 0, "Sprite 2 (com flip): pixel 0 deve ser 0");
}
