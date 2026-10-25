use gb_emu::GB::timer::Timer;

const TAC_ON: u8 = 0x05; // habilitado, borda no bit 3 (a cada 16 T-cycles)

/// Overflow no T-cycle em que div_counter chega a 16, seguido de TAC=0.
/// Retorna (timer, TIMA, IF) com o reload previsto para div_counter = 20.
fn overflow_then_disable() -> (Timer, u8, u8) {
    let mut timer = Timer::new();
    timer.notify_tma_write(0x42);
    timer.set_div_counter(8);
    let (tima, if_reg, _) = timer.tick(1, 0xFF, TAC_ON, 0, false);
    let (tima, if_reg, _) = timer.tick(7, tima, TAC_ON, if_reg, false);
    assert_eq!((timer.get_div_counter(), tima), (16, 0));
    let (tima, if_reg) = timer.write_tac(tima, TAC_ON, 0, if_reg);
    (timer, tima, if_reg)
}

#[test]
fn reload_after_disable_waits_four_t_cycles_and_interrupts() {
    let (mut timer, tima, if_reg) = overflow_then_disable();
    let (tima, if_reg, _) = timer.tick(3, tima, 0, if_reg, false);
    assert_eq!((tima, if_reg & 0x04), (0, 0));
    let (tima, if_reg, _) = timer.tick(1, tima, 0, if_reg, false);
    assert_eq!((tima, if_reg & 0x04), (0x42, 0x04));
}

#[test]
fn tma_written_during_delay_is_reloaded_with_timer_off() {
    let (mut timer, tima, if_reg) = overflow_then_disable();
    timer.notify_tma_write(0x99);
    let (tima, _, _) = timer.tick(4, tima, 0, if_reg, false);
    assert_eq!(tima, 0x99);
}

#[test]
fn tima_written_during_delay_cancels_reload_with_timer_off() {
    let (mut timer, _, if_reg) = overflow_then_disable();
    assert!(timer.notify_tima_write(0));
    let (tima, if_reg, _) = timer.tick(8, 0x10, 0, if_reg, false);
    assert_eq!((tima, if_reg & 0x04), (0x10, 0));
}

#[test]
fn overflow_from_disable_glitch_reloads_one_m_cycle_after_the_write() {
    let mut timer = Timer::new();
    timer.notify_tma_write(0x42);
    timer.set_div_counter(8); // bit 3 alto: desligar gera a borda de descida
    let (tima, if_reg) = timer.write_tac(0xFF, TAC_ON, 0, 0);
    assert_eq!(tima, 0);
    // M-cycle da própria escrita
    let (tima, if_reg, _) = timer.tick(4, tima, 0, if_reg, false);
    assert_eq!((tima, if_reg & 0x04), (0, 0));
    let (tima, if_reg, _) = timer.tick(4, tima, 0, if_reg, false);
    assert_eq!((tima, if_reg & 0x04), (0x42, 0x04));
}

#[test]
fn disabled_timer_does_not_count() {
    let mut timer = Timer::new();
    let (tima, if_reg, _) = timer.tick(4096, 0xFF, 0, 0, false);
    assert_eq!((tima, if_reg), (0xFF, 0));
}
