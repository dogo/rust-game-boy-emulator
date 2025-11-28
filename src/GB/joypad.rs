// Joypad module: encapsula toda a lógica do controle

pub struct Joypad {
    select: u8,              // bits 4 e 5: seleção de grupo
    dpad: u8,                // bits 0-3: estado do D-pad (0=pressed, 1=released)
    buttons: u8,             // bits 0-3: estado dos botões de ação (0=pressed, 1=released)
    interrupt_pending: bool, // flag para IRQ
    prev_state: u8,          // estado anterior dos botões (active-low)
    state: u8,               // estado atual dos botões (active-low)
}

impl Joypad {
    /// Retorna o estado atual dos botões (active-low, 8 bits)
    pub fn raw_state(&self) -> u8 {
        self.state
    }
    pub fn new() -> Self {
        Joypad {
            select: 0x30,  // bits 4 e 5 = 1 (nenhum grupo selecionado)
            dpad: 0x0F,    // todos soltos
            buttons: 0x0F, // todos soltos
            interrupt_pending: false,
            prev_state: 0xFF,
            state: 0xFF,
        }
    }
    /// Atualiza o estado do Joypad (para edge detection)
    pub fn update_input(&mut self, new_state: u8) {
        self.prev_state = self.state;
        self.state = new_state;
    }

    /// Retorna true se houve algum botão que mudou de 1 -> 0 (não pressionado -> pressionado)
    pub fn has_new_press(&self) -> bool {
        let changed = self.prev_state ^ self.state;
        let newly_pressed = changed & (!self.state);
        newly_pressed != 0
    }

    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    pub fn read(&self) -> u8 {
        // bits 6 e 7 sempre 1
        let mut result = 0xC0 | self.select;
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
        let mut irq = false;
        match button {
            "RIGHT" => {
                if self.dpad & (1 << 0) != 0 {
                    irq = true;
                }
                self.dpad &= !(1 << 0);
            }
            "LEFT" => {
                if self.dpad & (1 << 1) != 0 {
                    irq = true;
                }
                self.dpad &= !(1 << 1);
            }
            "UP" => {
                if self.dpad & (1 << 2) != 0 {
                    irq = true;
                }
                self.dpad &= !(1 << 2);
            }
            "DOWN" => {
                if self.dpad & (1 << 3) != 0 {
                    irq = true;
                }
                self.dpad &= !(1 << 3);
            }
            "A" => {
                if self.buttons & (1 << 0) != 0 {
                    irq = true;
                }
                self.buttons &= !(1 << 0);
            }
            "B" => {
                if self.buttons & (1 << 1) != 0 {
                    irq = true;
                }
                self.buttons &= !(1 << 1);
            }
            "SELECT" => {
                if self.buttons & (1 << 2) != 0 {
                    irq = true;
                }
                self.buttons &= !(1 << 2);
            }
            "START" => {
                if self.buttons & (1 << 3) != 0 {
                    irq = true;
                }
                self.buttons &= !(1 << 3);
            }
            _ => {}
        }
        // Só dispara interrupção se houve transição solto->pressionado
        if irq {
            self.interrupt_pending = true;
        }
        let new_state = (self.dpad & 0x0F) | ((self.buttons & 0x0F) << 4);
        self.update_input(new_state);
    }
    /// Consome o pedido de interrupção, se houver
    pub fn take_interrupt_request(&mut self) -> bool {
        if self.interrupt_pending {
            self.interrupt_pending = false;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, button: &str) {
        match button {
            "RIGHT" => self.dpad |= 1 << 0,
            "LEFT" => self.dpad |= 1 << 1,
            "UP" => self.dpad |= 1 << 2,
            "DOWN" => self.dpad |= 1 << 3,
            "A" => self.buttons |= 1 << 0,
            "B" => self.buttons |= 1 << 1,
            "SELECT" => self.buttons |= 1 << 2,
            "START" => self.buttons |= 1 << 3,
            _ => {}
        }
        let new_state = (self.dpad & 0x0F) | ((self.buttons & 0x0F) << 4);
        self.update_input(new_state);
    }
}
