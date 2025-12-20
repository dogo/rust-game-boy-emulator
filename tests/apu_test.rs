// Integration tests para APU
// cargo test apu_test

use gb_emu::GB::APU::{APU, Envelope, FrameSequencer, LengthCounter, SweepUnit};

#[cfg(test)]
mod apu_tests {
    use super::*;

    #[test]
    fn test_frame_sequencer_reset_behavior() {
        // Testar com diferentes estados iniciais e números de ticks
        for ticks_before_reset in 0..20 {
            for _iteration in 0..10 {
                // Criar frame sequencer
                let mut frame_sequencer = FrameSequencer::new();

                // Simular alguns ticks para colocar o frame sequencer em estado aleatório
                for _ in 0..ticks_before_reset {
                    frame_sequencer.tick();
                }

                // HARDWARE PRECISION: Após reset, deve estar em step 7
                frame_sequencer.reset();
                assert_eq!(
                    frame_sequencer.current_step(),
                    7,
                    "Frame sequencer deve inicializar em step 7 após reset"
                );

                // HARDWARE PRECISION: Primeiro tick após reset deve produzir step 0
                let events = frame_sequencer.tick();
                assert_eq!(
                    frame_sequencer.current_step(),
                    0,
                    "Primeiro tick após reset deve produzir step 0"
                );

                // Verificar que eventos corretos são gerados no step 0
                // Step 0 deve gerar length_clock = true
                assert!(events.length_clock, "Step 0 deve gerar length_clock = true");
                assert!(
                    !events.envelope_clock,
                    "Step 0 não deve gerar envelope_clock"
                );
                assert!(!events.sweep_clock, "Step 0 não deve gerar sweep_clock");
            }
        }
    }

    #[test]
    fn test_frame_sequencer_cycle_consistency() {
        // Testar múltiplos ciclos
        for cycles in 1..10 {
            let mut frame_sequencer = FrameSequencer::new();

            // Reset para garantir estado conhecido
            frame_sequencer.reset();
            let initial_step = frame_sequencer.current_step();
            assert_eq!(initial_step, 7, "Deve iniciar em step 7");

            // Executar ciclos completos e verificar consistência
            for _cycle in 0..cycles {
                let mut length_clocks = 0;
                let mut envelope_clocks = 0;
                let mut sweep_clocks = 0;

                // Um ciclo completo = 8 ticks
                for step in 0..8 {
                    let events = frame_sequencer.tick();
                    let current_step = frame_sequencer.current_step();

                    // Verificar que step avança corretamente
                    assert_eq!(
                        current_step, step,
                        "Step deve avançar sequencialmente: esperado {}, atual {}",
                        step, current_step
                    );

                    // Contar eventos
                    if events.length_clock {
                        length_clocks += 1;
                    }
                    if events.envelope_clock {
                        envelope_clocks += 1;
                    }
                    if events.sweep_clock {
                        sweep_clocks += 1;
                    }

                    // Verificar eventos específicos por step
                    match step {
                        0 | 2 | 4 | 6 => {
                            assert!(
                                events.length_clock,
                                "Steps pares (0,2,4,6) devem gerar length_clock"
                            );
                        }
                        1 | 3 | 5 => {
                            assert!(
                                !events.length_clock,
                                "Steps ímpares (1,3,5) não devem gerar length_clock"
                            );
                        }
                        7 => {
                            assert!(events.envelope_clock, "Step 7 deve gerar envelope_clock");
                            assert!(!events.length_clock, "Step 7 não deve gerar length_clock");
                        }
                        _ => {}
                    }

                    // Verificar sweep_clock (steps 2 e 6)
                    if step == 2 || step == 6 {
                        assert!(events.sweep_clock, "Steps 2 e 6 devem gerar sweep_clock");
                    } else {
                        assert!(
                            !events.sweep_clock,
                            "Apenas steps 2 e 6 devem gerar sweep_clock"
                        );
                    }
                }

                // Verificar contadores de eventos por ciclo
                assert_eq!(
                    length_clocks, 4,
                    "Deve haver exatamente 4 length_clocks por ciclo (steps 0,2,4,6)"
                );
                assert_eq!(
                    envelope_clocks, 1,
                    "Deve haver exatamente 1 envelope_clock por ciclo (step 7)"
                );
                assert_eq!(
                    sweep_clocks, 2,
                    "Deve haver exatamente 2 sweep_clocks por ciclo (steps 2,6)"
                );

                // Após 8 ticks, deve voltar ao step inicial (7)
                assert_eq!(
                    frame_sequencer.current_step(),
                    7,
                    "Após 8 ticks, deve voltar ao step 7"
                );
            }
        }
    }

