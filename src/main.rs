#![allow(non_snake_case)]

use gb_emu::GB;
use std::env;
use std::fs;

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

fn run_trace(cpu: &mut GB::CPU::CPU) {
    print_cart_info(&cpu);
    println!("Trace iniciado (CTRL+C para interromper)");
    GB::trace::run_with_trace(cpu, usize::MAX);
    println!("Trace encerrado");
}

fn try_init_sdl() -> Result<sdl3::Sdl, String> {
    sdl3::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
    sdl3::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "nearest");
    sdl3::init().map_err(|e| format!("{:?}", e))
}

fn run_sdl(cpu: &mut GB::CPU::CPU) {
    print_cart_info(&cpu);
    println!("Iniciando modo gráfico SDL3 (ESC para sair)");

    let sdl_ctx = try_init_sdl().expect("Falha ao inicializar SDL3");
    let video = sdl_ctx.video().expect("Falha subsistema de vídeo");

    // ==== ÁUDIO ====
    let audio_subsystem = sdl_ctx.audio().expect("Falha subsistema de áudio");
    let desired_spec = sdl3::audio::AudioSpec {
        freq: Some(44100),
        channels: Some(2), // Stereo
        format: Some(sdl3::audio::AudioFormat::f32_sys()),
    };

    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    let audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>> = Arc::new(Mutex::new(VecDeque::new()));

    struct AudioCallbackData {
        buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,
    }

    impl sdl3::audio::AudioCallback<f32> for AudioCallbackData {
        fn callback(&mut self, stream: &mut sdl3::audio::AudioStream, requested: i32) {
            let mut audio_buffer = self.buffer.lock().unwrap();
            let mut out = Vec::<f32>::with_capacity((requested * 2) as usize);

            for _ in 0..requested {
                if let Some((l, r)) = audio_buffer.pop_front() {
                    out.push(l.clamp(-1.0, 1.0));
                    out.push(r.clamp(-1.0, 1.0));
                } else {
                    out.push(0.0);
                    out.push(0.0);
                }
            }

            let _ = stream.put_data_f32(&out);
        }
    }    let audio_device = audio_subsystem
        .open_playback_stream(
            &desired_spec,
            AudioCallbackData {
                buffer: audio_buffer.clone(),
            },
        )
        .expect("Falha ao abrir dispositivo de áudio");

    audio_device.resume().expect("Failed to start audio playback");

    // ==== VÍDEO ====
    let scale = 3u32;
    let window = video
        .window("GB Emulator", 160 * scale, 144 * scale)
        .position_centered()
        .build()
        .expect("Falha ao criar janela");
    let mut canvas = window.into_canvas();

    // Tentativa de habilitar VSync usando FFI direta
    unsafe {
        let renderer = canvas.raw();
        unsafe extern "C" {
            fn SDL_SetRenderVSync(renderer: *mut std::ffi::c_void, vsync: std::ffi::c_int) -> bool;
        }
        if SDL_SetRenderVSync(renderer as *mut std::ffi::c_void, 1) {
            println!("✅ VSync habilitado com sucesso!");
        } else {
            println!("❌ Falha ao habilitar VSync");
        }
    }

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl3::pixels::PixelFormat::RGB24, 160, 144)
        .expect("Falha texture");

    let mut event_pump = sdl_ctx.event_pump().expect("Falha event pump");
    let mut frame_counter: u64 = 0;

    use sdl3::event::Event;
    use sdl3::keyboard::Keycode;
    use sdl3::rect::Rect;
    use std::time::{Duration, Instant};

    // Controle de timing: VSync limita velocidade, executa 1/4 frame por loop

    // Constantes do Game Boy
    const GB_CPU_HZ: u64 = 4_194_304;           // Clock da CPU
    const GB_FPS: f64 = 59.7275;                // FPS reais do Game Boy
    const CYCLES_PER_FRAME: u64 = (GB_CPU_HZ as f64 / GB_FPS) as u64; // ~70224
    const SAMPLE_RATE: u32 = 44_100;            // Taxa de áudio
    const CYCLES_PER_SAMPLE: u64 = GB_CPU_HZ / SAMPLE_RATE as u64; // ~95 ciclos por sample

    let mut apu_cycle_accum: u64 = 0;
    let mut frame_cycle_accum: u64 = 0;  // Acumula ciclos até formar 1 frame
    let mut debug_print_timer = 0;

    // Timing baseado em tempo real
    let mut pending_cycles: f64 = 0.0;
    let mut last_time = Instant::now();

    loop {
        let mut exit = false;

        // Loop de eventos SDL3
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    exit = true;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    exit = true;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
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
                Event::KeyUp {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
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
                // qualquer outra coisa a gente só ignora
                _ => {}
            }
        }

        if exit {
            break;
        }

        // ==== Timing baseado em tempo real ====
        let now = Instant::now();
        let dt = now.duration_since(last_time);
        last_time = now;

        // Quantos ciclos o Game Boy REAL teria rodado em dt
        pending_cycles += dt.as_secs_f64() * (GB_CPU_HZ as f64);

        // Evita acumular demais se o app travar (alt+tab etc.)
        let max_backlog = (GB_CPU_HZ as f64) * 0.25; // no máx 0,25s de backlog
        if pending_cycles > max_backlog {
            pending_cycles = max_backlog;
        }

        // ==== CPU / PPU ====
        // Roda CPU enquanto ainda há ciclos "em débito"
        while pending_cycles >= 1.0 {
            let (instruction_cycles, _) = cpu.execute_next();
            let c = instruction_cycles as u64;

            pending_cycles -= c as f64;
            frame_cycle_accum += c;
            apu_cycle_accum += c;

            // AUDIO: gera sample sempre que acumula CYCLES_PER_SAMPLE
            while apu_cycle_accum >= CYCLES_PER_SAMPLE {
                apu_cycle_accum -= CYCLES_PER_SAMPLE;

                let (l, r) = cpu.ram.apu.generate_sample();
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.push_back((l * 0.8, r * 0.8));

                // Buffer de ~100ms (4410 samples = 0.1s a 44.1kHz)
                while buffer.len() > 4410 {
                    buffer.pop_front();
                }
            }

            // FRAME: conta frames emulados
            if frame_cycle_accum >= CYCLES_PER_FRAME {
                frame_cycle_accum -= CYCLES_PER_FRAME;
                frame_counter += 1;
                debug_print_timer += 1;

                if debug_print_timer >= 60 {
                    debug_print_timer = 0;
                    println!("Frames: {} | PC: {:04X} | LY: {}",
                        frame_counter, cpu.registers.get_pc(), cpu.ram.ppu.ly);
                }
            }
        }

        // ==== Render ====
        if cpu.ram.ppu.frame_ready {
            cpu.ram.ppu.frame_ready = false;

            let fb = &cpu.ram.ppu.framebuffer;
            texture
                .with_lock(None, |buf: &mut [u8], _pitch| {
                    for y in 0..144 {
                        for x in 0..160 {
                            let idx = y * 160 + x;
                            let color = fb[idx];
                            let shade = match color {
                                0 => 0xFF,
                                1 => 0xAA,
                                2 => 0x55,
                                _ => 0x00,
                            };
                            let off = idx * 3;
                            buf[off] = shade;
                            buf[off + 1] = shade;
                            buf[off + 2] = shade;
                        }
                    }
                })
                .unwrap();

            canvas.clear();
            canvas
                .copy(
                    &texture,
                    None,
                    Some(Rect::new(0, 0, 160 * scale, 144 * scale).into()),
                )
                .unwrap();
            canvas.present();
        }

        // Pequena folga pra não travar 100% CPU
        std::thread::sleep(Duration::from_micros(50));
    }

    println!("Encerrando SDL3 após {} frames", frame_counter);
}

/// Gera o nome do arquivo .sav baseado no nome da ROM
fn get_sav_path(rom_path: &str) -> String {
    let path = std::path::Path::new(rom_path);
    match path.with_extension("sav").to_str() {
        Some(sav_path) => sav_path.to_string(),
        None => format!("{}.sav", rom_path), // fallback
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: cargo run -- <rom.gb> [--trace]");
        return;
    }
    let rom_path = &args[1];
    let sav_path = get_sav_path(rom_path);

    let data = fs::read(rom_path).expect("Falha ao ler ROM");
    let mut cpu = GB::CPU::CPU::new();
    cpu.load_rom(&data);
    cpu.init_post_boot();

    // Carrega save se existir
    if let Err(e) = cpu.ram.load_cart_ram(&sav_path) {
        eprintln!("⚠️ Erro ao carregar save: {}", e);
    }

    println!("ROM carregada: {} ({} bytes)", rom_path, data.len());

    if args.iter().any(|a| a == "--trace") {
        run_trace(&mut cpu);
    } else {
        run_sdl(&mut cpu);
    }

    // Salva RAM ao sair
    if let Err(e) = cpu.ram.save_cart_ram(&sav_path) {
        eprintln!("⚠️ Erro ao salvar: {}", e);
    }
}
