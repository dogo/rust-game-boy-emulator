//! Módulo para execução com interface gráfica SDL3
//! Arquitetura: Emulação em thread separada + Render com VSync no main thread

use crate::GB::CPU::CPU;
use crate::GB::debugger::{DebugCommand, DebugResponse, Debugger};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sdl3::audio::{AudioCallback, AudioSpec, AudioStream};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::rect::Rect;

// Constantes do Game Boy
const GB_WIDTH: usize = 160;
const GB_HEIGHT: usize = 144;
const GB_CPU_HZ: u64 = 4_194_304;
const GB_FPS: f64 = 59.7275;
const CYCLES_PER_FRAME: u64 = (GB_CPU_HZ as f64 / GB_FPS) as u64;
const SAMPLE_RATE: u32 = 44_100;

// =============================================================================
// TRIPLE BUFFER
// =============================================================================

struct TripleBuffer {
    buffers: [Mutex<Vec<u8>>; 3],
    write_idx: AtomicU8,
    ready_idx: AtomicU8,
    read_idx: AtomicU8,
    new_frame_available: AtomicBool,
}

impl TripleBuffer {
    fn new() -> Self {
        Self {
            buffers: [
                Mutex::new(vec![0u8; GB_WIDTH * GB_HEIGHT]),
                Mutex::new(vec![0u8; GB_WIDTH * GB_HEIGHT]),
                Mutex::new(vec![0u8; GB_WIDTH * GB_HEIGHT]),
            ],
            write_idx: AtomicU8::new(0),
            ready_idx: AtomicU8::new(1),
            read_idx: AtomicU8::new(2),
            new_frame_available: AtomicBool::new(false),
        }
    }

    fn submit_frame(&self, framebuffer: &[u8; GB_WIDTH * GB_HEIGHT]) {
        let write_idx = self.write_idx.load(Ordering::Acquire) as usize;
        {
            let mut buf = self.buffers[write_idx].lock().unwrap();
            buf.copy_from_slice(framebuffer);
        }
        let old_ready = self.ready_idx.swap(write_idx as u8, Ordering::AcqRel);
        self.write_idx.store(old_ready, Ordering::Release);
        self.new_frame_available.store(true, Ordering::Release);
    }

    fn get_frame(&self) -> Option<Vec<u8>> {
        if !self.new_frame_available.swap(false, Ordering::AcqRel) {
            return None;
        }
        let ready_idx = self.ready_idx.load(Ordering::Acquire);
        let old_read = self.read_idx.swap(ready_idx, Ordering::AcqRel);
        self.ready_idx.store(old_read, Ordering::Release);
        let read_idx = self.read_idx.load(Ordering::Acquire) as usize;
        let buf = self.buffers[read_idx].lock().unwrap();
        Some(buf.clone())
    }
}

// =============================================================================
// ESTADO COMPARTILHADO ENTRE THREADS
// =============================================================================

struct SharedState {
    frame_buffer: TripleBuffer,
    audio_buffer: Mutex<VecDeque<(f32, f32)>>,
    running: AtomicBool,
    paused: AtomicBool,
    debug_requested: AtomicBool,
    joypad_pressed: AtomicU8,
    joypad_released: AtomicU8,
    emu_fps: Mutex<f64>,
    audio_buffer_size: Mutex<usize>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            frame_buffer: TripleBuffer::new(),
            audio_buffer: Mutex::new(VecDeque::with_capacity(SAMPLE_RATE as usize)),
            running: AtomicBool::new(true),
            paused: AtomicBool::new(false),
            debug_requested: AtomicBool::new(false),
            joypad_pressed: AtomicU8::new(0),
            joypad_released: AtomicU8::new(0),
            emu_fps: Mutex::new(0.0),
            audio_buffer_size: Mutex::new(0),
        }
    }
}

// =============================================================================
// AUDIO CALLBACK
// =============================================================================

struct AudioCallbackData {
    state: Arc<SharedState>,
}

