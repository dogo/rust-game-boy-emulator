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
// FRAME MAIS RECENTE
// =============================================================================

struct LatestFrame {
    frame: Mutex<(Vec<u8>, bool)>,
}

impl LatestFrame {
    fn new() -> Self {
        Self {
            frame: Mutex::new((vec![0u8; GB_WIDTH * GB_HEIGHT], false)),
        }
    }

    fn submit_frame(&self, framebuffer: &[u8; GB_WIDTH * GB_HEIGHT]) {
        let mut frame = self.frame.lock().unwrap();
        frame.0.copy_from_slice(framebuffer);
        frame.1 = true;
    }

    fn get_frame(&self) -> Option<Vec<u8>> {
        let mut frame = self.frame.lock().unwrap();
        if !std::mem::replace(&mut frame.1, false) {
            return None;
        }
        Some(frame.0.clone())
    }
}

// =============================================================================
// ESTADO COMPARTILHADO ENTRE THREADS
// =============================================================================

struct SharedState {
    frame_buffer: LatestFrame,
    audio_buffer: Mutex<VecDeque<(f32, f32)>>,
    running: AtomicBool,
    paused: AtomicBool,
    debug_requested: AtomicBool,
    joypad_held: AtomicU8,
    joypad_pressed: AtomicU8,
    emu_fps: Mutex<f64>,
    audio_buffer_size: Mutex<usize>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            frame_buffer: LatestFrame::new(),
            audio_buffer: Mutex::new(VecDeque::with_capacity(SAMPLE_RATE as usize)),
            running: AtomicBool::new(true),
            paused: AtomicBool::new(false),
            debug_requested: AtomicBool::new(false),
            joypad_held: AtomicU8::new(0),
            joypad_pressed: AtomicU8::new(0),
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
    let mut skip_breakpoint = false;

    prefill_audio(&state);

    while state.running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        // Checa se debug foi solicitado
        if state.debug_requested.load(Ordering::Acquire) {
            state.paused.store(true, Ordering::Release);

            // Usa o loop de comandos do debugger
            let should_quit = debugger.debug_command_loop(cpu, &cmd_rx, &resp_tx, &state.running);

            if should_quit {
                state.running.store(false, Ordering::Relaxed);
            }

            skip_breakpoint = true;
            // Nessa ordem: a interface vê paused=false e não pode mais ver debug_requested
            state.debug_requested.store(false, Ordering::Release);
            state.paused.store(false, Ordering::Release);

            // Com menos folga que no início o buffer não se recupera e estala
            prefill_audio(&state);
            continue;
        }

        // Processa input do joypad
        process_joypad_input(cpu, &state);

