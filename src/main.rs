#![allow(non_snake_case)]

use gb_emu::GB;
use std::env;
use std::fs;
use std::time::{Duration, Instant};

fn print_cart_info(cpu: &GB::CPU::CPU) {
    let mut title = String::new();
    for addr in 0x0134..=0x0143 {
        let ch = cpu.ram.read(addr);
        if ch == 0 { break; }
        if ch.is_ascii() { title.push(ch as char); }
    }
    println!("Título: {}", title);
    let cart_type = cpu.ram.read(0x0147);
    let rom_size_code = cpu.ram.read(0x0148);
    let ram_size_code = cpu.ram.read(0x0149);
    let cart_str = match cart_type {
        0x00 => "ROM ONLY",
        0x01 | 0x02 | 0x03 => "MBC1",
        0x05 | 0x06 => "MBC2",
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 => "MBC3",
        0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => "MBC5",
        _ => "(desconhecido)",
    };
    let rom_kb: u32 = match rom_size_code {
        0x00 => 32 * 1024, 0x01 => 64 * 1024, 0x02 => 128 * 1024, 0x03 => 256 * 1024,
        0x04 => 512 * 1024, 0x05 => 1024 * 1024, 0x06 => 2048 * 1024, 0x07 => 4096 * 1024,
        0x08 => 8192 * 1024, 0x52 => 1152 * 1024, 0x53 => 1280 * 1024, 0x54 => 1536 * 1024, _ => 0,
    };
    let ram_kb: u32 = match ram_size_code {
        0x00 => 0, 0x01 => 2 * 1024, 0x02 => 8 * 1024, 0x03 => 32 * 1024, 0x04 => 128 * 1024, 0x05 => 64 * 1024, _ => 0,
    };
    println!(
        "Cart: {:02X} ({}) | ROM code {:02X} (~{} KB) | RAM code {:02X} (~{} KB)",
        cpu.ram.read(0x0147), cart_str, rom_size_code, rom_kb / 1024, ram_size_code, ram_kb / 1024
    );
}

fn run_trace(mut cpu: GB::CPU::CPU) {
    print_cart_info(&cpu);
    println!("Trace iniciado (CTRL+C para interromper)");
    GB::trace::run_with_trace(&mut cpu, usize::MAX);
    println!("Trace encerrado");
}

fn try_init_sdl() -> Result<sdl2::Sdl, String> {
    sdl2::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
    sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "nearest");
    sdl2::init()
}

fn run_sdl(mut cpu: GB::CPU::CPU) {
    print_cart_info(&cpu);
    println!("Iniciando modo gráfico SDL2 (ESC para sair)");

    // Tenta inicializar SDL2, forçando X11 se Wayland falhar
    let sdl_ctx = match try_init_sdl() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Falha SDL2 no Wayland: {}", e);
            println!("Tentando fallback para X11...");
            unsafe { std::env::set_var("SDL_VIDEODRIVER", "x11"); }
            try_init_sdl().expect("Falha SDL2 init com X11 também")
        }
    };
    let video = sdl_ctx.video().expect("Falha subsistema de vídeo");
    let scale = 3u32; // escala 3x (160x144 → 480x432)
    let window = video
        .window("GB Emulator", 160 * scale, 144 * scale)
        .position_centered()
        .build()
        .expect("Falha ao criar janela");
    let mut canvas = window.into_canvas().present_vsync().build().expect("Falha canvas");
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 160, 144)
        .expect("Falha texture");

    let mut event_pump = sdl_ctx.event_pump().expect("Falha event pump");
    let mut last_frame = Instant::now();
    let frame_duration = Duration::from_micros(16_667); // ~60 FPS
    let mut frame_counter: u64 = 0;

    loop {
        let mut exit = false;
        // Processa eventos SDL com robustez contra eventos inválidos
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        // Tenta processar eventos; se der panic (valor enum inválido), pula para próximo frame
        let events_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            event_pump.poll_iter().collect::<Vec<_>>()
        }));

        match events_result {
            Ok(events) => {
                for event in events {
                    match event {
                        Event::Quit { .. } => { exit = true; break; }
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { exit = true; break; }
                        Event::KeyDown { keycode: Some(k), repeat: false, .. } => match k {
                            Keycode::Right => cpu.press_button("RIGHT"),
                            Keycode::Left => cpu.press_button("LEFT"),
                            Keycode::Up => cpu.press_button("UP"),
                            Keycode::Down => cpu.press_button("DOWN"),
                            Keycode::Z => cpu.press_button("A"),
                            Keycode::X => cpu.press_button("B"),
                            Keycode::Return => cpu.press_button("START"),
                            Keycode::Backspace => cpu.press_button("SELECT"),
                            _ => {}
                        },
                        Event::KeyUp { keycode: Some(k), repeat: false, .. } => match k {
                            Keycode::Right => cpu.release_button("RIGHT"),
                            Keycode::Left => cpu.release_button("LEFT"),
                            Keycode::Up => cpu.release_button("UP"),
                            Keycode::Down => cpu.release_button("DOWN"),
                            Keycode::Z => cpu.release_button("A"),
                            Keycode::X => cpu.release_button("B"),
                            Keycode::Return => cpu.release_button("START"),
                            Keycode::Backspace => cpu.release_button("SELECT"),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            },
            Err(_) => {
                eprintln!("Aviso: evento SDL com valor inválido (0x207) ignorado");
            }
        }
        if exit { break; }

        // Executa instruções aproximando um frame (70224 ciclos ≈ 154 linhas * 456 ciclos)
        let target_cycles = cpu.cycles + 70224;
        while cpu.cycles < target_cycles {
            let _ = cpu.execute_next();
        }
        frame_counter += 1;

        // Atualiza textura
        let fb = &cpu.ram.ppu.framebuffer;
        texture.with_lock(None, |buffer: &mut [u8], _pitch| {
            for y in 0..144 {
                for x in 0..160 {
                    let idx = y * 160 + x;
                    let color = fb[idx];
                    // Mapear 0..3 → tons de cinza (0=branco, 3=preto)
                    let shade = match color { 0 => 0xFF, 1 => 0xAA, 2 => 0x55, _ => 0x00 };
                    let off = idx * 3;
                    buffer[off] = shade;      // R
                    buffer[off + 1] = shade;  // G
                    buffer[off + 2] = shade;  // B
                }
            }
        }).unwrap();
        canvas.clear();
        use sdl2::rect::Rect;
        canvas.copy(&texture, None, Some(Rect::new(0, 0, 160 * scale, 144 * scale))).unwrap();
        canvas.present();

        // Sincroniza FPS
        let elapsed = last_frame.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_frame = Instant::now();

        if frame_counter % 120 == 0 {
            println!("Frames: {} | PC: {:04X} | LY: {}", frame_counter, cpu.registers.get_pc(), cpu.ram.ppu.ly);
        }
    }
    println!("Encerrando SDL2 após {} frames", frame_counter);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <rom.gb> [--trace]");
        return;
    }
    let rom_path = &args[1];
    let data = fs::read(rom_path).expect("Falha ao ler ROM");
    let mut cpu = GB::CPU::CPU::new();
    cpu.load_rom(&data);
    cpu.init_post_boot();
    println!("ROM carregada: {} ({} bytes)", rom_path, data.len());

    if args.iter().any(|a| a == "--trace") {
        run_trace(cpu);
    } else {
        run_sdl(cpu);
    }
}
