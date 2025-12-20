#![allow(non_snake_case)]

use gb_emu::GB;
use std::env;
use std::fs;

fn get_sav_path(rom_path: &str) -> String {
    std::path::Path::new(rom_path)
        .with_extension("sav")
        .to_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}.sav", rom_path))
}

fn run_trace(cpu: &mut GB::CPU::CPU, rom_data: &[u8]) {
    GB::cartridge::print_info(rom_data);
    GB::trace::run_with_trace(cpu, usize::MAX);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("Uso: cargo run -- <rom.gb> [--trace] [--headless]");
        eprintln!("  --trace     : Executa com trace detalhado");
        eprintln!("  --headless  : Executa sem interface gráfica");
        return;
    }

    // Encontra o arquivo ROM (não é um flag)
    let rom_path = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with("--"))
        .expect("Nenhum arquivo ROM especificado");

    let headless = args.iter().any(|a| a == "--headless");
    let trace = args.iter().any(|a| a == "--trace");
    let sav_path = get_sav_path(rom_path);

    // Carrega ROM
    let data = fs::read(rom_path).expect("Falha ao ler ROM");

    // Valida header
    if let Err(e) = GB::cartridge::validate_header(&data) {
        eprintln!("{}", e);
        return;
    }

    // Inicializa CPU
    let mut cpu = GB::CPU::CPU::new(data.clone());

    // Boot ROM ou estado pós-boot
    if let Ok(boot_rom) = fs::read("dmg_boot.bin") {
        cpu.bus.load_boot_rom(boot_rom);
        cpu.registers.set_pc(0x0000);
    } else {
        cpu.init_post_boot();
    }

    // Carrega save
    if let Err(e) = cpu.bus.load_cart_ram(&sav_path) {
        if !e.contains("No such file") {
            eprintln!("⚠️ Erro ao carregar save: {}", e);
        } else {
            println!("📂 Nenhum save encontrado, começando novo jogo.");
        }
    }

    println!("ROM carregada: {} ({} bytes)", rom_path, data.len());

    // Executa
    if headless {
        let result = GB::test_runner::run(&mut cpu);
        match result {
            GB::test_runner::TestResult::Passed => {
                println!("✅ Teste passou");
                std::process::exit(0);
            }
            GB::test_runner::TestResult::Failed(code) => {
                println!("❌ Teste falhou com código {}", code);
                std::process::exit(1);
            }
            GB::test_runner::TestResult::Timeout => {
                println!("⏱️ Teste deu timeout");
                std::process::exit(2);
            }
        }
    } else if trace {
        run_trace(&mut cpu, &data);
    } else {
        GB::cartridge::print_info(&data);
        GB::sdl_runner::run(&mut cpu);
    }

    // Salva RAM
    if let Err(e) = cpu.bus.save_cart_ram(&sav_path) {
        if !e.contains("No RAM to save") {
            eprintln!("⚠️ Erro ao salvar: {}", e);
        }
    }
}
