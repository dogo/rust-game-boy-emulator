//! Afinação dos canais medida nas amostras, como o sdl_runner as gera (44,1 kHz).

use gb_emu::GB::CPU::CPU;

const RATE: f64 = 44_100.0;

/// `secs` segundos do canal esquerdo após desligar/religar o APU e aplicar `regs`
fn samples(regs: &[(u16, u8)], secs: f64) -> Vec<f32> {
    let mut rom = vec![0; 32768];
    rom[0x100..0x102].copy_from_slice(&[0x18, 0xFE]); // jr -2: a CPU não sai da ROM
    let mut cpu = CPU::new(rom);
    cpu.init_post_boot();
    // Religar o APU zera o canal 1 que o pós-boot deixa tocando
    for &(addr, val) in [
        (0xFF26, 0x00),
        (0xFF26, 0x80),
        (0xFF24, 0x77),
        (0xFF25, 0xFF),
    ]
    .iter()
    .chain(regs)
    {
        cpu.bus.write(addr, val);
    }
    let per_sample = 4_194_304.0 / RATE;
    let (mut acc, mut out) = (0.0, Vec::new());
    while out.len() < (RATE * secs) as usize {
        acc += cpu.execute_next() as f64;
        while acc >= per_sample {
            acc -= per_sample;
            out.push(cpu.bus.apu.generate_sample().0);
        }
    }
    out
}

/// Subidas do sinal por segundo, com histerese entre 25% e 75% da amplitude
fn frequency(s: &[f32]) -> f64 {
    let (min, max) = s
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    let (lo, hi) = (min + (max - min) * 0.25, min + (max - min) * 0.75);
    let (mut low, mut rising) = (s[0] < lo, 0);
    for &v in s {
        if low && v > hi {
            rising += 1;
            low = false;
        } else if v < lo {
            low = true;
        }
    }
    rising as f64 * RATE / s.len() as f64
}

fn assert_close(measured: f64, expected: f64, tolerance: f64, what: &str) {
    let error = measured / expected - 1.0;
    assert!(
        error.abs() < tolerance,
        "{what}: esperado {expected:.2}, medido {measured:.2}"
    );
}

#[test]
fn square_channels_play_at_131072_over_2048_minus_x() {
    let x: u16 = 2000;
    let expected = 131072.0 / (2048.0 - x as f64);
    let (lo, hi) = ((x & 0xFF) as u8, 0x80 | (x >> 8) as u8);
    let ch1 = samples(
        &[(0xFF11, 0x80), (0xFF12, 0xF0), (0xFF13, lo), (0xFF14, hi)],
        1.0,
    );
    assert_close(frequency(&ch1), expected, 0.005, "canal 1");
    let ch2 = samples(
        &[(0xFF16, 0x80), (0xFF17, 0xF0), (0xFF18, lo), (0xFF19, hi)],
        1.0,
    );
    assert_close(frequency(&ch2), expected, 0.005, "canal 2");
}

#[test]
fn wave_channel_plays_at_65536_over_2048_minus_x() {
    let x: u16 = 1792;
    let mut regs = vec![(0xFF1A, 0x00)];
    regs.extend((0..16).map(|i| (0xFF30 + i, if i < 8 { 0xFF } else { 0x00 })));
    regs.extend([
        (0xFF1A, 0x80),
        (0xFF1C, 0x20),
        (0xFF1D, x as u8),
        (0xFF1E, 0x80 | (x >> 8) as u8),
    ]);
    let s = samples(&regs, 1.0);
    assert_close(
        frequency(&s),
        65536.0 / (2048.0 - x as f64),
        0.005,
        "canal 3",
    );
}

#[test]
fn noise_lfsr_ticks_at_262144_over_r_times_2_pow_s() {
    // LFSR de 7 bits (sequência de comprimento máximo): cada ciclo de 127 ticks tem
    // 64 trechos, ou seja, 32 subidas
    for (r, shift) in [(1u8, 7u8), (2, 6), (4, 5)] {
        let s = samples(
            &[
                (0xFF21, 0xF0),
                (0xFF22, (shift << 4) | 0x08 | r),
                (0xFF23, 0x80),
            ],
            1.0,
        );
        let ticks_hz = 262144.0 / (r as f64 * (1u32 << shift) as f64);
        assert_close(
            frequency(&s) * 127.0 / 32.0,
            ticks_hz,
            0.02,
            &format!("canal 4 r={r} s={shift}"),
        );
    }
}
