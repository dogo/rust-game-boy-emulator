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

    println!("PC inicial: {:04X}", cpu.registers.get_pc());
    println!("Iniciando trace (200 instruções máx)...");
    cpu.run_with_trace(200);
    println!("Trace encerrado.");
}