impl AudioCallback<f32> for AudioCallbackData {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        if self.state.paused.load(Ordering::Relaxed) {
            let silence = vec![0.0f32; (requested * 2) as usize];
            let _ = stream.put_data_f32(&silence);
            return;
        }

        let mut audio_buffer = self.state.audio_buffer.lock().unwrap();
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

        *self.state.audio_buffer_size.lock().unwrap() = audio_buffer.len();
        let _ = stream.put_data_f32(&out);
    }
}

// =============================================================================
// THREAD DE EMULAÇÃO
// =============================================================================

fn emulation_thread(
    cpu: &mut CPU,
    state: Arc<SharedState>,
    cmd_rx: Receiver<DebugCommand>,
    resp_tx: Sender<DebugResponse>,
) {
    let cycles_per_sample = GB_CPU_HZ as f64 / SAMPLE_RATE as f64;
    let target_frame_time = Duration::from_secs_f64(1.0 / GB_FPS);

    let mut apu_cycle_accum: f64 = 0.0;
    let mut frame_cycle_accum: u64 = 0;
    let mut frame_count: u64 = 0;
    let mut fps_timer = Instant::now();
    let mut fps_frame_count: u64 = 0;

    // Debugger com breakpoints
    let mut debugger = Debugger::new();

    // Pré-buffer de áudio (~80ms)
    {
        let mut buf = state.audio_buffer.lock().unwrap();
        let prefill = (SAMPLE_RATE as usize * 80) / 1000;
        for _ in 0..prefill {
            buf.push_back((0.0, 0.0));
        }
    }

    while state.running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        // Checa se debug foi solicitado
        if state.debug_requested.load(Ordering::Relaxed) {
            state.paused.store(true, Ordering::Relaxed);

            // Usa o loop de comandos do debugger
            let should_quit = debugger.debug_command_loop(cpu, &cmd_rx, &resp_tx);

            if should_quit {
                state.running.store(false, Ordering::Relaxed);
            }

            state.debug_requested.store(false, Ordering::Relaxed);
            state.paused.store(false, Ordering::Relaxed);

            // Limpa e preenche buffer de áudio após debug
            {
                let mut buf = state.audio_buffer.lock().unwrap();
                buf.clear();
                for _ in 0..2048 {
                    buf.push_back((0.0, 0.0));
                }
            }
            continue;
        }

        // Processa input do joypad
        process_joypad_input(cpu, &state);

        // Roda um frame completo de emulação
        while frame_cycle_accum < CYCLES_PER_FRAME {
            // Checa breakpoints
            if debugger.check_breakpoint(cpu.registers.get_pc()) {
                println!("🔴 Breakpoint hit at 0x{:04X}", cpu.registers.get_pc());
                state.debug_requested.store(true, Ordering::Relaxed);
                break;
            }

            let (cycles, _) = cpu.execute_next();
            let c = cycles as u64;

            frame_cycle_accum += c;
            apu_cycle_accum += c as f64;

            while apu_cycle_accum >= cycles_per_sample {
                apu_cycle_accum -= cycles_per_sample;
                let (l, r) = cpu.bus.apu.generate_sample();
                let mut buffer = state.audio_buffer.lock().unwrap();
                buffer.push_back((l * 0.8, r * 0.8));
                while buffer.len() > (SAMPLE_RATE as usize * 200) / 1000 {
                    buffer.pop_front();
                }
            }
        }

        frame_cycle_accum -= CYCLES_PER_FRAME;
        frame_count += 1;
        fps_frame_count += 1;

        if cpu.bus.ppu.frame_ready {
            cpu.bus.ppu.frame_ready = false;
            state.frame_buffer.submit_frame(&cpu.bus.ppu.framebuffer);
        }

        if fps_timer.elapsed() >= Duration::from_secs(1) {
            let fps = fps_frame_count as f64 / fps_timer.elapsed().as_secs_f64();
            *state.emu_fps.lock().unwrap() = fps;
            fps_frame_count = 0;
            fps_timer = Instant::now();
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            let sleep_time = target_frame_time - elapsed;
            if sleep_time > Duration::from_micros(1500) {
                thread::sleep(sleep_time - Duration::from_micros(1000));
            }
            while frame_start.elapsed() < target_frame_time {
                std::hint::spin_loop();
            }
        }
    }

    println!("🛑 Emulation thread finalizada após {} frames", frame_count);
}

