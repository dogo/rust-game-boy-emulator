# Rust Game Boy Emulator

A Game Boy emulator written in Rust for learning purposes.

## About

This project is being developed to study and learn about:
- Rust programming language
- Emulator development
- Game Boy hardware architecture

I'm following the excellent guide at [https://aquova.net/emudev/gb/index.html](https://aquova.net/emudev/gb/index.html) to understand the Game Boy's internals and implement the emulator.

## Status


🚧 Work in progress

## Test Status

| Suite   | Command                    | Total | Passed | Failed | Timeout |
|---------|----------------------------|-------|--------|--------|---------|
| Blargg  | `./run_all_tests.sh blargg` | 52    | 52     | 0      | 0       |
| Mooneye | `./run_all_tests.sh mooneye` | 111   | 82     | 29     | 0       |

### Blargg Test Status

| Test                                     | Status         |
|------------------------------------------|----------------|
| **cpu_instrs**                           |                |
| 01-special                               | ✅ Passed      |
| 02-interrupts                            | ✅ Passed      |
| 03-op sp,hl                              | ✅ Passed      |
| 04-op r,imm                              | ✅ Passed      |
| 05-op rp                                 | ✅ Passed      |
| 06-ld r,r                                | ✅ Passed      |
| 07-jr,jp,call,ret,rst                    | ✅ Passed      |
| 08-misc instrs                           | ✅ Passed      |
| 09-op r,r                                | ✅ Passed      |
| 10-bit ops                               | ✅ Passed      |
| 11-op a,(hl)                             | ✅ Passed      |
| **Other tests**                          |                |
| halt_bug                                 | ✅ Passed      |
| instr_timing                             | ✅ Passed      |
| interrupt_time                           | ✅ Passed      |
| **mem_timing**                           |                |
| 01-read_timing                           | ✅ Passed      |
| 02-write_timing                          | ✅ Passed      |
| 03-modify_timing                         | ✅ Passed      |
| **mem_timing-2**                         |                |
| 01-read_timing                           | ✅ Passed      |
| 02-write_timing                          | ✅ Passed      |
| 03-modify_timing                         | ✅ Passed      |
| **oam_bug**                              |                |
| 1-lcd_sync                               | ✅ Passed      |
| 2-causes                                 | ✅ Passed      |
| 3-non_causes                             | ✅ Passed      |
| 4-scanline_timing                        | ✅ Passed      |
| 5-timing_bug                             | ✅ Passed      |
| 6-timing_no_bug                          | ✅ Passed      |
| 7-timing_effect                          | ✅ Passed      |
| 8-instr_effect                           | ✅ Passed      |
| **dmg_sound**                            |                |
| 01-registers                             | ✅ Passed      |
| 02-len ctr                               | ✅ Passed      |
| 03-trigger                               | ✅ Passed      |
| 04-sweep                                 | ✅ Passed      |
| 05-sweep details                         | ✅ Passed      |
| 06-overflow on trigger                   | ✅ Passed      |
| 07-len sweep period sync                 | ✅ Passed      |
| 08-len ctr during power                  | ✅ Passed      |
| 09-wave read while on                    | ✅ Passed      |
| 10-wave trigger while on                 | ✅ Passed      |
| 11-regs after power                      | ✅ Passed      |
| 12-wave write while on                   | ✅ Passed      |
| **cgb_sound**                            |                |
| 01-registers                             | ✅ Passed      |
| 02-len ctr                               | ✅ Passed      |
| 03-trigger                               | ✅ Passed      |
| 04-sweep                                 | ✅ Passed      |
| 05-sweep details                         | ✅ Passed      |
| 06-overflow on trigger                   | ✅ Passed      |
| 07-len sweep period sync                 | ✅ Passed      |
| 08-len ctr during power                  | ✅ Passed      |
| 09-wave read while on                    | ✅ Passed      |
| 10-wave trigger while on                 | ✅ Passed      |
| 11-regs after power                      | ✅ Passed      |
| 12-wave                                  | ✅ Passed      |

### Mooneye Test Status

The automated Mooneye run includes `acceptance`, `emulator-only`, and `misc`.
It intentionally skips `manual-only`, `utils`, and `madness`.

