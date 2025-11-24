// Integration tests para joypad
// cargo test joypad_test

#[cfg(test)]
mod joypad_tests {
    use gb_emu::GB::CPU::CPU;

    #[test]
    fn test_joypad_basic_operations() {
        let mut cpu = CPU::new(Vec::new());

        // Inicialmente, todos os botões devem estar soltos (bits = 1)
        // Seleciona D-pad: bit 4=0, bit 5=1 → escreve 0x20
        cpu.bus.write(0xFF00, 0x20);
        let val = cpu.bus.read(0xFF00);
        println!(
            "Estado inicial D-pad: 0x{:02X}, bits 3-0: 0x{:X}",
            val,
            val & 0x0F
        );
        assert_eq!(
            val & 0x0F,
            0x0F,
            "D-pad inicial deve ser 0x0F (todos soltos)"
        );

        // Pressiona RIGHT (bit 0)
        cpu.bus.joypad.press("RIGHT");
        cpu.bus.write(0xFF00, 0x20); // seleciona D-pad
        let val = cpu.bus.read(0xFF00);
        println!("Leitura após RIGHT: 0x{:02X}, bit 0: {}", val, val & 0x01);
        assert_eq!(val & 0x01, 0x00, "RIGHT deve estar pressionado (bit 0 = 0)");

        // Solta RIGHT
        cpu.bus.joypad.release("RIGHT");
        let val = cpu.bus.read(0xFF00);
        assert_eq!(val & 0x01, 0x01, "RIGHT deve estar solto (bit 0 = 1)");

        // Seleciona botões de ação: bit 5=0, bit 4=1 → escreve 0x10
        cpu.bus.write(0xFF00, 0x10);
        let val = cpu.bus.read(0xFF00);
        assert_eq!(val & 0x0F, 0x0F, "Botões ação inicial deve ser 0x0F");

        // Pressiona A (bit 0)
        cpu.bus.joypad.press("A");
        cpu.bus.write(0xFF00, 0x10); // seleciona ação (bit 5=0)
        let val = cpu.bus.read(0xFF00);
        assert_eq!(val & 0x01, 0x00, "A deve estar pressionado (bit 0 = 0)");

        // Pressiona START (bit 3)
        cpu.bus.joypad.press("START");
        let val = cpu.bus.read(0xFF00);
        assert_eq!(val & 0x08, 0x00, "START deve estar pressionado (bit 3 = 0)");

        println!("✅ Todos os testes de joypad passaram!");
    }

    #[test]
    fn test_joypad_multiple_buttons() {
        let mut cpu = CPU::new(Vec::new());

        // Pressiona múltiplos botões do D-pad
        cpu.bus.joypad.press("UP");
        cpu.bus.joypad.press("RIGHT");

        cpu.bus.write(0xFF00, 0x20); // seleciona D-pad (bit 4=0)
        let val = cpu.bus.read(0xFF00);

        // UP = bit 2, RIGHT = bit 0 → ambos devem ser 0
        assert_eq!(val & 0x05, 0x00, "UP e RIGHT devem estar pressionados");
        assert_eq!(val & 0x0A, 0x0A, "LEFT e DOWN devem estar soltos");

        println!("✅ Teste de múltiplos botões passou!");
    }

    #[test]
    fn test_joypad_mode_switching() {
        let mut cpu = CPU::new(Vec::new());

        // Pressiona botões de ambos os grupos
        cpu.bus.joypad.press("DOWN"); // D-pad
        cpu.bus.joypad.press("B"); // Ação

        // Lê D-pad: bit 4=0, bit 5=1 → 0x20
        cpu.bus.write(0xFF00, 0x20);
        let dpad = cpu.bus.read(0xFF00);
        assert_eq!(dpad & 0x08, 0x00, "DOWN deve aparecer no modo D-pad");
        assert_eq!(dpad & 0x02, 0x02, "B não deve aparecer no modo D-pad");

        // Lê ação: bit 5=0, bit 4=1 → 0x10
        cpu.bus.write(0xFF00, 0x10);
        let action = cpu.bus.read(0xFF00);
        assert_eq!(action & 0x02, 0x00, "B deve aparecer no modo ação");
        assert_eq!(action & 0x08, 0x08, "DOWN não deve aparecer no modo ação");

        println!("✅ Teste de troca de modo passou!");
    }
}
