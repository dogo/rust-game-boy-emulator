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
        eprintln!("Uso: cargo run -- <rom.gb> [--trace] [--headless] [--model <modelo>]");
        eprintln!("  --trace     : Executa com trace detalhado");
        eprintln!("  --headless  : Executa sem interface gráfica");
        eprintln!(
            "  --model     : Hardware emulado ({}); padrão: dmg. ROMs só CGB exigem modelo CGB",
            GB::CPU::BootModel::NAMES
        );
        return;
    }

    // ROM é o primeiro argumento que não é flag nem valor de --model
    let mut rom_path = None;
    let mut boot_model = None;
    let mut rest = args.iter().skip(1);
    while let Some(arg) = rest.next() {
        if arg == "--model" {
            let name = rest.next().map(String::as_str).unwrap_or("");
            let Some(model) = GB::CPU::BootModel::from_name(name) else {
                eprintln!(
                    "❌ Modelo inválido: '{}'. Use: {}",
                    name,
                    GB::CPU::BootModel::NAMES
                );
                std::process::exit(1);
            };
            boot_model = Some(model);
        } else if !arg.starts_with("--") && rom_path.is_none() {
            rom_path = Some(arg.as_str());
        }
    }
    let rom_path = rom_path.expect("Nenhum arquivo ROM especificado");

    let headless = args.iter().any(|a| a == "--headless");
    let trace = args.iter().any(|a| a == "--trace");
    let sav_path = get_sav_path(rom_path);

    // Carrega ROM
    let data = fs::read(rom_path).expect("Falha ao ler ROM");

    // Valida header
    if let Err(e) = GB::cartridge::validate_header(&data) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let model = boot_model.unwrap_or(GB::CPU::BootModel::DmgAbc);
    // O PPU ainda não desenha os recursos CGB, mas ROMs só CGB exigem hardware CGB.
    if GB::cartridge::is_cgb_only_rom(&data) && !model.is_cgb() {
        eprintln!(
            "❌ ROM exclusiva de Game Boy Color: o emulador desenha só como o DMG. \
             Use --model cgb para executar mesmo assim, sem os gráficos CGB."
        );
        std::process::exit(1);
    }

    // Inicializa CPU
    let mut cpu = GB::CPU::CPU::new_with_model(data.clone(), model);

    // Boot ROM ou estado pós-boot; o boot ROM do DMG deixaria os registradores errados nos outros modelos
    if model == GB::CPU::BootModel::DmgAbc
        && let Ok(boot_rom) = fs::read("dmg_boot.bin")
    {
        cpu.bus.load_boot_rom(boot_rom);
        cpu.registers.set_pc(0x0000);
    } else {
        cpu.init_post_boot();
    }

    // Carrega save
    match cpu.bus.load_cart_ram(&sav_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("📂 Nenhum save encontrado, começando novo jogo.");
        }
        Err(error) => {
            eprintln!("⚠️ Erro ao carregar save {sav_path}: {error}");
            std::process::exit(1);
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
        GB::sdl_runner::run(&mut cpu, &sav_path);
    }

    // Salva RAM
    if let Err(e) = cpu.bus.save_cart_ram(&sav_path) {
        eprintln!("⚠️ Erro ao salvar: {}", e);
        std::process::exit(1);
    }
}
