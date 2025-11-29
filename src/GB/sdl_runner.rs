//! Módulo para execução com interface gráfica SDL3

use crate::GB::CPU::CPU;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sdl3::audio::{AudioCallback, AudioSpec, AudioStream};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;

// Constantes do Game Boy
const GB_CPU_HZ: u64 = 4_194_304;
const GB_FPS: f64 = 59.7275;
const CYCLES_PER_FRAME: u64 = (GB_CPU_HZ as f64 / GB_FPS) as u64;
const SAMPLE_RATE: u32 = 44_100;

struct AudioCallbackData {
    buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,
}

impl AudioCallback<f32> for AudioCallbackData {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let mut audio_buffer = self.buffer.lock().unwrap();
        let mut out = Vec::<f32>::with_capacity((requested * 2) as usize);
        let mut underflow_count = 0;

        for _ in 0..requested {
            if let Some((l, r)) = audio_buffer.pop_front() {
                out.push(l.clamp(-1.0, 1.0));
                out.push(r.clamp(-1.0, 1.0));
            } else {
                out.push(0.0);
                out.push(0.0);
                underflow_count += 1;
            }
        }

        if underflow_count > 0 {
            static mut UNDERFLOW_COUNT: u32 = 0;
            static mut STARTUP_GRACE: u32 = 0;
            unsafe {
                STARTUP_GRACE += 1;
                if STARTUP_GRACE > 220 {
                    UNDERFLOW_COUNT += 1;
                    if UNDERFLOW_COUNT % 5 == 1 {
                        println!(
                            "⚠️  Audio underflow: {} samples (buffer: {}, req: {})",
                            underflow_count,
                            audio_buffer.len(),
                            requested
                        );
                    }
                }
            }
        }

        let _ = stream.put_data_f32(&out);
    }
}

fn init_sdl() -> Result<sdl3::Sdl, String> {
    sdl3::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
    sdl3::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "nearest");
    sdl3::init().map_err(|e| format!("{:?}", e))
}

fn setup_audio(
    sdl_ctx: &sdl3::Sdl,
) -> (
    sdl3::audio::AudioStreamWithCallback<AudioCallbackData>,
    Arc<Mutex<VecDeque<(f32, f32)>>>,
) {
    let audio_subsystem = sdl_ctx.audio().expect("Falha subsistema de áudio");
    let desired_spec = AudioSpec {
        freq: Some(44100),
        channels: Some(2),
        format: Some(sdl3::audio::AudioFormat::f32_sys()),
    };

    let audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>> = Arc::new(Mutex::new(VecDeque::new()));

    // Pré-buffer de silêncio
    {
        let mut buf = audio_buffer.lock().unwrap();
        let prefill_samples = (SAMPLE_RATE as usize * 120) / 1000;
        for _ in 0..prefill_samples {
            buf.push_back((0.0, 0.0));
        }
        println!(
            "🔇 Pré-buffer de áudio: {} samples (~120ms)",
            prefill_samples
        );
    }

    let audio_device = audio_subsystem
        .open_playback_stream(
            &desired_spec,
            AudioCallbackData {
                buffer: audio_buffer.clone(),
            },
        )
        .expect("Falha ao abrir dispositivo de áudio");

    audio_device.resume().expect("Falha ao iniciar áudio");

    (audio_device, audio_buffer)
}

fn handle_input(cpu: &mut CPU, event: &Event) -> bool {
    match event {
        Event::Quit { .. } => return true,
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => return true,
        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
            ..
        } => {
            let button = match k {
                Keycode::Right => Some("RIGHT"),
                Keycode::Left => Some("LEFT"),
                Keycode::Up => Some("UP"),
                Keycode::Down => Some("DOWN"),
                Keycode::Z => Some("A"),
                Keycode::X => Some("B"),
                Keycode::Return => Some("START"),
                Keycode::Backspace => Some("SELECT"),
                _ => None,
            };
            if let Some(b) = button {
                cpu.bus.joypad.press(b);
                if cpu.bus.joypad.take_interrupt_request() {
                    cpu.bus.request_joypad_interrupt();
                }
            }
        }
        Event::KeyUp {
            keycode: Some(k),
            repeat: false,
            ..
        } => {
            let button = match k {
                Keycode::Right => Some("RIGHT"),
                Keycode::Left => Some("LEFT"),
                Keycode::Up => Some("UP"),
                Keycode::Down => Some("DOWN"),
                Keycode::Z => Some("A"),
                Keycode::X => Some("B"),
                Keycode::Return => Some("START"),
                Keycode::Backspace => Some("SELECT"),
                _ => None,
            };
            if let Some(b) = button {
                cpu.bus.joypad.release(b);
            }
        }
        _ => {}
    }
    false
}

