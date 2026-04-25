pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;
pub mod none;

pub fn create_mbc(rom: Vec<u8>) -> Box<dyn MBC + Send> {
    let cart_type = rom.get(0x0147).copied().unwrap_or(0x00);
    let ram_size = get_ram_size_for_type(&rom, cart_type);
    match cart_type {
        0x00 => Box::new(none::NoMBC::new(rom)),
        0x01..=0x03 => Box::new(mbc1::MBC1::new(rom, ram_size)),
        0x05..=0x06 => Box::new(mbc2::MBC2::new(rom)),
        0x0F..=0x13 => Box::new(mbc3::MBC3::new(rom, ram_size)),
        0x19..=0x1E => Box::new(mbc5::MBC5::new(rom, ram_size)),
        _ => Box::new(none::NoMBC::new(rom)),
    }
}

fn cart_type_has_ram(cart_type: u8) -> bool {
    matches!(
        cart_type,
        0x02 | 0x03
            | 0x08
            | 0x09
            | 0x0C
            | 0x0D
            | 0x10
            | 0x12
            | 0x13
            | 0x1A
            | 0x1B
            | 0x1D
            | 0x1E
            | 0x22
            | 0xFF
    )
}

fn get_ram_size_for_type(rom: &[u8], cart_type: u8) -> usize {
    let size = match rom.get(0x0149).copied().unwrap_or(0x00) {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    };
    // Alguns ROMs de teste declaram ram_size=0 mas cart_type indica RAM presente
    // Aloca 8KB por padrão nesses casos
    if size == 0 && cart_type_has_ram(cart_type) {
        8 * 1024
    } else {
        size
    }
}
pub trait MBC: Send {
    /// Lê um byte da ROM (0x0000-0x7FFF)
    fn read_rom(&self, address: u16) -> u8;

    /// Escreve em registradores do MBC (0x0000-0x7FFF)
    fn write_register(&mut self, address: u16, value: u8);

    /// Lê RAM externa (0xA000-0xBFFF)
    fn read_ram(&self, address: u16) -> u8;

    /// Escreve RAM externa (0xA000-0xBFFF)
    fn write_ram(&mut self, address: u16, value: u8);

    /// Salva RAM para arquivo
    fn save_ram(&self) -> Option<Vec<u8>>;

    /// Carrega RAM de arquivo
    fn load_ram(&mut self, data: &[u8]);
}
