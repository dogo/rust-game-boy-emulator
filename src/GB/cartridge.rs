//! Módulo para parsing e validação de cartuchos

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

/// Valida o logo Nintendo e o checksum do header
pub fn validate_header(data: &[u8]) -> Result<(), String> {
    if data.len() <= 0x014D {
        return Err("❌ ROM muito pequena para conter um header válido!".to_string());
    }

    let logo = &data[0x0104..=0x0133];
    if logo != NINTENDO_LOGO {
        return Err("❌ Logo Nintendo inválido no header da ROM!".to_string());
    }

    let mut x: u8 = 0;
    for i in 0x0134..=0x014C {
        x = x.wrapping_sub(data[i]).wrapping_sub(1);
    }
    let checksum = data[0x014D];
    if x != checksum {
        return Err(format!(
            "❌ Checksum do header inválido! Calculado: {:02X}, esperado: {:02X}",
            x, checksum
        ));
    }

    Ok(())
}

/// Extrai o título do jogo
pub fn get_title(data: &[u8]) -> String {
    let mut title = String::new();
    for addr in 0x0134..=0x0143 {
        let ch = data.get(addr).copied().unwrap_or(0);
        if ch == 0 {
            break;
        }
        if ch.is_ascii() {
            title.push(ch as char);
        }
    }
    title
}

/// Detecta se a ROM é Game Boy Color baseada na flag CGB (0x143)
pub fn is_cgb_rom(data: &[u8]) -> bool {
    let cgb_flag = data.get(0x143).copied().unwrap_or(0x00);
    // 0x80 = CGB compatible, 0xC0 = CGB only
    (cgb_flag & 0x80) != 0
}

/// Retorna o nome do tipo de cartucho
pub fn get_cart_type_name(cart_type: u8) -> &'static str {
    match cart_type {
        0x00 => "ROM ONLY",
        0x01 | 0x02 | 0x03 => "MBC1",
        0x05 | 0x06 => "MBC2",
        0x0F | 0x10 | 0x11 | 0x12 | 0x13 => "MBC3",
        0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => "MBC5",
        _ => "(desconhecido)",
    }
}

/// Calcula tamanho da ROM em KB
pub fn get_rom_size_kb(code: u8) -> u32 {
    let bytes: u32 = match code {
        0x00 => 32 * 1024,
        0x01 => 64 * 1024,
        0x02 => 128 * 1024,
        0x03 => 256 * 1024,
        0x04 => 512 * 1024,
        0x05 => 1024 * 1024,
        0x06 => 2048 * 1024,
        0x07 => 4096 * 1024,
        0x08 => 8192 * 1024,
        0x52 => 1152 * 1024,
        0x53 => 1280 * 1024,
        0x54 => 1536 * 1024,
        _ => 0,
    };
    bytes / 1024
}

/// Calcula tamanho da RAM em KB
pub fn get_ram_size_kb(code: u8) -> u32 {
    let bytes: u32 = match code {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    };
    bytes / 1024
}

/// Imprime informações do cartucho
pub fn print_info(data: &[u8]) {
    let title = get_title(data);
    let cart_type = data.get(0x0147).copied().unwrap_or(0xFF);
    let rom_code = data.get(0x0148).copied().unwrap_or(0xFF);
    let ram_code = data.get(0x0149).copied().unwrap_or(0xFF);

    println!("Título: {}", title);
    println!(
        "Cart: {:02X} ({}) | ROM: {} KB | RAM: {} KB",
        cart_type,
        get_cart_type_name(cart_type),
        get_rom_size_kb(rom_code),
        get_ram_size_kb(ram_code)
    );
}