fn process_joypad_input(cpu: &mut CPU, state: &Arc<SharedState>) {
    let pressed = state.joypad_pressed.swap(0, Ordering::AcqRel);
    if pressed != 0 {
        if pressed & 0x01 != 0 {
            cpu.bus.joypad.press("RIGHT");
        }
        if pressed & 0x02 != 0 {
            cpu.bus.joypad.press("LEFT");
        }
        if pressed & 0x04 != 0 {
            cpu.bus.joypad.press("UP");
        }
        if pressed & 0x08 != 0 {
            cpu.bus.joypad.press("DOWN");
        }
        if pressed & 0x10 != 0 {
            cpu.bus.joypad.press("A");
        }
        if pressed & 0x20 != 0 {
            cpu.bus.joypad.press("B");
        }
        if pressed & 0x40 != 0 {
            cpu.bus.joypad.press("SELECT");
        }
        if pressed & 0x80 != 0 {
            cpu.bus.joypad.press("START");
        }
        if cpu.bus.joypad.take_interrupt_request() {
            cpu.bus.request_joypad_interrupt();
        }
    }

    let released = state.joypad_released.swap(0, Ordering::AcqRel);
    if released != 0 {
        if released & 0x01 != 0 {
            cpu.bus.joypad.release("RIGHT");
        }
        if released & 0x02 != 0 {
            cpu.bus.joypad.release("LEFT");
        }
        if released & 0x04 != 0 {
            cpu.bus.joypad.release("UP");
        }
        if released & 0x08 != 0 {
            cpu.bus.joypad.release("DOWN");
        }
        if released & 0x10 != 0 {
            cpu.bus.joypad.release("A");
        }
        if released & 0x20 != 0 {
            cpu.bus.joypad.release("B");
        }
        if released & 0x40 != 0 {
            cpu.bus.joypad.release("SELECT");
        }
        if released & 0x80 != 0 {
            cpu.bus.joypad.release("START");
        }
    }
}

// =============================================================================
// FUNÇÕES SDL (MAIN THREAD)
// =============================================================================

fn init_sdl() -> Result<sdl3::Sdl, String> {
    sdl3::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
    sdl3::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "nearest");
    sdl3::hint::set("SDL_HINT_RENDER_VSYNC", "1");
    sdl3::init().map_err(|e| format!("{:?}", e))
}

fn setup_audio(
    sdl_ctx: &sdl3::Sdl,
    state: Arc<SharedState>,
) -> sdl3::audio::AudioStreamWithCallback<AudioCallbackData> {
    let audio_subsystem = sdl_ctx.audio().expect("Falha subsistema de áudio");
    let desired_spec = AudioSpec {
        freq: Some(44100),
        channels: Some(2),
        format: Some(sdl3::audio::AudioFormat::f32_sys()),
    };

    let audio_device = audio_subsystem
        .open_playback_stream(&desired_spec, AudioCallbackData { state })
        .expect("Falha ao abrir dispositivo de áudio");

    audio_device.resume().expect("Falha ao iniciar áudio");
    audio_device
}

fn keycode_to_button(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Right => Some(0x01),
        Keycode::Left => Some(0x02),
        Keycode::Up => Some(0x04),
        Keycode::Down => Some(0x08),
        Keycode::Z => Some(0x10),
        Keycode::X => Some(0x20),
        Keycode::Backspace => Some(0x40),
        Keycode::Return => Some(0x80),
        _ => None,
    }
}

enum InputResult {
    Continue,
    Quit,
    Debug,
}

