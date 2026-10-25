use gb_emu::GB::CPU::CPU;

fn cpu_with_rom(bytes: &[u8]) -> CPU {
    let mut rom = vec![0x00; 32 * 1024];
    rom[..bytes.len()].copy_from_slice(bytes);
    let mut cpu = CPU::new(rom);
    cpu.registers.set_pc(0x0000);
    cpu
}

#[test]
fn di_does_not_execute_cb_set_6_e() {
    let mut cpu = cpu_with_rom(&[0xF3]);
    cpu.registers.set_de(0x0000);

    let (_cycles, unknown) = cpu.execute_next();

    assert!(!unknown);
    assert_eq!(cpu.registers.get_de(), 0x0000);
    assert!(!cpu.ime);
}

#[test]
fn cb_f3_still_executes_set_6_e() {
    let mut cpu = cpu_with_rom(&[0xCB, 0xF3]);
    cpu.registers.set_de(0x0000);

    let (_cycles, unknown) = cpu.execute_next();

    assert!(!unknown);
    assert_eq!(cpu.registers.get_de(), 0x0040);
}

/// STOP em $0000, INC A em $0001 e $0002: o valor de A mostra se o padding foi pulado
fn stop_cpu() -> CPU {
    let mut cpu = cpu_with_rom(&[0x10, 0x3C, 0x3C, 0x18, 0xFE]);
    cpu.registers.set_a(0);
    cpu.bus.write(0xFFFF, 0x00);
    cpu.bus.write(0xFF0F, 0x00);
    cpu.bus.set_div_counter(0x1234);
    cpu
}

fn set_interrupt_pending(cpu: &mut CPU) {
    cpu.bus.write(0xFFFF, 0x01);
    cpu.bus.write(0xFF0F, 0x01);
}

#[test]
fn stop_resets_and_freezes_divider_and_ppu_until_joypad() {
    let mut cpu = stop_cpu();
    cpu.bus.write(0xFF00, 0x10); // seleciona os botões de ação
    cpu.execute_next();
    assert!(cpu.stopped);
    assert_eq!(cpu.registers.get_pc(), 0x0002);
    assert_eq!(cpu.bus.read(0xFF04), 0);
    let (div, ly) = (cpu.bus.read(0xFF04), cpu.bus.read(0xFF44));
    for _ in 0..20_000 {
        cpu.execute_next();
    }
    assert!(cpu.stopped);
    assert_eq!((cpu.bus.read(0xFF04), cpu.bus.read(0xFF44)), (div, ly));

    cpu.bus.joypad.press("A");
    cpu.execute_next();
    assert!(!cpu.stopped);
    assert_eq!(cpu.registers.get_a(), 1);
}

#[test]
fn stop_ignores_buttons_of_unselected_groups() {
    let mut cpu = stop_cpu();
    cpu.bus.write(0xFF00, 0x20); // seleciona só o D-pad
    cpu.execute_next();
    cpu.bus.joypad.press("A");
    cpu.execute_next();
    assert!(cpu.stopped);
}

#[test]
fn stop_with_pending_interrupt_is_one_byte() {
    let mut cpu = stop_cpu();
    cpu.bus.write(0xFF00, 0x10);
    set_interrupt_pending(&mut cpu);
    cpu.execute_next();
    assert!(cpu.stopped);
    assert_eq!(cpu.registers.get_pc(), 0x0001);
    assert_eq!(cpu.bus.read(0xFF04), 0);
}

#[test]
fn stop_with_button_held_enters_halt_without_resetting_div() {
    let mut cpu = stop_cpu();
    cpu.bus.write(0xFF00, 0x10);
    cpu.bus.joypad.press("A");
    cpu.execute_next();
    assert!(cpu.halted && !cpu.stopped);
    assert_eq!(cpu.registers.get_pc(), 0x0002);
    assert_ne!(cpu.bus.read(0xFF04), 0);
}

#[test]
fn stop_with_button_held_and_pending_interrupt_is_a_nop() {
    let mut cpu = stop_cpu();
    cpu.bus.write(0xFF00, 0x10);
    cpu.bus.joypad.press("A");
    set_interrupt_pending(&mut cpu);
    cpu.execute_next();
    assert!(!cpu.halted && !cpu.stopped);
    assert_eq!(cpu.registers.get_pc(), 0x0001);
    assert_ne!(cpu.bus.read(0xFF04), 0);
}
