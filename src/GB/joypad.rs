// Joypad module: encapsula toda a lógica do controle

pub struct Joypad {
    select: u8,         // bits 4 e 5: seleção de grupo
    dpad: u8,          // bits 0-3: estado do D-pad (0=pressed, 1=released)
    buttons: u8,       // bits 0-3: estado dos botões de ação (0=pressed, 1=released)
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select: 0x30, // bits 4 e 5 = 1 (nenhum grupo selecionado)
            dpad: 0x0F,   // todos soltos
            buttons: 0x0F // todos soltos
        }
    }

    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    pub fn read(&self) -> u8 {
        let mut result = self.select;
        if self.select & 0x10 == 0 {
            // D-pad selecionado
            result |= self.dpad & 0x0F;
        } else if self.select & 0x20 == 0 {
            // Botões de ação selecionados
            result |= self.buttons & 0x0F;
        } else {
            // Nenhum grupo selecionado
            result |= 0x0F;
        }
        result
    }

    pub fn press(&mut self, button: &str) {
        match button {
            "RIGHT" => self.dpad &= !(1 << 0),
            "LEFT"  => self.dpad &= !(1 << 1),
            "UP"    => self.dpad &= !(1 << 2),
            "DOWN"  => self.dpad &= !(1 << 3),
            "A"      => self.buttons &= !(1 << 0),
            "B"      => self.buttons &= !(1 << 1),
            "SELECT" => self.buttons &= !(1 << 2),
            "START"  => self.buttons &= !(1 << 3),
            _ => {}
        }
    }

    pub fn release(&mut self, button: &str) {
        match button {
            "RIGHT" => self.dpad |= 1 << 0,
            "LEFT"  => self.dpad |= 1 << 1,
            "UP"    => self.dpad |= 1 << 2,
            "DOWN"  => self.dpad |= 1 << 3,
            "A"      => self.buttons |= 1 << 0,
            "B"      => self.buttons |= 1 << 1,
            "SELECT" => self.buttons |= 1 << 2,
            "START"  => self.buttons |= 1 << 3,
            _ => {}
        }
    }
}