fn handle_input(state: &Arc<SharedState>, event: &Event) -> InputResult {
    match event {
        Event::Quit { .. } => InputResult::Quit,
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => InputResult::Quit,
        Event::KeyDown {
            keycode: Some(Keycode::F12),
            repeat: false,
            ..
        } => InputResult::Debug,
        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
            ..
        } => {
            if let Some(button) = keycode_to_button(*k) {
                state.joypad_pressed.fetch_or(button, Ordering::Release);
            }
            InputResult::Continue
        }
        Event::KeyUp {
            keycode: Some(k),
            repeat: false,
            ..
        } => {
            if let Some(button) = keycode_to_button(*k) {
                state.joypad_released.fetch_or(button, Ordering::Release);
            }
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

// =============================================================================
// ENTRY POINT
// =============================================================================

pub fn run(cpu: &mut CPU) {
    println!("🎮 Iniciando modo gráfico SDL3 (threaded)");
    println!("   ESC = sair | F12 = debugger");

    let sdl_ctx = init_sdl().expect("Falha ao inicializar SDL3");
    let video = sdl_ctx.video().expect("Falha subsistema de vídeo");

    let state = Arc::new(SharedState::new());
    let _audio_device = setup_audio(&sdl_ctx, state.clone());

    // Canais para debug
    let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>();
    let (resp_tx, resp_rx) = mpsc::channel::<DebugResponse>();

    let scale = 3u32;
    let window = video
        .window("GB Emulator", 160 * scale, 144 * scale)
        .position_centered()
        .build()
        .expect("Falha ao criar janela");
    let mut canvas = window.into_canvas();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl3::pixels::PixelFormat::RGB24, 160, 144)
        .expect("Falha texture");

    let mut event_pump = sdl_ctx.event_pump().expect("Falha event pump");

    thread::scope(|scope| {
        let state_clone = state.clone();
        let _emu_handle = scope.spawn(move || {
            emulation_thread(cpu, state_clone, cmd_rx, resp_tx);
        });

        let mut render_frame_count: u64 = 0;
        let mut stats_timer = Instant::now();

        loop {
            let events: Vec<_> = event_pump.poll_iter().collect();

            for event in events {
                match handle_input(&state, &event) {
                    InputResult::Quit => {
                        state.running.store(false, Ordering::Relaxed);
                        println!(
                            "👋 Encerrando após {} frames renderizados",
                            render_frame_count
                        );
                        return;
                    }
                    InputResult::Debug => {
                        state.debug_requested.store(true, Ordering::Relaxed);

                        // Espera emulação pausar
                        while !state.paused.load(Ordering::Relaxed)
                            && state.running.load(Ordering::Relaxed)
                        {
                            thread::sleep(Duration::from_millis(1));
                        }

                        // Usa o loop de input do debugger
                        if Debugger::terminal_input_loop(&cmd_tx, &resp_rx) {
                            state.running.store(false, Ordering::Relaxed);
                            println!("👋 Saindo via debugger");
                            return;
                        }
                    }
                    InputResult::Continue => {}
                }
            }

            if let Some(framebuffer) = state.frame_buffer.get_frame() {
                texture
                    .with_lock(None, |buf: &mut [u8], _pitch| {
                        for i in 0..(144 * 160) {
                            let shade = match framebuffer[i] {
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

                render_frame_count += 1;
            }

            canvas.clear();
            canvas
                .copy(
                    &texture,
                    None,
                    Some(Rect::new(0, 0, 160 * scale, 144 * scale).into()),
                )
                .unwrap();
            canvas.present();

            if stats_timer.elapsed() >= Duration::from_secs(2) {
                let emu_fps = *state.emu_fps.lock().unwrap();
                let audio_buf = *state.audio_buffer_size.lock().unwrap();
                let audio_ms = (audio_buf as f64 / SAMPLE_RATE as f64) * 1000.0;

                println!(
                    "📊 Emu: {:.1} FPS | Render: {} frames | Audio buffer: {:.0}ms",
                    emu_fps, render_frame_count, audio_ms
                );

                stats_timer = Instant::now();
            }
        }
    });
}
