use crate::GB::CPU::CPU;
use crate::GB::instructions;

pub fn run_with_trace(cpu: &mut CPU, max_steps: usize) {
    for step in 0..max_steps {
        let pc = cpu.registers.get_pc();
        let opcode = cpu.ram.read(pc); // peek
        let instr = instructions::decode(opcode);
        // Detalhes extras para diagnosticar polling em IO
        let extra = build_trace_extra(cpu, pc, opcode);

        if extra.is_empty() {
            println!("{:05} PC={:04X} OP={:02X} {}", step, pc, opcode, instr.name);
        } else {
            println!("{:05} PC={:04X} OP={:02X} {}{}", step, pc, opcode, instr.name, extra);
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
            let n = cpu.ram.read(pc.wrapping_add(1));
            let val = cpu.ram.read(0xFF00u16.wrapping_add(n as u16));
            format!(" n={:02X} [FF{:02X}]=>{:02X}", n, n, val)
        }

        // LDH (n),A
        0xE0 => {
            let n = cpu.ram.read(pc.wrapping_add(1));
            let a = cpu.registers.get_a();
            format!(" n={:02X} [FF{:02X}]<=A({:02X})", n, n, a)
        }

        // LD A,(C)
        0xF2 => {
            let c = cpu.registers.get_c();
            let val = cpu.ram.read(0xFF00u16.wrapping_add(c as u16));
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
            let lo = cpu.ram.read(pc.wrapping_add(1)) as u16;
            let hi = cpu.ram.read(pc.wrapping_add(2)) as u16;
            let addr = (hi << 8) | lo;
            let val = cpu.ram.read(addr);
            format!(" a16={:04X}=>{:02X}", addr, val)
        }

        // LD (a16),A
        0xEA => {
            let lo = cpu.ram.read(pc.wrapping_add(1)) as u16;
            let hi = cpu.ram.read(pc.wrapping_add(2)) as u16;
            let addr = (hi << 8) | lo;
            let a = cpu.registers.get_a();
            format!(" a16={:04X}<=A({:02X})", addr, a)
        }

        // CP A,d8 (FE) — mostra comparacao e flags resultantes
        0xFE => {
            let n = cpu.ram.read(pc.wrapping_add(1));
            let a = cpu.registers.get_a();
            let z = a == n; // Z set if equal
            let c = a < n;  // C set if borrow (a < n)
            let h = (a & 0x0F) < (n & 0x0F); // half-borrow
            format!(" A={:02X} n={:02X} => Z={} N=1 H={} C={}", a, n, z as u8, h as u8, c as u8)
        }

        // JR cc,r8 — 20,28,30,38: mostra offset, condicao e alvo
        0x20 | 0x28 | 0x30 | 0x38 => {
            let off = cpu.ram.read(pc.wrapping_add(1)) as i8;
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
            format!(" r8={:+#04X} cond={} target={:04X}", off, cond as u8, target)
        }

        // JR r8 incondicional — 18
        0x18 => {
            let off = cpu.ram.read(pc.wrapping_add(1)) as i8;
            let base = cpu.registers.get_pc().wrapping_add(2) as i32;
            let target = (base + off as i32) as u16;
            format!(" r8={:+#04X} target={:04X}", off, target)
        }

        _ => String::new()
    }
}

fn build_cb_trace(cpu: &CPU, pc: u16) -> String {
    let cb = cpu.ram.read(pc.wrapping_add(1));
    let r_idx = cb & 0x07;
    let bit_idx = (cb >> 3) & 0x07;
    let r_name = match r_idx {
        0 => "B", 1 => "C", 2 => "D", 3 => "E",
        4 => "H", 5 => "L", 6 => "(HL)", _ => "A"
    };

    let val: u8 = if r_idx == 6 {
        cpu.ram.read(cpu.registers.get_hl())
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
            format!(" CB={:02X} {} {},{} val={:02X} => Z={}", cb, desc, bit_idx, r_name, val, (!bit_set) as u8)
        }
        "RES" | "SET" => {
            format!(" CB={:02X} {} {},{} before={:02X}", cb, desc, bit_idx, r_name, val)
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
                format!(" CB={:02X} {} {} val={:02X} C_in={} => C_out=0", cb, desc, r_name, val, c_in)
            } else {
                format!(" CB={:02X} {} {} val={:02X} C_in={} => C_out={}", cb, desc, r_name, val, c_in, c_out)
            }
        }
    }
}