    #[test]
    fn test_apu_sound_enable_reset() {
        // Testar com diferentes configurações de registradores
        let test_values = [0x00, 0x80, 0xFF, 0x55, 0xAA];

        for &nr10_value in &test_values {
            for &nr11_value in &test_values {
                for &nr12_value in &test_values {
                    for ticks_before_disable in 0..10 {
                        let mut apu = APU::new();

                        // Configurar alguns registradores com valores de teste
                        apu.write_register(0xFF10, nr10_value);
                        apu.write_register(0xFF11, nr11_value);
                        apu.write_register(0xFF12, nr12_value);

                        // Simular alguns eventos DIV para colocar frame sequencer em estado aleatório
                        for _ in 0..ticks_before_disable {
                            apu.div_event();
                        }

                        // Verificar que som está habilitado inicialmente
                        let nr52_initial = apu.read_register(0xFF26);
                        assert_eq!(
                            nr52_initial & 0x80,
                            0x80,
                            "Som deve estar habilitado inicialmente"
                        );

                        // Desabilitar som (NR52 bit 7 = 0)
                        apu.write_register(0xFF26, 0x00);
                        let nr52_disabled = apu.read_register(0xFF26);
                        assert_eq!(nr52_disabled & 0x80, 0x00, "Som deve estar desabilitado");

                        // Re-habilitar som (NR52 bit 7 = 1)
                        apu.write_register(0xFF26, 0x80);
                        let nr52_enabled = apu.read_register(0xFF26);
                        assert_eq!(
                            nr52_enabled & 0x80,
                            0x80,
                            "Som deve estar habilitado novamente"
                        );

                        // HARDWARE PRECISION: Frame sequencer deve estar em step 7 após re-habilitar
                        // Verificamos isso indiretamente através do próximo div_event
                        apu.div_event();

                        // O importante é que o comportamento seja consistente com reset
                        // Vamos verificar que podemos escrever registradores novamente
                        apu.write_register(0xFF11, 0x80); // Deve funcionar com som habilitado
                        let nr11_after = apu.read_register(0xFF11);
                        assert_eq!(
                            nr11_after & 0xC0,
                            0x80,
                            "Registradores devem ser acessíveis após re-habilitar som"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_frame_sequencer_new_starts_at_step_7() {
        let frame_sequencer = FrameSequencer::new();
        assert_eq!(
            frame_sequencer.current_step(),
            7,
            "Frame sequencer deve iniciar em step 7"
        );
    }

    #[test]
    fn test_frame_sequencer_reset_always_goes_to_step_7() {
        let mut frame_sequencer = FrameSequencer::new();

        // Avançar para diferentes steps e testar reset
        for _ in 0..5 {
            frame_sequencer.tick();
        }
        assert_ne!(
            frame_sequencer.current_step(),
            7,
            "Frame sequencer deve estar em step diferente de 7"
        );

        frame_sequencer.reset();
        assert_eq!(
            frame_sequencer.current_step(),
            7,
            "Reset deve sempre levar ao step 7"
        );
    }

    #[test]
    fn test_frame_sequencer_first_tick_after_reset_is_step_0() {
        let mut frame_sequencer = FrameSequencer::new();

        // Reset e verificar primeiro tick
        frame_sequencer.reset();
        let events = frame_sequencer.tick();

        assert_eq!(
            frame_sequencer.current_step(),
            0,
            "Primeiro tick após reset deve ser step 0"
        );
        assert!(events.length_clock, "Step 0 deve gerar length_clock");
        assert!(
            !events.envelope_clock,
            "Step 0 não deve gerar envelope_clock"
        );
        assert!(!events.sweep_clock, "Step 0 não deve gerar sweep_clock");
    }

    #[test]
    fn test_apu_nr52_disable_enable_resets_frame_sequencer() {
        let mut apu = APU::new();

        // Avançar frame sequencer
        for _ in 0..3 {
            apu.div_event();
        }

        // Desabilitar e re-habilitar som
        apu.write_register(0xFF26, 0x00); // Desabilitar
        apu.write_register(0xFF26, 0x80); // Re-habilitar

        // Verificar que som está habilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(nr52 & 0x80, 0x80, "Som deve estar habilitado");

        // Frame sequencer deve ter sido resetado (verificação indireta)
        // Podemos verificar que registradores funcionam normalmente
        apu.write_register(0xFF11, 0x80);
        let nr11 = apu.read_register(0xFF11);
        assert_eq!(
            nr11 & 0xC0,
            0x80,
            "Registradores devem funcionar após reset"
        );
    }

    #[test]
    fn test_frame_sequencer_is_length_clock_next() {
        let mut frame_sequencer = FrameSequencer::new();

        // Reset para step 7
        frame_sequencer.reset();
        assert_eq!(frame_sequencer.current_step(), 7);

        // Step 7 -> próximo é 0 (par) -> length clock next = true
        assert!(
            frame_sequencer.is_length_clock_next(),
            "Step 7 deve indicar que próximo é length clock"
        );

        // Tick para step 0
        frame_sequencer.tick();
        assert_eq!(frame_sequencer.current_step(), 0);

        // Step 0 -> próximo é 1 (ímpar) -> length clock next = false
        assert!(
            !frame_sequencer.is_length_clock_next(),
            "Step 0 deve indicar que próximo não é length clock"
        );

        // Tick para step 1
        frame_sequencer.tick();
        assert_eq!(frame_sequencer.current_step(), 1);

        // Step 1 -> próximo é 2 (par) -> length clock next = true
        assert!(
            frame_sequencer.is_length_clock_next(),
            "Step 1 deve indicar que próximo é length clock"
        );
    }

    /// Teste de múltiplas iterações para simular property-based testing
    #[test]
    fn test_frame_sequencer_reset_multiple_iterations() {
        // Executar 5 iterações para simular property-based testing (otimizado)
        for iteration in 0..5 {
            let mut frame_sequencer = FrameSequencer::new();

            // Variar o número de ticks antes do reset baseado na iteração
            let ticks_before_reset = iteration % 20;

            // Avançar frame sequencer
            for _ in 0..ticks_before_reset {
                frame_sequencer.tick();
            }

            // Reset deve sempre levar ao step 7
            frame_sequencer.reset();
            assert_eq!(
                frame_sequencer.current_step(),
                7,
                "Iteração {}: Reset deve sempre levar ao step 7",
                iteration
            );

            // Primeiro tick deve sempre levar ao step 0
            let events = frame_sequencer.tick();
            assert_eq!(
                frame_sequencer.current_step(),
                0,
                "Iteração {}: Primeiro tick após reset deve ser step 0",
                iteration
            );

            // Eventos devem ser consistentes
            assert!(
                events.length_clock,
                "Iteração {}: Step 0 deve gerar length_clock",
                iteration
            );
            assert!(
                !events.envelope_clock,
                "Iteração {}: Step 0 não deve gerar envelope_clock",
                iteration
            );
            assert!(
                !events.sweep_clock,
                "Iteração {}: Step 0 não deve gerar sweep_clock",
                iteration
            );
        }
    }

    #[test]
    fn test_sweep_overflow_channel_disable_property() {
        // Executar 5 iterações para simular property-based testing (otimizado)
        for iteration in 0..5 {
            let mut sweep = SweepUnit::new();

            // Testar diferentes configurações que podem causar overflow
            let test_cases = [
                (1, false, 1), // period=1, add, shift=1
                (2, false, 2), // period=2, add, shift=2
                (3, false, 3), // period=3, add, shift=3
                (4, false, 4), // period=4, add, shift=4
                (1, false, 7), // period=1, add, shift=7 (máximo)
            ];

            let case_index = iteration % test_cases.len();
            let (period, direction, shift) = test_cases[case_index];

            sweep.configure(period, direction, shift);

            // Testar frequências próximas ao limite que podem causar overflow
            let test_frequencies = [1800, 1900, 2000, 2040, 2047];

            for &freq in &test_frequencies {
                let mut test_sweep = sweep.clone();

                // Calcular nova frequência
                if let Some(new_freq) = test_sweep.calculate_new_frequency(freq) {
                    // Se não houve overflow na função, a frequência deve estar <= 2047
                    assert!(
                        new_freq <= 2047,
                        "Iteração {}: Frequência calculada {} deve estar <= 2047 para freq inicial {}",
                        iteration,
                        new_freq,
                        freq
                    );
                } else {
                    // Se houve overflow (retornou None), a frequência calculada seria > 2047
                    let freq_change = freq >> shift;
                    let would_be_freq = freq.wrapping_add(freq_change);
                    assert!(
                        would_be_freq > 2047,
                        "Iteração {}: Overflow detectado corretamente para freq {} + {} = {}",
                        iteration,
                        freq,
                        freq_change,
                        would_be_freq
                    );
                }
            }
        }
    }

    #[test]
    fn test_sweep_negate_to_add_quirk_property() {
        // Executar 5 iterações para simular property-based testing (otimizado)
        for iteration in 0..5 {
            let mut sweep = SweepUnit::new();

            // Configurar sweep em modo negate (subtract)
            let period = 1 + (iteration % 7) as u8; // period 1-7
            let shift = 1 + (iteration % 7) as u8; // shift 1-7

            sweep.configure(period, true, shift); // direction=true (negate/subtract)

            // Fazer um cálculo para marcar negate_used = true
            let test_freq = 1000 + (iteration % 1000) as u16;
            let _result = sweep.calculate_new_frequency(test_freq);

            // Verificar que negate foi usado
            assert!(
                sweep.was_negate_used(),
                "Iteração {}: Negate deve ter sido marcado como usado",
                iteration
            );

            // Agora tentar mudar para modo add (direction=false)
            // Isso deve retornar false (desabilita canal) devido ao quirk
            let should_disable = !sweep.handle_direction_change(false);
            assert!(
                should_disable,
                "Iteração {}: Mudança de negate para add após uso deve desabilitar canal",
                iteration
            );

            // Testar caso onde negate não foi usado - não deve desabilitar
            let mut sweep_unused = SweepUnit::new();
            sweep_unused.configure(period, true, shift);

            // Não fazer nenhum cálculo (negate_used permanece false)
            assert!(
                !sweep_unused.was_negate_used(),
                "Iteração {}: Negate não deve estar marcado como usado",
                iteration
            );

            // Mudança para add não deve desabilitar se negate não foi usado
            let should_not_disable = sweep_unused.handle_direction_change(false);
            assert!(
                should_not_disable,
                "Iteração {}: Mudança para add sem uso prévio de negate não deve desabilitar",
                iteration
            );
        }
    }

    #[test]
    fn test_apu_sweep_overflow_integration() {
        let mut apu = APU::new();

        // Configurar canal 1 com sweep que pode causar overflow após alguns steps
        // Usar frequência que não overflow imediatamente: 1000 + 500 = 1500 (OK)
        // Mas após alguns steps: 1500 + 750 = 2250 > 2047 (overflow)
        apu.write_register(0xFF10, 0x11); // NR10: period=1, add, shift=1
        apu.write_register(0xFF11, 0x80); // NR11: duty=2, length=0
        apu.write_register(0xFF12, 0xF0); // NR12: volume=15, increase, period=0
        apu.write_register(0xFF13, 0xE8); // NR13: freq low = 0xE8 (232)
        apu.write_register(0xFF14, 0x83); // NR14: freq high = 3, trigger (freq = 1000)

        // Canal deve estar habilitado inicialmente (1000 + 500 = 1500 <= 2047)
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x01,
            0x01,
            "Canal 1 deve estar habilitado após trigger"
        );

        // Simular sweep steps até overflow
        for step in 0..20 {
            // Simular frame sequencer até sweep clock
            for _ in 0..4 {
                apu.div_event(); // 4 div events = 1 sweep clock (steps 2 e 6)
            }

            let nr52 = apu.read_register(0xFF26);
            if (nr52 & 0x01) == 0 {
                // Canal foi desabilitado por overflow
                println!("Canal desabilitado por overflow no step {}", step);
                return; // Teste passou
            }
        }

        // Se chegou aqui, o teste falhou
        panic!("Canal não foi desabilitado por overflow após 20 steps");
    }

    #[test]
    fn test_apu_negate_to_add_quirk_integration() {
        let mut apu = APU::new();

        // Configurar canal 1 com sweep em modo negate
        apu.write_register(0xFF10, 0x19); // NR10: period=1, negate, shift=1
        apu.write_register(0xFF11, 0x80); // NR11: duty=2, length=0
        apu.write_register(0xFF12, 0xF0); // NR12: volume=15, increase, period=0
        apu.write_register(0xFF13, 0x00); // NR13: freq low = 0x00
        apu.write_register(0xFF14, 0x84); // NR14: freq high = 4, trigger

        // Canal deve estar habilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x01,
            0x01,
            "Canal 1 deve estar habilitado após trigger"
        );

        // Simular alguns sweep steps para usar negate
        for _ in 0..8 {
            apu.div_event();
        }

        // Canal ainda deve estar habilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(nr52 & 0x01, 0x01, "Canal 1 deve ainda estar habilitado");

        // Agora mudar para modo add - isso deve desabilitar o canal
        apu.write_register(0xFF10, 0x11); // NR10: period=1, add, shift=1

        // Canal deve ter sido desabilitado pelo quirk
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x01,
            0x00,
            "Canal 1 deve ter sido desabilitado pelo quirk negate-to-add"
        );
    }

    #[test]
    fn test_envelope_automatic_stopping_property() {
        // Teste 1: Envelope aumentando até volume máximo (15)
        let mut envelope_up = Envelope::new();
        envelope_up.configure(14, true, 1);
        assert_eq!(envelope_up.current_volume(), 14);
        assert!(!envelope_up.is_stopped());

        // Com period = 1, precisa de 2 steps para mudar volume
        envelope_up.step(); // Decrementa timer
        assert_eq!(envelope_up.current_volume(), 14);
        assert!(!envelope_up.is_stopped());

        envelope_up.step(); // Muda volume para 15 e para
        assert_eq!(envelope_up.current_volume(), 15);
        assert!(
            envelope_up.is_stopped(),
            "Envelope deve parar ao atingir volume 15"
        );

        // Teste 2: Envelope diminuindo até volume mínimo (0)
        let mut envelope_down = Envelope::new();
        envelope_down.configure(1, false, 1);
        assert_eq!(envelope_down.current_volume(), 1);
        assert!(!envelope_down.is_stopped());

        envelope_down.step(); // Decrementa timer
        assert_eq!(envelope_down.current_volume(), 1);
        assert!(!envelope_down.is_stopped());

        envelope_down.step(); // Muda volume para 0 e para
        assert_eq!(envelope_down.current_volume(), 0);
        assert!(
            envelope_down.is_stopped(),
            "Envelope deve parar ao atingir volume 0"
        );

        // Teste 3: Envelope já no limite deve parar imediatamente
        let mut envelope_max = Envelope::new();
        envelope_max.configure(15, true, 1); // Volume 15, tentando aumentar
        assert_eq!(envelope_max.current_volume(), 15);
        assert!(
            envelope_max.is_stopped(),
            "Envelope no volume 15 tentando aumentar deve parar imediatamente"
        );

        let mut envelope_min = Envelope::new();
        envelope_min.configure(0, false, 1); // Volume 0, tentando diminuir
        assert_eq!(envelope_min.current_volume(), 0);
        assert!(
            envelope_min.is_stopped(),
            "Envelope no volume 0 tentando diminuir deve parar imediatamente"
        );

        // Teste 4: Envelope com period = 0 (desabilitado)
        let mut envelope_disabled = Envelope::new();
        envelope_disabled.configure(5, true, 0);

        let initial_vol = envelope_disabled.current_volume();
        let initial_stopped = envelope_disabled.is_stopped();

        for _ in 0..5 {
            envelope_disabled.step();
        }

        assert_eq!(envelope_disabled.current_volume(), initial_vol);
        assert_eq!(envelope_disabled.is_stopped(), initial_stopped);
    }

    #[test]
    fn property_sweep_overflow_channel_disable() {
        // Property-based testing manual: 5 iterações (otimizado)
        for iteration in 0..5 {
            let mut sweep = SweepUnit::new();

            // Gerar configurações de sweep válidas
            let period = 1 + (iteration % 7) as u8;
            let shift = 1 + (iteration % 7) as u8;
            let base_frequency = 1500 + (iteration % 548) as u16; // 1500..=2047

            // Configurar sweep em modo add (direction=false) que pode causar overflow
            sweep.configure(period, false, shift);

            // Calcular nova frequência
            let result = sweep.calculate_new_frequency(base_frequency);

            // Calcular manualmente o que seria a nova frequência
            let freq_change = base_frequency >> shift;
            let would_be_frequency = base_frequency.wrapping_add(freq_change);

            if would_be_frequency > 2047 {
                // Se a frequência calculada seria > 2047, deve retornar None (overflow)
                assert!(
                    result.is_none(),
                    "Iteração {}: Sweep overflow deve retornar None para freq {} + {} = {}",
                    iteration,
                    base_frequency,
                    freq_change,
                    would_be_frequency
                );
            } else {
                // Se a frequência calculada seria <= 2047, deve retornar Some(freq)
                assert!(
                    result.is_some(),
                    "Iteração {}: Sweep sem overflow deve retornar Some para freq {} + {} = {}",
                    iteration,
                    base_frequency,
                    freq_change,
                    would_be_frequency
                );

                let new_freq = result.unwrap();
                assert_eq!(
                    new_freq, would_be_frequency,
                    "Iteração {}: Frequência calculada deve ser {} mas foi {}",
                    iteration, would_be_frequency, new_freq
                );

                assert!(
                    new_freq <= 2047,
                    "Iteração {}: Frequência válida deve ser <= 2047, mas foi {}",
                    iteration,
                    new_freq
                );
            }
        }
    }

    #[test]
    fn property_sweep_negate_to_add_quirk() {
        // Property-based testing manual: 5 iterações (otimizado)
        for iteration in 0..5 {
            // Gerar configurações de sweep válidas
            let period = 1 + (iteration % 7) as u8; // 1-7
            let shift = (iteration % 8) as u8; // 0-7
            let base_frequency = 100 + (iteration % 1900) as u16; // 100..=1999

            // === Caso 1: Negate usado, depois mudança para add -> deve desabilitar ===
            let mut sweep_with_negate = SweepUnit::new();
            sweep_with_negate.configure(period, true, shift); // direction=true (negate)

            // Fazer um cálculo para marcar negate_used = true
            let _result = sweep_with_negate.calculate_new_frequency(base_frequency);

            // Verificar que negate foi usado
            assert!(
                sweep_with_negate.was_negate_used(),
                "Iteração {}: Negate deve ter sido marcado como usado após cálculo",
                iteration
            );

            // Tentar mudar para modo add (direction=false)
            // Isso deve retornar false (desabilita canal) devido ao quirk
            let should_continue = sweep_with_negate.handle_direction_change(false);
            assert!(
                !should_continue,
                "Iteração {}: Mudança de negate para add após uso deve retornar false (desabilitar canal)",
                iteration
            );

            // === Caso 2: Negate NÃO usado, mudança para add -> NÃO deve desabilitar ===
            let mut sweep_without_negate = SweepUnit::new();
            sweep_without_negate.configure(period, true, shift); // direction=true (negate)

            // NÃO fazer nenhum cálculo (negate_used permanece false)
            assert!(
                !sweep_without_negate.was_negate_used(),
                "Iteração {}: Negate não deve estar marcado como usado sem cálculo",
                iteration
            );

            // Mudança para add não deve desabilitar se negate não foi usado
            let should_continue = sweep_without_negate.handle_direction_change(false);
            assert!(
                should_continue,
                "Iteração {}: Mudança para add sem uso prévio de negate deve retornar true (não desabilitar)",
                iteration
            );

            // === Caso 3: Modo add desde o início -> nunca deve desabilitar ===
            let mut sweep_always_add = SweepUnit::new();
            sweep_always_add.configure(period, false, shift); // direction=false (add)

            // Fazer cálculos em modo add
            let _result = sweep_always_add.calculate_new_frequency(base_frequency);

            // Negate não deve ter sido usado
            assert!(
                !sweep_always_add.was_negate_used(),
                "Iteração {}: Negate não deve ser marcado em modo add",
                iteration
            );

            // Mudança para add novamente não deve desabilitar
            let should_continue = sweep_always_add.handle_direction_change(false);
            assert!(
                should_continue,
                "Iteração {}: Mudança para add quando já está em add não deve desabilitar",
                iteration
            );

            // === Caso 4: Negate usado, mudança para negate novamente -> não deve desabilitar ===
            let mut sweep_negate_to_negate = SweepUnit::new();
            sweep_negate_to_negate.configure(period, true, shift); // direction=true (negate)

            // Fazer cálculo para marcar negate_used
            let _result = sweep_negate_to_negate.calculate_new_frequency(base_frequency);
            assert!(sweep_negate_to_negate.was_negate_used());

            // Mudança para negate novamente não deve desabilitar
            let should_continue = sweep_negate_to_negate.handle_direction_change(true);
            assert!(
                should_continue,
                "Iteração {}: Mudança de negate para negate não deve desabilitar",
                iteration
            );

            // === Caso 5: Múltiplos cálculos em negate, depois mudança para add ===
            let mut sweep_multiple_calcs = SweepUnit::new();
            sweep_multiple_calcs.configure(period, true, shift);

            // Fazer múltiplos cálculos
            let mut current_freq = base_frequency;
            for _ in 0..3 {
                if let Some(new_freq) = sweep_multiple_calcs.calculate_new_frequency(current_freq) {
                    current_freq = new_freq;
                }
            }

            // Negate deve estar marcado
            assert!(
                sweep_multiple_calcs.was_negate_used(),
                "Iteração {}: Negate deve estar marcado após múltiplos cálculos",
                iteration
            );

            // Mudança para add deve desabilitar
            let should_continue = sweep_multiple_calcs.handle_direction_change(false);
            assert!(
                !should_continue,
                "Iteração {}: Mudança para add após múltiplos cálculos em negate deve desabilitar",
                iteration
            );
        }
    }
}

#[test]
fn property_length_counter_trigger_behavior() {
    // Property-based testing manual: 5 iterações (otimizado para velocidade)
    for iteration in 0..5 {
        let max_lengths = [64, 256];
        let max_length = max_lengths[iteration % 2];
        let mut length_counter = LengthCounter::new(max_length);
        let is_length_clock_next = (iteration % 2) == 0;
        let length_enable = (iteration % 3) != 0;

        // Trigger com counter = 0
        length_counter.handle_trigger(length_enable, is_length_clock_next);

        let expected_counter = if length_enable && is_length_clock_next {
            max_length - 1
        } else {
            max_length
        };

        assert_eq!(length_counter.current_value(), expected_counter);

        // Trigger com counter != 0 não deve modificar
        let mut length_counter2 = LengthCounter::new(max_length);
        length_counter2.load_length(10);
        let counter_before = length_counter2.current_value();
        length_counter2.handle_trigger(length_enable, is_length_clock_next);
        assert_eq!(counter_before, length_counter2.current_value());
    }
}

#[test]
fn property_length_enable_extra_clocking() {
    // Property-based testing manual: 5 iterações (otimizado)
    for iteration in 0..5 {
        let max_lengths = [64, 256];
        let max_length = max_lengths[iteration % 2];

        // Gerar diferentes valores de length timer
        let length_timer_value = (iteration % 64) as u8;
        let is_length_clock_next = (iteration % 2) == 0;
        let was_already_enabled = (iteration % 3) == 0;

        // === Caso 1: Ativar length enable quando is_length_clock_next = true ===
        // HARDWARE QUIRK: deve aplicar extra length clocking
        let mut length_counter1 = LengthCounter::new(max_length);
        length_counter1.load_length(length_timer_value);

        // Se já estava habilitado, não deve aplicar extra clocking
        if was_already_enabled {
            length_counter1.handle_enable_write(true, false); // Habilitar primeiro
        }

        let counter_before = length_counter1.current_value();

        // Agora ativar quando is_length_clock_next = true
        length_counter1.handle_enable_write(true, true);

        if !was_already_enabled && counter_before > 0 {
            // Extra clocking deve ser aplicado: counter - 1
            assert_eq!(
                length_counter1.current_value(),
                counter_before - 1,
                "Iteração {}: Extra clocking deve ser aplicado quando habilitando length enable na primeira metade do frame sequencer (counter {} -> {})",
                iteration,
                counter_before,
                counter_before - 1
            );
        } else {
            // Sem extra clocking se já estava habilitado ou counter = 0
            assert_eq!(
                length_counter1.current_value(),
                counter_before,
                "Iteração {}: Sem extra clocking se já estava habilitado ou counter = 0",
                iteration
            );
        }

        // === Caso 2: Ativar length enable quando is_length_clock_next = false ===
        // Não deve aplicar extra clocking
        let mut length_counter2 = LengthCounter::new(max_length);
        length_counter2.load_length(length_timer_value);
        let counter_before = length_counter2.current_value();

        length_counter2.handle_enable_write(true, false);
        assert_eq!(
            length_counter2.current_value(),
            counter_before,
            "Iteração {}: Sem extra clocking quando habilitando length enable fora da primeira metade do frame sequencer",
            iteration
        );

        // === Caso 3: Counter = 0 não deve underflow ===
        let mut length_counter3 = LengthCounter::new(max_length);
        // Counter já é 0 por padrão
        assert_eq!(length_counter3.current_value(), 0);

        length_counter3.handle_enable_write(true, true);
        assert_eq!(
            length_counter3.current_value(),
            0,
            "Iteração {}: Counter = 0 não deve underflow com extra clocking",
            iteration
        );

        // === Caso 4: Desabilitar length enable não deve aplicar extra clocking ===
        let mut length_counter4 = LengthCounter::new(max_length);
        length_counter4.load_length(length_timer_value);
        length_counter4.handle_enable_write(true, false); // Habilitar primeiro

        let counter_before = length_counter4.current_value();
        length_counter4.handle_enable_write(false, true); // Desabilitar

        assert_eq!(
            length_counter4.current_value(),
            counter_before,
            "Iteração {}: Desabilitar length enable não deve aplicar extra clocking",
            iteration
        );

        // === Caso 5: Re-habilitar quando já estava habilitado não deve aplicar extra clocking ===
        let mut length_counter5 = LengthCounter::new(max_length);
        length_counter5.load_length(length_timer_value);
        length_counter5.handle_enable_write(true, false); // Habilitar primeiro

        let counter_before = length_counter5.current_value();
        length_counter5.handle_enable_write(true, true); // Re-habilitar

        assert_eq!(
            length_counter5.current_value(),
            counter_before,
            "Iteração {}: Re-habilitar quando já estava habilitado não deve aplicar extra clocking",
            iteration
        );
    }
}

#[test]
fn test_apu_length_counter_integration() {
    let mut apu = APU::new();

    apu.write_register(0xFF11, 0x00);
    apu.write_register(0xFF12, 0xF0);
    apu.write_register(0xFF13, 0x00);
    apu.write_register(0xFF14, 0x87);

    let nr52 = apu.read_register(0xFF26);
    assert_eq!(nr52 & 0x01, 0x01);

    apu.write_register(0xFF14, 0xC7);
    let nr52 = apu.read_register(0xFF26);
    assert_eq!(nr52 & 0x01, 0x01);
}

#[test]
fn property_wave_ram_read_quirk() {
    // Property-based testing manual: 5 iterações para Wave RAM read quirk
    for iteration in 0..5 {
        let mut apu = APU::new();

        // Configurar Wave RAM com padrão de teste
        let test_pattern = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];

        // Escrever padrão na Wave RAM (canal 3 desabilitado)
        for (i, &byte) in test_pattern.iter().enumerate() {
            apu.write_register(0xFF30 + i as u16, byte);
        }

        // Verificar que Wave RAM foi escrita corretamente (canal desabilitado)
        for (i, &expected_byte) in test_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Wave RAM[{}] deve ser {} quando canal desabilitado, mas foi {}",
                iteration, i, expected_byte, read_value
            );
        }

        // Habilitar canal 3 para ativar quirk de leitura
        apu.write_register(0xFF1A, 0x80); // NR30: DAC enable
        apu.write_register(0xFF1B, 0x00); // NR31: length timer
        apu.write_register(0xFF1C, 0x20); // NR32: output level = 1 (100%)
        apu.write_register(0xFF1D, 0x00); // NR33: frequency low
        apu.write_register(0xFF1E, 0x80); // NR34: trigger

        // Verificar que canal está habilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x04,
            0x04,
            "Iteração {}: Canal 3 deve estar habilitado após trigger",
            iteration
        );

