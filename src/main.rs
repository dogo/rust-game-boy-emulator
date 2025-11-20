mod GB;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <caminho_para_rom.gb>");
        return;
    }
    let rom_path = &args[1];
    let data = fs::read(rom_path).expect("Falha ao ler ROM");
    let mut cpu = GB::CPU::CPU::new();
    cpu.load_rom(&data);
    cpu.init_post_boot();
    println!("ROM carregada: {} ({} bytes)", rom_path, data.len());

    // Ler título do cartucho (0x0134..0x0143)
    let mut title = String::new();
    for addr in 0x0134..=0x0143 {
        let ch = cpu.ram.read(addr);
        if ch == 0 { break; }
        if ch.is_ascii() { title.push(ch as char); }
    }
    println!("Título detectado: {}", title);

    // Info do cartucho: tipo, tamanho ROM/RAM
    let cart_type = cpu.ram.read(0x0147);
    let rom_size_code = cpu.ram.read(0x0148);
    let ram_size_code = cpu.ram.read(0x0149);

    let cart_str = match cart_type {
        0x00 => "ROM ONLY",
        0x01 | 0x02 | 0x03 => "MBC1",
        0x05 | 0x06 => "MBC2",
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 => "MBC3",
        0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => "MBC5",
        _ => "(desconhecido/variante)",
    };

    let rom_kb: u32 = match rom_size_code {
        0x00 => 32 * 1024,
        0x01 => 64 * 1024,
        0x02 => 128 * 1024,
        0x03 => 256 * 1024,
        0x04 => 512 * 1024,
        0x05 => 1024 * 1024,
        0x06 => 2048 * 1024,
        0x07 => 4096 * 1024,
        0x08 => 8192 * 1024,
        0x52 => 1152 * 1024,
        0x53 => 1280 * 1024,
        0x54 => 1536 * 1024,
        _ => 0,
    };

    let ram_kb: u32 = match ram_size_code {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    };

    println!(
        "Tipo: {:02X} ({}) | ROM: code={:02X} (~{} KB) | RAM: code={:02X} (~{} KB)",
        cart_type, cart_str, rom_size_code, rom_kb / 1024, ram_size_code, ram_kb / 1024
    );

    if matches!(cart_type, 0x01..=0x03 | 0x05 | 0x06 | 0x0F..=0x13 | 0x19..=0x1E) {
        println!(
            "Aviso: cartucho usa {} — mapeamento MBC ainda não implementado; ROMs maiores e bancos não funcionarão corretamente.",
            cart_str
        );
    }

    println!("PC inicial: {:04X}", cpu.registers.get_pc());
    println!("Iniciando trace ...");
    cpu.run_with_trace(usize::MAX);
    println!("Trace encerrado.");
}
