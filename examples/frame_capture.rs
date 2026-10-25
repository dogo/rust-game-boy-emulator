//! Roda uma ROM sem janela seguindo um roteiro e grava screenshots em BMP.
//!
//! Uso: cargo run --release --example frame_capture -- <rom> <dir_saida> [save] <passos...>
//! Passos: `wait:N` (N frames), `press:BOTÃO`, `release:BOTÃO`, `tap:BOTÃO` (6 frames),
//! `shot:NOME` (grava NOME.bmp), `save:NOME` (grava a RAM do cartucho em NOME.sav).
//! O save de entrada, se dado (arquivo .sav), é só lido; nada é gravado fora de <dir_saida>.

use gb_emu::GB::CPU::{BootModel, CPU};
use std::time::Instant;
use std::{env, fs, path::Path, process};

const W: usize = 160;
const H: usize = 144;

fn run_frame(cpu: &mut CPU) {
    while !cpu.bus.ppu.frame_ready {
        cpu.execute_next();
    }
    cpu.bus.ppu.frame_ready = false;
}

fn set_button(cpu: &mut CPU, button: &str, down: bool) {
    if down {
        cpu.bus.joypad.press(button);
        if cpu.bus.joypad.take_interrupt_request() {
            cpu.bus.request_joypad_interrupt();
        }
    } else {
        cpu.bus.joypad.release(button);
    }
}

fn write_bmp(path: &Path, framebuffer: &[u8]) {
    let row = W * 3; // 480, já múltiplo de 4
    let size = 54 + row * H;
    let mut bmp = Vec::with_capacity(size);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&54u32.to_le_bytes());
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(W as i32).to_le_bytes());
    bmp.extend_from_slice(&(H as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&[0; 24]);
    // Mesmas cores do sdl_runner; BMP guarda as linhas de baixo para cima
    for y in (0..H).rev() {
        for &px in &framebuffer[y * W..(y + 1) * W] {
            let shade = [0xFF, 0xAA, 0x55, 0x00][px as usize & 3];
            bmp.extend_from_slice(&[shade; 3]);
        }
    }
    fs::write(path, bmp).expect("falha ao gravar screenshot");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("Uso: frame_capture <rom> <dir_saida> [save.sav] <passos...>");
        process::exit(1);
    }
    let rom = fs::read(&args[0]).expect("falha ao ler ROM");
    let out = Path::new(&args[1]);
    fs::create_dir_all(out).expect("falha ao criar diretório de saída");

    let mut cpu = CPU::new(rom);
    let model = if cpu.bus.cgb_mode {
        BootModel::Cgb
    } else {
        BootModel::DmgAbc
    };
    cpu.init_post_boot_model(model);

    let mut steps = &args[2..];
    if let Some(save) = steps.first().filter(|s| s.ends_with(".sav")) {
        cpu.bus.load_cart_ram(save).expect("falha ao carregar save");
        steps = &steps[1..];
    }

    let start = Instant::now();
    let mut frames = 0u64;
    for step in steps {
        let (cmd, arg) = step.split_once(':').unwrap_or((step, ""));
        match cmd {
            "wait" => {
                let n: u64 = arg.parse().expect("wait:N");
                for _ in 0..n {
                    run_frame(&mut cpu);
                }
                frames += n;
            }
            "press" => set_button(&mut cpu, arg, true),
            "release" => set_button(&mut cpu, arg, false),
            "tap" => {
                set_button(&mut cpu, arg, true);
                for _ in 0..6 {
                    run_frame(&mut cpu);
                }
                set_button(&mut cpu, arg, false);
                frames += 6;
            }
            "shot" => write_bmp(&out.join(format!("{arg}.bmp")), &cpu.bus.ppu.framebuffer),
            "save" => {
                let path = out.join(format!("{arg}.sav"));
                cpu.bus
                    .save_cart_ram(path.to_str().expect("caminho UTF-8"))
                    .expect("falha ao gravar save");
            }
            _ => {
                eprintln!("Passo inválido: {step}");
                process::exit(1);
            }
        }
    }
    let secs = start.elapsed().as_secs_f64();
    println!(
        "{frames} frames em {secs:.2}s ({:.0} fps, {:.1}x a velocidade real)",
        frames as f64 / secs,
        frames as f64 / secs / 59.7275
    );
}
