use gb_emu::GB::mbc::{MBC, mbc3::MBC3};

#[test]
fn test_mbc3_rom_banking_basic() {
    // 128 banks × 16KB = 2MB (valor máximo típico do MBC3)
    let mut rom = vec![0u8; 2 * 1024 * 1024];

    // Preenche banco 2 com padrão reconhecível
    let bank2_start = 2 * 0x4000;
    for i in 0..0x4000 {
        rom[bank2_start + i] = 0xAA;
    }

    let mut mbc = MBC3::new(rom, 0);

    // Selecionar banco 2
    mbc.write_register(0x2000, 0x02);

    // Ler no range de banco selecionado
    let v = mbc.read_rom(0x4000);
    assert_eq!(v, 0xAA, "ROM banking failed for bank 2");
}

#[test]
fn test_mbc3_ram_enable_disable() {
    let rom = vec![0; 32 * 1024];
    let mut mbc = MBC3::new(rom, 8 * 1024);

    // RAM desabilitada → não deve escrever
    mbc.write_ram(0xA000, 0x55);
    assert_eq!(mbc.read_ram(0xA000), 0xFF);

    // Habilitar RAM
    mbc.write_register(0x0000, 0x0A);
    mbc.write_ram(0xA000, 0x77);
    assert_eq!(mbc.read_ram(0xA000), 0x77);
}

#[test]
fn test_mbc3_ram_banking() {
    // 4 bancos de 8KB
    let rom = vec![0; 32 * 1024];
    let mut mbc = MBC3::new(rom, 4 * 0x2000);

    mbc.write_register(0x0000, 0x0A); // enable RAM

    // Bank 0
    mbc.write_register(0x4000, 0x00);
    mbc.write_ram(0xA000, 0x11);

    // Bank 1
    mbc.write_register(0x4000, 0x01);
    mbc.write_ram(0xA000, 0x22);

    // Validar leitura por banco
    mbc.write_register(0x4000, 0x00);
    assert_eq!(mbc.read_ram(0xA000), 0x11);

    mbc.write_register(0x4000, 0x01);
    assert_eq!(mbc.read_ram(0xA000), 0x22);
}

#[test]
fn test_mbc3_rtc_basic_write_and_latch() {
    let rom = vec![0; 32 * 1024];
    let mut mbc = MBC3::new(rom, 0);

    // Habilitar RAM/RTC
    mbc.write_register(0x0000, 0x0A);

    // Escrever segundos no RTC (reg 0x08)
    mbc.write_register(0x4000, 0x08); // selecionar RTC S
    mbc.write_ram(0xA000, 30);

    // Latch 0 → 1
    mbc.write_register(0x6000, 0x00);
    mbc.write_register(0x6000, 0x01);

    // Ler via RTC latched register
    mbc.write_register(0x4000, 0x08);
    let sec = mbc.read_ram(0xA000);
    assert_eq!(
        sec, 30,
        "RTC seconds should match written value after latch"
    );
}

#[test]
fn test_mbc3_rtc_latch_freezes_time() {
    let rom = vec![0; 32 * 1024];
    let mut mbc = MBC3::new(rom, 0);

    // Habilitar RAM/RTC
    mbc.write_register(0x0000, 0x0A);

    // Escrever tempo inicial no RTC via registros
    // S = 10, M = 20
    mbc.write_register(0x4000, 0x08); // RTC S
    mbc.write_ram(0xA000, 10);
    mbc.write_register(0x4000, 0x09); // RTC M
    mbc.write_ram(0xA000, 20);

    // Latch (0 → 1)
    mbc.write_register(0x6000, 0x00);
    mbc.write_register(0x6000, 0x01);

    // Modificar o RTC "ao vivo" depois do latch
    mbc.write_register(0x4000, 0x08); // RTC S
    mbc.write_ram(0xA000, 50);
    mbc.write_register(0x4000, 0x09); // RTC M
    mbc.write_ram(0xA000, 40);

    // Ler valores *latched* (devem ser os antigos)
    mbc.write_register(0x4000, 0x08);
    let latched_s = mbc.read_ram(0xA000);
    mbc.write_register(0x4000, 0x09);
    let latched_m = mbc.read_ram(0xA000);

    assert_eq!(
        latched_s, 10,
        "Latched RTC seconds should not change after RTC update"
    );
    assert_eq!(
        latched_m, 20,
        "Latched RTC minutes should not change after RTC update"
    );
}

#[test]
fn test_mbc3_save_and_load_ram_with_rtc() {
    let rom = vec![0; 32 * 1024];
    let mut mbc = MBC3::new(rom, 8 * 1024);

    // Habilitar RAM/RTC
    mbc.write_register(0x0000, 0x0A);

    // Escrever RAM banco 0
    mbc.write_register(0x4000, 0x00);
    mbc.write_ram(0xA000, 0x42);

    // Escrever alguns registros de RTC via interface normal
    mbc.write_register(0x4000, 0x08); // S
    mbc.write_ram(0xA000, 5);
    mbc.write_register(0x4000, 0x09); // M
    mbc.write_ram(0xA000, 10);
    mbc.write_register(0x4000, 0x0A); // H
    mbc.write_ram(0xA000, 1);

    // Salvar
    let saved = mbc.save_ram().expect("MBC3 should have save data");

    // Criar novo MBC3 e carregar save
    let rom2 = vec![0; 32 * 1024];
    let mut mbc2 = MBC3::new(rom2, 8 * 1024);
    mbc2.load_ram(&saved);

    // RAM deve ter sido restaurada
    mbc2.write_register(0x0000, 0x0A);
    mbc2.write_register(0x4000, 0x00);
    assert_eq!(mbc2.read_ram(0xA000), 0x42);

    // RTC: faz um latch e lê regs via latched view
    mbc2.write_register(0x6000, 0x00);
    mbc2.write_register(0x6000, 0x01);

    mbc2.write_register(0x4000, 0x08);
    let s = mbc2.read_ram(0xA000);
    mbc2.write_register(0x4000, 0x09);
    let m = mbc2.read_ram(0xA000);
    mbc2.write_register(0x4000, 0x0A);
    let h = mbc2.read_ram(0xA000);

    assert_eq!(s, 5, "RTC seconds should be restored from save");
    assert_eq!(m, 10, "RTC minutes should be restored from save");
    assert_eq!(h, 1, "RTC hours should be restored from save");
}
