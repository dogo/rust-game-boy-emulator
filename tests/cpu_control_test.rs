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
