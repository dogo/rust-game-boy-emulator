pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;
pub mod none;

pub fn create_mbc(rom: Vec<u8>) -> Box<dyn MBC> {
    let cart_type = rom.get(0x0147).copied().unwrap_or(0x00);
    let ram_size = get_ram_size(&rom);
    match cart_type {
        0x00 => Box::new(none::NoMBC::new(rom)),
        0x01..=0x03 => Box::new(mbc1::MBC1::new(rom, ram_size)),
        0x05..=0x06 => Box::new(mbc2::MBC2::new(rom)),
        0x0F..=0x13 => Box::new(mbc3::MBC3::new(rom, ram_size)),
        0x19..=0x1E => Box::new(mbc5::MBC5::new(rom, ram_size)),
        _ => Box::new(none::NoMBC::new(rom)),
    }
}

fn get_ram_size(rom: &[u8]) -> usize {
    match rom.get(0x0149).copied().unwrap_or(0x00) {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    }
}
pub trait MBC {
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