/// Executa o emulador com interface gráfica SDL3
pub fn run(cpu: &mut CPU) {
    println!("Iniciando modo gráfico SDL3 (ESC para sair)");

    let sdl_ctx = init_sdl().expect("Falha ao inicializar SDL3");
    let video = sdl_ctx.video().expect("Falha subsistema de vídeo");
    let (_audio_device, audio_buffer) = setup_audio(&sdl_ctx);

    // Vídeo
    let scale = 3u32;
    let window = video
        .window("GB Emulator", 160 * scale, 144 * scale)
        .position_centered()
        .build()
        .expect("Falha ao criar janela");
    let mut canvas = window.into_canvas();

    // VSync
    unsafe {
        let renderer = canvas.raw();
        unsafe extern "C" {
            fn SDL_SetRenderVSync(renderer: *mut std::ffi::c_void, vsync: std::ffi::c_int) -> bool;
        }
        if SDL_SetRenderVSync(renderer as *mut std::ffi::c_void, 1) {
            println!("✅ VSync habilitado");
        } else {
            println!("❌ Falha ao habilitar VSync");
        }
    }

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl3::pixels::PixelFormat::RGB24, 160, 144)
        .expect("Falha texture");

    let mut event_pump = sdl_ctx.event_pump().expect("Falha event pump");

    // Estado do loop
    let cycles_per_sample = GB_CPU_HZ as f64 / SAMPLE_RATE as f64;
    let mut apu_cycle_accum: f64 = 0.0;
    let mut frame_cycle_accum: u64 = 0;
    let mut frame_counter: u64 = 0;
    let mut debug_print_timer = 0;
    let mut samples_produced: u64 = 0;
    let mut pending_cycles: f64 = 0.0;
    let mut last_time = Instant::now();

    loop {
        // Eventos
        for event in event_pump.poll_iter() {
            if handle_input(cpu, &event) {
                println!("Encerrando SDL3 após {} frames", frame_counter);
                return;
            }
        }

        // Timing
        let now = Instant::now();
        let dt = now.duration_since(last_time);
        last_time = now;
        pending_cycles += dt.as_secs_f64() * (GB_CPU_HZ as f64);
        pending_cycles = pending_cycles.min((GB_CPU_HZ as f64) * 0.25);

        // CPU/APU
        while pending_cycles >= 1.0 {
            let (instruction_cycles, _) = cpu.execute_next();
            let c = instruction_cycles as u64;

            pending_cycles -= c as f64;
            frame_cycle_accum += c;
            apu_cycle_accum += c as f64;

            while apu_cycle_accum >= cycles_per_sample {
                apu_cycle_accum -= cycles_per_sample;
                samples_produced += 1;
                let (l, r) = cpu.bus.apu.generate_sample();
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.push_back((l * 0.8, r * 0.8));
                while buffer.len() > 44100 {
                    buffer.pop_front();
                }
            }

            if frame_cycle_accum >= CYCLES_PER_FRAME {
                frame_cycle_accum -= CYCLES_PER_FRAME;
                frame_counter += 1;
                debug_print_timer += 1;

                if debug_print_timer >= 60 {
                    debug_print_timer = 0;
                    let buffer_size = audio_buffer.lock().unwrap().len();
                    println!(
                        "Frames: {} | PC: {:04X} | LY: {} | Audio: {}samples | Prod: {:.0}Hz",
                        frame_counter,
                        cpu.registers.get_pc(),
                        cpu.bus.ppu.ly,
                        buffer_size,
                        samples_produced as f32
                    );
                    samples_produced = 0;
                }
            }
        }

        // Render
        if cpu.bus.ppu.frame_ready {
            cpu.bus.ppu.frame_ready = false;
            let fb = &cpu.bus.ppu.framebuffer;
            texture
                .with_lock(None, |buf: &mut [u8], _pitch| {
                    for i in 0..(144 * 160) {
                        let shade = match fb[i] {
                            0 => 0xFF,
                            1 => 0xAA,
                            2 => 0x55,
                            _ => 0x00,
                        };
                        let off = i * 3;
                        buf[off] = shade;
                        buf[off + 1] = shade;
                        buf[off + 2] = shade;
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

        std::thread::sleep(Duration::from_micros(50));
    }
}