| Group                    | Total | Passed | Failed | Timeout |
|--------------------------|-------|--------|--------|---------|
| acceptance               | 41    | 35     | 6      | 0       |
| acceptance/bits          | 3     | 3      | 0      | 0       |
| acceptance/instr         | 1     | 1      | 0      | 0       |
| acceptance/interrupts    | 1     | 1      | 0      | 0       |
| acceptance/oam_dma       | 3     | 3      | 0      | 0       |
| acceptance/ppu           | 12    | 0      | 12     | 0       |
| acceptance/serial        | 1     | 1      | 0      | 0       |
| acceptance/timer         | 13    | 13     | 0      | 0       |
| emulator-only/mbc1       | 13    | 13     | 0      | 0       |
| emulator-only/mbc2       | 7     | 7      | 0      | 0       |
| emulator-only/mbc5       | 8     | 8      | 0      | 0       |
| misc                     | 6     | 0      | 6      | 0       |
| misc/bits                | 1     | 0      | 1      | 0       |
| misc/ppu                 | 1     | 0      | 1      | 0       |

<details>
<summary>Full Mooneye test list</summary>

| Test | Status |
|------|--------|
| acceptance/add_sp_e_timing | ✅ Passed |
| acceptance/bits/mem_oam | ✅ Passed |
| acceptance/bits/reg_f | ✅ Passed |
| acceptance/bits/unused_hwio-GS | ✅ Passed |
| acceptance/boot_div-dmg0 | ❌ Failed |
| acceptance/boot_div-dmgABCmgb | ✅ Passed |
| acceptance/boot_div-S | ❌ Failed |
| acceptance/boot_div2-S | ❌ Failed |
| acceptance/boot_hwio-dmg0 | ❌ Failed |
| acceptance/boot_hwio-dmgABCmgb | ✅ Passed |
| acceptance/boot_hwio-S | ❌ Failed |
| acceptance/boot_regs-dmg0 | ❌ Failed |
| acceptance/boot_regs-dmgABC | ✅ Passed |
| acceptance/boot_regs-mgb | ❌ Failed |
| acceptance/boot_regs-sgb | ❌ Failed |
| acceptance/boot_regs-sgb2 | ❌ Failed |
| acceptance/call_cc_timing | ✅ Passed |
| acceptance/call_cc_timing2 | ✅ Passed |
| acceptance/call_timing | ✅ Passed |
| acceptance/call_timing2 | ✅ Passed |
| acceptance/di_timing-GS | ✅ Passed |
| acceptance/div_timing | ✅ Passed |
| acceptance/ei_sequence | ✅ Passed |
| acceptance/ei_timing | ✅ Passed |
| acceptance/halt_ime0_ei | ✅ Passed |
| acceptance/halt_ime0_nointr_timing | ✅ Passed |
| acceptance/halt_ime1_timing | ✅ Passed |
| acceptance/halt_ime1_timing2-GS | ✅ Passed |
| acceptance/if_ie_registers | ✅ Passed |
| acceptance/instr/daa | ✅ Passed |
| acceptance/interrupts/ie_push | ✅ Passed |
| acceptance/intr_timing | ✅ Passed |
| acceptance/jp_cc_timing | ✅ Passed |
| acceptance/jp_timing | ✅ Passed |
| acceptance/ld_hl_sp_e_timing | ✅ Passed |
| acceptance/oam_dma_restart | ✅ Passed |
| acceptance/oam_dma_start | ✅ Passed |
| acceptance/oam_dma_timing | ✅ Passed |
| acceptance/oam_dma/basic | ✅ Passed |
| acceptance/oam_dma/reg_read | ✅ Passed |
| acceptance/oam_dma/sources-GS | ✅ Passed |
| acceptance/pop_timing | ✅ Passed |
| acceptance/ppu/hblank_ly_scx_timing-GS | ❌ Failed |
| acceptance/ppu/intr_1_2_timing-GS | ❌ Failed |
| acceptance/ppu/intr_2_0_timing | ❌ Failed |
| acceptance/ppu/intr_2_mode0_timing_sprites | ❌ Failed |
| acceptance/ppu/intr_2_mode0_timing | ❌ Failed |
| acceptance/ppu/intr_2_mode3_timing | ❌ Failed |
| acceptance/ppu/intr_2_oam_ok_timing | ❌ Failed |
| acceptance/ppu/lcdon_timing-GS | ❌ Failed |
| acceptance/ppu/lcdon_write_timing-GS | ❌ Failed |
| acceptance/ppu/stat_irq_blocking | ❌ Failed |
| acceptance/ppu/stat_lyc_onoff | ❌ Failed |
| acceptance/ppu/vblank_stat_intr-GS | ❌ Failed |
| acceptance/push_timing | ✅ Passed |
| acceptance/rapid_di_ei | ✅ Passed |
| acceptance/ret_cc_timing | ✅ Passed |
| acceptance/ret_timing | ✅ Passed |
| acceptance/reti_intr_timing | ✅ Passed |
| acceptance/reti_timing | ✅ Passed |
| acceptance/rst_timing | ✅ Passed |
| acceptance/serial/boot_sclk_align-dmgABCmgb | ✅ Passed |
| acceptance/timer/div_write | ✅ Passed |
| acceptance/timer/rapid_toggle | ✅ Passed |
| acceptance/timer/tim00_div_trigger | ✅ Passed |
| acceptance/timer/tim00 | ✅ Passed |
| acceptance/timer/tim01_div_trigger | ✅ Passed |
| acceptance/timer/tim01 | ✅ Passed |
| acceptance/timer/tim10_div_trigger | ✅ Passed |
| acceptance/timer/tim10 | ✅ Passed |
| acceptance/timer/tim11_div_trigger | ✅ Passed |
| acceptance/timer/tim11 | ✅ Passed |
| acceptance/timer/tima_reload | ✅ Passed |
| acceptance/timer/tima_write_reloading | ✅ Passed |
| acceptance/timer/tma_write_reloading | ✅ Passed |
| emulator-only/mbc1/bits_bank1 | ✅ Passed |
| emulator-only/mbc1/bits_bank2 | ✅ Passed |
| emulator-only/mbc1/bits_mode | ✅ Passed |
| emulator-only/mbc1/bits_ramg | ✅ Passed |
| emulator-only/mbc1/multicart_rom_8Mb | ✅ Passed |
| emulator-only/mbc1/ram_256kb | ✅ Passed |
| emulator-only/mbc1/ram_64kb | ✅ Passed |
| emulator-only/mbc1/rom_16Mb | ✅ Passed |
| emulator-only/mbc1/rom_1Mb | ✅ Passed |
| emulator-only/mbc1/rom_2Mb | ✅ Passed |
| emulator-only/mbc1/rom_4Mb | ✅ Passed |
| emulator-only/mbc1/rom_512kb | ✅ Passed |
| emulator-only/mbc1/rom_8Mb | ✅ Passed |
| emulator-only/mbc2/bits_ramg | ✅ Passed |
| emulator-only/mbc2/bits_romb | ✅ Passed |
| emulator-only/mbc2/bits_unused | ✅ Passed |
| emulator-only/mbc2/ram | ✅ Passed |
| emulator-only/mbc2/rom_1Mb | ✅ Passed |
| emulator-only/mbc2/rom_2Mb | ✅ Passed |
| emulator-only/mbc2/rom_512kb | ✅ Passed |
| emulator-only/mbc5/rom_16Mb | ✅ Passed |
| emulator-only/mbc5/rom_1Mb | ✅ Passed |
| emulator-only/mbc5/rom_2Mb | ✅ Passed |
| emulator-only/mbc5/rom_32Mb | ✅ Passed |
| emulator-only/mbc5/rom_4Mb | ✅ Passed |
| emulator-only/mbc5/rom_512kb | ✅ Passed |
| emulator-only/mbc5/rom_64Mb | ✅ Passed |
| emulator-only/mbc5/rom_8Mb | ✅ Passed |
| misc/bits/unused_hwio-C | ❌ Failed |
| misc/boot_div-A | ❌ Failed |
| misc/boot_div-cgb0 | ❌ Failed |
| misc/boot_div-cgbABCDE | ❌ Failed |
| misc/boot_hwio-C | ❌ Failed |
| misc/boot_regs-A | ❌ Failed |
| misc/boot_regs-cgb | ❌ Failed |
| misc/ppu/vblank_stat_intr-C | ❌ Failed |

</details>

The headless test runner supports Blargg's memory output protocol at `$A000`.
It streams the text buffer at `$A004` incrementally, which lets verbose ROMs
such as `oam_bug/rom_singles/7-timing_effect.gb` finish without overflowing the
cartridge RAM text buffer.

## Mooneye Test ROMs

The `mooneye-test-suite` submodule tracks the official test sources. The upstream
repository does not commit compiled `.gb` files, so fetch the matching official
prebuilt package with:

```sh
./scripts/fetch_mooneye_roms.sh
```

This extracts the ROMs into `mooneye-roms/`, which is intentionally ignored by
Git.

Run a specific test suite with:

```sh
./run_all_tests.sh blargg
./run_all_tests.sh mooneye
./run_all_tests.sh all
```

## Resources

- [Game Boy Emulation Guide](https://aquova.net/emudev/gb/index.html)
- [Pandocs - Game Boy Technical Reference](https://gbdev.io/pandocs/) — comprehensive and up-to-date documentation on Game Boy hardware, instructions, quirks, and details.