        // Roda um frame completo de emulação
        let mut breakpoint_hit = false;
        while frame_cycle_accum < CYCLES_PER_FRAME {
            // Checa breakpoints
            if !skip_breakpoint && debugger.check_breakpoint(cpu.registers.get_pc()) {
                println!("🔴 Breakpoint hit at 0x{:04X}", cpu.registers.get_pc());
                state.debug_requested.store(true, Ordering::Release);
                breakpoint_hit = true;
                break;
            }
            skip_breakpoint = false;

            let cycles = cpu.execute_next();
            frame_cycle_accum += cycles;
            apu_cycle_accum += cycles as f64;

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

        if breakpoint_hit {
            // Frame parcial: os ciclos acumulados continuam após a retomada
            continue;
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

fn prefill_audio(state: &SharedState) {
    let mut buf = state.audio_buffer.lock().unwrap();
    buf.clear();
    buf.resize((SAMPLE_RATE as usize * 80) / 1000, (0.0, 0.0));
}

/// Mesma ordem de bits de `Joypad::raw_state` (D-pad no nibble baixo)
const BUTTONS: [&str; 8] = ["RIGHT", "LEFT", "UP", "DOWN", "A", "B", "SELECT", "START"];

fn key_down(state: &SharedState, button: u8) {
    state.joypad_held.fetch_or(button, Ordering::Release);
    state.joypad_pressed.fetch_or(button, Ordering::Release);
}

fn key_up(state: &SharedState, button: u8) {
    state.joypad_held.fetch_and(!button, Ordering::Release);
}

fn process_joypad_input(cpu: &mut CPU, state: &Arc<SharedState>) {
    let down =
        state.joypad_held.load(Ordering::Acquire) | state.joypad_pressed.swap(0, Ordering::AcqRel);
    let current = !cpu.bus.joypad.raw_state();

    for (i, name) in BUTTONS.iter().enumerate() {
        let bit = 1 << i;
        if (down ^ current) & bit != 0 {
            if down & bit != 0 {
                cpu.bus.joypad.press(name);
            } else {
                cpu.bus.joypad.release(name);
            }
        }
    }
    if cpu.bus.joypad.take_interrupt_request() {
        cpu.bus.request_joypad_interrupt();
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
                key_down(state, button);
            }
            InputResult::Continue
        }
        Event::KeyUp {
            keycode: Some(k),
            repeat: false,
            ..
        } => {
            if let Some(button) = keycode_to_button(*k) {
                key_up(state, button);
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
    let mut window = video
        .window("GB Emulator", 160 * scale, 144 * scale)
        .position_centered()
        .build()
        .expect("Falha ao criar janela");

    // Define o ícone da janela
    if let Ok(icon) = sdl3::surface::Surface::load_bmp("icon.bmp") {
        window.set_icon(&icon);
    }

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
                    InputResult::Debug => state.debug_requested.store(true, Ordering::Release),
                    InputResult::Continue => {}
                }
            }

            if state.debug_requested.load(Ordering::Acquire) {
                // Espera emulação pausar
                while !state.paused.load(Ordering::Acquire) && state.running.load(Ordering::Relaxed)
                {
                    thread::sleep(Duration::from_millis(1));
                }

                // Usa o loop de input do debugger
                if Debugger::terminal_input_loop(&cmd_tx, &resp_rx) {
                    state.running.store(false, Ordering::Relaxed);
                    println!("👋 Saindo via debugger");
                    return;
                }

                // Espera a retomada para não reabrir o debugger com a pausa anterior
                while state.paused.load(Ordering::Acquire) && state.running.load(Ordering::Relaxed)
                {
                    thread::sleep(Duration::from_millis(1));
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

#[cfg(test)]
mod known_regressions {
    use super::*;

    #[test]
    fn automatic_breakpoint_does_not_underflow_frame_count() {
        let mut cpu = CPU::new(vec![0; 32768]);
        let state = Arc::new(SharedState::new());
        state.debug_requested.store(true, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        let (resp_tx, _resp_rx) = mpsc::channel();
        tx.send(DebugCommand::AddBreakpoint(0)).unwrap();
        tx.send(DebugCommand::Continue).unwrap();

        let stop_state = state.clone();
        let stop = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            stop_state.running.store(false, Ordering::Relaxed);
            let _ = tx.send(DebugCommand::Quit);
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            emulation_thread(&mut cpu, state, rx, resp_tx);
        }));
        stop.join().unwrap();
        assert!(result.is_ok());
    }

    // NOP em 0x0000 seguido de JR -2 em 0x0001 (laço infinito)
    fn loop_rom() -> Vec<u8> {
        let mut rom = vec![0; 32768];
        rom[1] = 0x18;
        rom[2] = 0xFE;
        rom
    }

    fn wait_until(cond: impl Fn() -> bool) -> bool {
        let start = Instant::now();
        while !cond() {
            if start.elapsed() > Duration::from_secs(5) {
                return false;
            }
            thread::sleep(Duration::from_millis(1));
        }
        true
    }

    // Encerra a emulação se uma asserção falhar, evitando que o join trave o teste
    struct StopOnDrop(Arc<SharedState>);
    impl Drop for StopOnDrop {
        fn drop(&mut self) {
            self.0.running.store(false, Ordering::Relaxed);
        }
    }

    fn expect_text(rx: &Receiver<DebugResponse>, needle: &str) {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(DebugResponse::Text(text)) => assert!(text.contains(needle), "{text}"),
            other => panic!("resposta inesperada: {other:?}"),
        }
    }

    #[test]
    fn breakpoint_on_first_opcode_resumes_past_it() {
        let mut cpu = CPU::new(loop_rom());
        let state = Arc::new(SharedState::new());
        let (tx, rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();
        state.debug_requested.store(true, Ordering::Relaxed);
        tx.send(DebugCommand::AddBreakpoint(0x0000)).unwrap();
        tx.send(DebugCommand::Continue).unwrap();

        thread::scope(|scope| {
            let emu_state = state.clone();
            let stop = StopOnDrop(state.clone());
            let emu = scope.spawn(|| emulation_thread(&mut cpu, emu_state, rx, resp_tx));
            expect_text(&resp_rx, "0x0000");
            assert!(matches!(resp_rx.recv().unwrap(), DebugResponse::Resume));

            assert!(wait_until(|| !state.paused.load(Ordering::Acquire)));
            thread::sleep(Duration::from_millis(50));
            assert!(!state.paused.load(Ordering::Acquire));
            drop(stop);
            emu.join().unwrap();
        });
        assert_eq!(cpu.registers.get_pc(), 0x0001);
    }

    #[test]
    fn mid_frame_breakpoint_step_continue_and_quit() {
        let mut cpu = CPU::new(loop_rom());
        let state = Arc::new(SharedState::new());
        let (tx, rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();
        state.debug_requested.store(true, Ordering::Relaxed);
        tx.send(DebugCommand::AddBreakpoint(0x0001)).unwrap();
        tx.send(DebugCommand::Continue).unwrap();

        thread::scope(|scope| {
            let emu_state = state.clone();
            let _stop = StopOnDrop(state.clone());
            let emu = scope.spawn(|| emulation_thread(&mut cpu, emu_state, rx, resp_tx));
            expect_text(&resp_rx, "0x0001");
            assert!(matches!(resp_rx.recv().unwrap(), DebugResponse::Resume));
            assert!(wait_until(|| state.paused.load(Ordering::Acquire)));

            tx.send(DebugCommand::Step).unwrap();
            expect_text(&resp_rx, "0001");
            tx.send(DebugCommand::Continue).unwrap();
            assert!(matches!(resp_rx.recv().unwrap(), DebugResponse::Resume));
            // Só há resposta se o laço voltou ao breakpoint e reabriu o debugger
            tx.send(DebugCommand::ShowRegisters).unwrap();
            expect_text(&resp_rx, "PC: 0001");
            assert!(state.paused.load(Ordering::Acquire));

            tx.send(DebugCommand::Quit).unwrap();
            assert!(matches!(resp_rx.recv().unwrap(), DebugResponse::Quit));
            emu.join().unwrap();
        });
        assert!(!state.running.load(Ordering::Relaxed));
    }

    #[test]
    fn latest_frame_is_delivered_once_and_newest_wins() {
        let frames = LatestFrame::new();
        assert!(frames.get_frame().is_none());
        frames.submit_frame(&[1; GB_WIDTH * GB_HEIGHT]);
        frames.submit_frame(&[2; GB_WIDTH * GB_HEIGHT]);
        assert!(frames.get_frame().unwrap().iter().all(|&p| p == 2));
        assert!(frames.get_frame().is_none());
    }

    #[test]
    fn concurrent_publish_never_yields_partial_frame() {
        const FRAMES: u8 = 200;
        let frames = LatestFrame::new();
        thread::scope(|scope| {
            scope.spawn(|| {
                for k in 1..=FRAMES {
                    frames.submit_frame(&[k; GB_WIDTH * GB_HEIGHT]);
                }
            });
            let mut last = 0;
            while last < FRAMES {
                if let Some(frame) = frames.get_frame() {
                    let k = frame[0];
                    assert!(frame.iter().all(|&p| p == k), "frame parcial");
                    assert!(k > last, "frame antigo após um mais novo");
                    last = k;
                }
            }
        });
    }

    const A: u8 = 0x10;
    const RIGHT: u8 = 0x01;

    fn action_lines(cpu: &mut CPU) -> u8 {
        cpu.bus.joypad.write(0x10);
        cpu.bus.joypad.read() & 0x0F
    }

    fn dpad_lines(cpu: &mut CPU) -> u8 {
        cpu.bus.joypad.write(0x20);
        cpu.bus.joypad.read() & 0x0F
    }

    fn take_joypad_irq(cpu: &mut CPU) -> bool {
        let pending = cpu.bus.read(0xFF0F) & 0x10 != 0;
        cpu.bus.write(0xFF0F, 0);
        pending
    }

    fn joypad_test_setup() -> (CPU, Arc<SharedState>) {
        let mut cpu = CPU::new(vec![0; 32768]);
        cpu.bus.write(0xFF0F, 0);
        (cpu, Arc::new(SharedState::new()))
    }

    #[test]
    fn release_then_press_keeps_button_held() {
        let (mut cpu, state) = joypad_test_setup();
        key_down(&state, A);
        process_joypad_input(&mut cpu, &state);
        key_up(&state, A);
        key_down(&state, A);
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu) & 1, 0);
    }

    #[test]
    fn press_then_release_within_frame_is_seen_for_one_frame() {
        let (mut cpu, state) = joypad_test_setup();
        key_down(&state, A);
        key_up(&state, A);
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu) & 1, 0);
        assert!(take_joypad_irq(&mut cpu));
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu) & 1, 1);
        assert!(!take_joypad_irq(&mut cpu));
    }

    #[test]
    fn held_key_stays_pressed_and_interrupts_once() {
        let (mut cpu, state) = joypad_test_setup();
        key_down(&state, A);
        process_joypad_input(&mut cpu, &state);
        assert!(take_joypad_irq(&mut cpu));
        for _ in 0..3 {
            process_joypad_input(&mut cpu, &state);
            assert_eq!(action_lines(&mut cpu) & 1, 0);
            assert!(!take_joypad_irq(&mut cpu));
        }
        key_up(&state, A);
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu) & 1, 1);
    }

    #[test]
    fn multiple_buttons_are_tracked_independently() {
        let (mut cpu, state) = joypad_test_setup();
        key_down(&state, A);
        key_down(&state, RIGHT);
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu), 0x0E);
        assert_eq!(dpad_lines(&mut cpu), 0x0E);
        key_up(&state, RIGHT);
        process_joypad_input(&mut cpu, &state);
        assert_eq!(action_lines(&mut cpu), 0x0E);
        assert_eq!(dpad_lines(&mut cpu), 0x0F);
    }

    #[test]
    fn press_with_simultaneous_release_wakes_from_stop() {
        let (mut cpu, state) = joypad_test_setup();
        cpu.bus.write(0xFF00, 0x00);
        key_down(&state, RIGHT);
        process_joypad_input(&mut cpu, &state);
        cpu.stopped = true;
        key_up(&state, RIGHT);
        key_down(&state, A);
        process_joypad_input(&mut cpu, &state);
        cpu.execute_next();
        assert!(!cpu.stopped);
    }
}
