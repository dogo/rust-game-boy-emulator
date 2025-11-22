use crate::GB::CPU::CPU;
use crate::GB::instructions;

// === Funções de trace para operações da RAM ===

pub fn trace_mbc_ram_enable(enabled: bool) {
    println!(
        "[MBC] RAM {}",
        if enabled {
            "habilitado"
        } else {
            "desabilitado"
        }
    );
}

pub fn trace_mbc_rom_bank(old_bank: u8, new_bank: u8) {
    println!("[MBC] Banco ROM: {:02X} -> {:02X}", old_bank, new_bank);
}

pub fn trace_mbc5_rom_bank(old_bank: u16, new_bank: u16) {
    println!("[MBC5] Banco ROM: {:03X} -> {:03X}", old_bank, new_bank);
}

pub fn trace_mbc_ram_rtc_select(byte: u8) {
    let desc = if byte <= 0x03 {
        format!("RAM banco {:02X}", byte)
    } else if byte >= 0x08 && byte <= 0x0C {
        format!("RTC reg {:02X}", byte)
    } else {
        format!("valor {:02X}", byte)
    };
    println!("[MBC] Seleção: {}", desc);
}

// === Funções específicas do MBC1 ===

pub fn trace_mbc1_reg1_write(old_reg: u8, new_reg: u8, old_rom: u8, new_rom: u8) {
    println!(
        "[MBC1] Reg1: {:02X} -> {:02X} (ROM bank {:02X} -> {:02X})",
        old_reg, new_reg, old_rom, new_rom
    );
}

pub fn trace_mbc1_reg2_write(old_reg: u8, new_reg: u8, old_rom: u8, new_rom: u8, ram_bank: u8) {
    println!(
        "[MBC1] Reg2: {:02X} -> {:02X} (ROM {:02X} -> {:02X}, RAM bank {})",
        old_reg, new_reg, old_rom, new_rom, ram_bank
    );
}

pub fn trace_mbc1_mode_switch(old_mode: u8, new_mode: u8) {
    println!(
        "[MBC1] Mode: {} -> {} ({})",
        old_mode,
        new_mode,
        if new_mode == 0 {
            "ROM banking"
        } else {
            "RAM banking"
        }
    );
}

pub fn trace_mbc_rtc_latch(h: u8, m: u8, s: u8, dh: u8, dl: u8) {
    println!(
        "[MBC3] RTC latched: {:02}:{:02}:{:02} dia={}",
        h,
        m,
        s,
        ((dh as u16 & 1) << 8) | dl as u16
    );
}

pub fn trace_joypad_selection(dpad: bool, buttons: bool) {
    println!(
        "[JOYPAD] Seleção: dpad={} buttons={}",
        dpad as u8, buttons as u8
    );
}

pub fn trace_joypad_press(button: &str) {
    println!("[JOYPAD] {} pressionado", button);
}

pub fn trace_joypad_release(button: &str) {
    println!("[JOYPAD] {} solto", button);
}

pub fn trace_timer_div_reset() {
    println!("[TIMER] DIV<=00 (reset)");
}

pub fn trace_timer_tac(byte: u8) {
    let en = (byte & 0x04) != 0;
    let freq = match byte & 0x03 {
        0b00 => 4096,
        0b01 => 262144,
        0b10 => 65536,
        _ => 16384,
    };
    println!(
        "[TIMER] TAC<={:02X} (enable={}, freq={}Hz)",
        byte & 0x07,
        en as u8,
        freq
    );
}

pub fn trace_timer_tima(byte: u8) {
    println!("[TIMER] TIMA<={:02X}", byte);
}

pub fn trace_timer_tma(byte: u8) {
    println!("[TIMER] TMA<={:02X}", byte);
}

pub fn trace_timer_interrupt(tma: u8) {
    println!("[TIMER] IF(TIMER)=1; TIMA<=TMA({:02X})", tma);
}

// === Loop principal de trace ===

pub fn run_with_trace(cpu: &mut CPU, max_steps: usize) {
    // Ativa trace de operações da RAM (MBC, timer, joypad)
    // Trace flag removed: MemoryBus does not have trace_enabled

    for step in 0..max_steps {
        let pc = cpu.registers.get_pc();
        let opcode = cpu.bus.read(pc); // peek
        let instr = instructions::decode(opcode);
        // Detalhes extras para diagnosticar polling em IO
        let extra = build_trace_extra(cpu, pc, opcode);

        if extra.is_empty() {
            println!("{:05} PC={:04X} OP={:02X} {}", step, pc, opcode, instr.name);
        } else {
            println!(
                "{:05} PC={:04X} OP={:02X} {}{}",
                step, pc, opcode, instr.name, extra
            );
        }

        let (_cycles, unknown) = cpu.execute_next();
        if unknown {
            println!("Parando: opcode desconhecido {:02X} em {:04X}", opcode, pc);
            break;
        }
    }
    println!("Total cycles: {}", cpu.cycles);
}