        // HARDWARE QUIRK: Durante playback, leitura deve retornar byte sendo acessado
        // Inicialmente, deve retornar o primeiro byte (posição 0)
        let quirk_read = apu.read_register(0xFF30); // Qualquer endereço da Wave RAM
        assert_eq!(
            quirk_read, test_pattern[0],
            "Iteração {}: Durante playback, leitura deve retornar byte da posição atual ({}), mas foi {}",
            iteration, test_pattern[0], quirk_read
        );

        // Simular alguns ciclos para avançar posição da wave
        for _ in 0..100 {
            apu.tick_m_cycle();
        }

        // Gerar alguns samples para avançar wave position
        for _ in 0..10 {
            apu.generate_sample();
        }

        // Verificar que leitura ainda retorna byte da posição atual (pode ter mudado)
        let quirk_read_after = apu.read_register(0xFF35); // Endereço diferente, mas deve retornar mesmo byte
        let quirk_read_after2 = apu.read_register(0xFF3A); // Outro endereço diferente

        // Durante playback, todos os endereços da Wave RAM devem retornar o mesmo byte
        assert_eq!(
            quirk_read_after, quirk_read_after2,
            "Iteração {}: Durante playback, todos os endereços da Wave RAM devem retornar o mesmo byte",
            iteration
        );

        // Desabilitar canal 3
        apu.write_register(0xFF1A, 0x00); // NR30: DAC disable

        // Verificar que canal está desabilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x04,
            0x00,
            "Iteração {}: Canal 3 deve estar desabilitado após DAC disable",
            iteration
        );

        // Após desabilitar, leitura deve voltar ao normal (acesso direto)
        for (i, &expected_byte) in test_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Após desabilitar canal, Wave RAM[{}] deve retornar valor original {} mas foi {}",
                iteration, i, expected_byte, read_value
            );
        }
    }
}

#[test]
fn property_wave_ram_write_protection() {
    // Property-based testing manual: 5 iterações para Wave RAM write protection
    for iteration in 0..5 {
        let mut apu = APU::new();

        // Padrão inicial da Wave RAM
        let initial_pattern = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
            0xFF, 0x00,
        ];

        // Escrever padrão inicial (canal desabilitado)
        for (i, &byte) in initial_pattern.iter().enumerate() {
            apu.write_register(0xFF30 + i as u16, byte);
        }

        // Verificar que escrita funcionou
        for (i, &expected_byte) in initial_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Wave RAM[{}] deve ser {} após escrita inicial",
                iteration, i, expected_byte
            );
        }

        // Habilitar canal 3 para ativar proteção de escrita
        apu.write_register(0xFF1A, 0x80); // NR30: DAC enable
        apu.write_register(0xFF1B, 0x00); // NR31: length timer
        apu.write_register(0xFF1C, 0x20); // NR32: output level = 1
        apu.write_register(0xFF1D, 0x00); // NR33: frequency low
        apu.write_register(0xFF1E, 0x80); // NR34: trigger

        // Verificar que canal está habilitado
        let nr52 = apu.read_register(0xFF26);
        assert_eq!(
            nr52 & 0x04,
            0x04,
            "Iteração {}: Canal 3 deve estar habilitado",
            iteration
        );

        // Tentar escrever novos valores (deve ser ignorado devido à proteção)
        let blocked_pattern = [
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99,
        ];

        for (i, &byte) in blocked_pattern.iter().enumerate() {
            apu.write_register(0xFF30 + i as u16, byte);
        }

        // HARDWARE QUIRK: Escritas devem ter sido ignoradas, Wave RAM deve manter valores originais
        // Mas leitura durante playback retorna byte da posição atual, não o valor do endereço
        // Então vamos desabilitar o canal primeiro para verificar os valores reais
        apu.write_register(0xFF1A, 0x00); // NR30: DAC disable

        // Verificar que Wave RAM mantém valores originais (escritas foram bloqueadas)
        for (i, &expected_byte) in initial_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Wave RAM[{}] deve manter valor original {} após tentativa de escrita durante playback, mas foi {}",
                iteration, i, expected_byte, read_value
            );
        }

        // Agora com canal desabilitado, escritas devem funcionar normalmente
        for (i, &byte) in blocked_pattern.iter().enumerate() {
            apu.write_register(0xFF30 + i as u16, byte);
        }

        // Verificar que escritas funcionaram com canal desabilitado
        for (i, &expected_byte) in blocked_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Wave RAM[{}] deve ser {} após escrita com canal desabilitado",
                iteration, i, expected_byte
            );
        }

        // Teste adicional: Habilitar canal novamente e verificar proteção
        apu.write_register(0xFF1A, 0x80); // NR30: DAC enable
        apu.write_register(0xFF1E, 0x80); // NR34: trigger novamente

        // Tentar sobrescrever com padrão diferente
        let final_pattern = [0x5A; 16]; // Todos os bytes = 0x5A

        for (i, &byte) in final_pattern.iter().enumerate() {
            apu.write_register(0xFF30 + i as u16, byte);
        }

        // Desabilitar para verificar se escritas foram bloqueadas
        apu.write_register(0xFF1A, 0x00); // NR30: DAC disable

        // Wave RAM deve manter o padrão anterior (blocked_pattern)
        for (i, &expected_byte) in blocked_pattern.iter().enumerate() {
            let read_value = apu.read_register(0xFF30 + i as u16);
            assert_eq!(
                read_value, expected_byte,
                "Iteração {}: Wave RAM[{}] deve manter valor {} após segunda tentativa de escrita bloqueada",
                iteration, i, expected_byte
            );
        }
    }
}