fn build_trace_extra(cpu: &CPU, pc: u16, opcode: u8) -> String {
    match opcode {
        // CB prefix — mostra operação, registrador/bit e valores relevantes
        0xCB => build_cb_trace(cpu, pc),

        // LDH A,(n)
        0xF0 => {
            let n = cpu.bus.read(pc.wrapping_add(1));
            let val = cpu.bus.read(0xFF00u16.wrapping_add(n as u16));
            format!(" n={:02X} [FF{:02X}]=>{:02X}", n, n, val)
        }

        // LDH (n),A
        0xE0 => {
            let n = cpu.bus.read(pc.wrapping_add(1));
            let a = cpu.registers.get_a();
            format!(" n={:02X} [FF{:02X}]<=A({:02X})", n, n, a)
        }

        // LD A,(C)
        0xF2 => {
            let c = cpu.registers.get_c();
            let val = cpu.bus.read(0xFF00u16.wrapping_add(c as u16));
            format!(" C={:02X} [FF{:02X}]=>{:02X}", c, c, val)
        }

        // LD (C),A
        0xE2 => {
            let c = cpu.registers.get_c();
            let a = cpu.registers.get_a();
            format!(" C={:02X} [FF{:02X}]<=A({:02X})", c, c, a)
        }

        // LD A,(a16)
        0xFA => {
            let lo = cpu.bus.read(pc.wrapping_add(1)) as u16;
            let hi = cpu.bus.read(pc.wrapping_add(2)) as u16;
            let addr = (hi << 8) | lo;
            let val = cpu.bus.read(addr);
            format!(" a16={:04X}=>{:02X}", addr, val)
        }

        // LD (a16),A
        0xEA => {
            let lo = cpu.bus.read(pc.wrapping_add(1)) as u16;
            let hi = cpu.bus.read(pc.wrapping_add(2)) as u16;
            let addr = (hi << 8) | lo;
            let a = cpu.registers.get_a();
            format!(" a16={:04X}<=A({:02X})", addr, a)
        }

        // CP A,d8 (FE) — mostra comparacao e flags resultantes
        0xFE => {
            let n = cpu.bus.read(pc.wrapping_add(1));
            let a = cpu.registers.get_a();
            let z = a == n; // Z set if equal
            let c = a < n; // C set if borrow (a < n)
            let h = (a & 0x0F) < (n & 0x0F); // half-borrow
            format!(
                " A={:02X} n={:02X} => Z={} N=1 H={} C={}",
                a, n, z as u8, h as u8, c as u8
            )
        }

        // JR cc,r8 — 20,28,30,38: mostra offset, condicao e alvo
        0x20 | 0x28 | 0x30 | 0x38 => {
            let off = cpu.bus.read(pc.wrapping_add(1)) as i8;
            let base = cpu.registers.get_pc().wrapping_add(2) as i32;
            let target = (base + off as i32) as u16;
            let z = cpu.registers.get_flag_z();
            let c = cpu.registers.get_flag_c();
            let cond = match opcode {
                0x20 => !z, // NZ
                0x28 => z,  // Z
                0x30 => !c, // NC
                0x38 => c,  // C
                _ => false,
            };
            format!(
                " r8={:+#04X} cond={} target={:04X}",
                off, cond as u8, target
            )
        }

        // JR r8 incondicional — 18
        0x18 => {
            let off = cpu.bus.read(pc.wrapping_add(1)) as i8;
            let base = cpu.registers.get_pc().wrapping_add(2) as i32;
            let target = (base + off as i32) as u16;
            format!(" r8={:+#04X} target={:04X}", off, target)
        }

        _ => String::new(),
    }
}

fn build_cb_trace(cpu: &CPU, pc: u16) -> String {
    let cb = cpu.bus.read(pc.wrapping_add(1));
    let r_idx = cb & 0x07;
    let bit_idx = (cb >> 3) & 0x07;
    let r_name = match r_idx {
        0 => "B",
        1 => "C",
        2 => "D",
        3 => "E",
        4 => "H",
        5 => "L",
        6 => "(HL)",
        _ => "A",
    };

    let val: u8 = if r_idx == 6 {
        cpu.bus.read(cpu.registers.get_hl())
    } else {
        match r_idx {
            0 => cpu.registers.get_b(),
            1 => cpu.registers.get_c(),
            2 => cpu.registers.get_d(),
            3 => cpu.registers.get_e(),
            4 => cpu.registers.get_h(),
            5 => cpu.registers.get_l(),
            _ => cpu.registers.get_a(),
        }
    };

    let desc = if cb <= 0x07 {
        "RLC"
    } else if cb <= 0x0F {
        "RRC"
    } else if cb <= 0x17 {
        "RL"
    } else if cb <= 0x1F {
        "RR"
    } else if cb <= 0x27 {
        "SLA"
    } else if cb <= 0x2F {
        "SRA"
    } else if cb <= 0x37 {
        "SWAP"
    } else if cb <= 0x3F {
        "SRL"
    } else if cb <= 0x7F {
        "BIT"
    } else if cb <= 0xBF {
        "RES"
    } else {
        "SET"
    };

    match desc {
        "BIT" => {
            let bit_set = (val & (1u8 << bit_idx)) != 0;
            format!(
                " CB={:02X} {} {},{} val={:02X} => Z={}",
                cb,
                desc,
                bit_idx,
                r_name,
                val,
                (!bit_set) as u8
            )
        }
        "RES" | "SET" => {
            format!(
                " CB={:02X} {} {},{} before={:02X}",
                cb, desc, bit_idx, r_name, val
            )
        }
        _ => {
            // Predict carry flag outcome for shifts/rotates
            let c_in = if cpu.registers.get_flag_c() { 1 } else { 0 };
            let bit7 = (val >> 7) & 1;
            let bit0 = val & 1;
            let c_out = match desc {
                "RLC" | "RL" | "SLA" => bit7,
                "RRC" | "RR" | "SRA" | "SRL" => bit0,
                "SWAP" => 0,
                _ => c_in,
            };
            if desc == "SWAP" {
                format!(
                    " CB={:02X} {} {} val={:02X} C_in={} => C_out=0",
                    cb, desc, r_name, val, c_in
                )
            } else {
                format!(
                    " CB={:02X} {} {} val={:02X} C_in={} => C_out={}",
                    cb, desc, r_name, val, c_in, c_out
                )
            }
        }
    }
}
