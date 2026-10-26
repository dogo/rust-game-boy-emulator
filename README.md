# Rust Game Boy Emulator

A Game Boy emulator written in Rust for learning purposes.

## About

This project is being developed to study and learn about:
- Rust programming language
- Emulator development
- Game Boy hardware architecture

I'm following the excellent guide at [https://aquova.net/emudev/gb/index.html](https://aquova.net/emudev/gb/index.html) to understand the Game Boy's internals and implement the emulator.

## Status

A DMG (original Game Boy) emulator. CPU, timer, PPU, APU, OAM DMA, joypad, serial
and the MBC1/MBC2/MBC3 (with RTC)/MBC5 cartridges are emulated and pass the test
suites below. The current plan and its history are in [ROADMAP.md](ROADMAP.md);
what was checked in real games is in [VALIDACAO.md](VALIDACAO.md).

## Building and running

Requires a Rust toolchain (edition 2024) and the SDL3 shared library
(`sdl3` crate 0.16; CI builds SDL3 from source, see `.github/workflows/ci.yml`).

```sh
cargo build --release
./target/release/gb_emu <rom.gb> [--headless] [--trace] [--model <model>]
```

| Option | Effect |
|--------|--------|
| *(none)* | Opens a 3× window with audio |
| `--headless` | Runs without a window as a test runner: exits 0 on pass, 1 on failure, 2 on timeout (Blargg serial/memory and Mooneye protocols) |
| `--trace` | Prints every executed instruction |
| `--model <model>` | Hardware to emulate: `dmg0`, `dmg`, `mgb`, `sgb`, `sgb2`, `cgb0`, `cgb`, `agb`. Default: `dmg`, or `cgb` for CGB-only ROMs. The ROM file name never selects the model |

If a `dmg_boot.bin` (256 bytes) is in the working directory it is executed;
otherwise the emulator starts in the post-boot state of the chosen model.

`examples/frame_capture.rs` runs a ROM without a window from a button script and
writes screenshots and saves to an output directory:

```sh
cargo run --release --example frame_capture -- <rom> <out_dir> [save.sav] wait:600 tap:START shot:title
```

## Controls

| Key | Game Boy |
|-----|----------|
| Arrow keys | D-pad |
| Z | A |
| X | B |
| Enter | Start |
| Backspace | Select |
| F12 | Pause and open the debugger in the terminal |
| Esc | Quit (saves) |

### Debugger

F12 or a breakpoint pauses emulation and opens a `(gbd)` prompt in the terminal
that started the emulator. Main commands: `c` continue, `n` step, `run <n>`,
`b <addr>` / `d <addr>` / `l` breakpoints, `reg`, `p <addr> [n]` memory, `disass [n]`,
`io`, `stack [n]`, `q` quit, `h` help. While the prompt is open the window does not
process events.

## Saves

Battery-backed cartridge RAM (and the MBC3 clock) is loaded from and saved to a
`.sav` file next to the ROM. Saving writes a temporary file in the same directory
and replaces the old save only after the write succeeds; it is not fsynced, so it
is not guaranteed across power loss. An invalid save makes the emulator exit with
an error and leaves the file untouched.

## Hardware support and limits

- **Supported cartridge types** (`$0147`): `$00`–`$03`, `$05`–`$06`, `$08`–`$09`,
  `$0F`–`$13`, `$19`–`$1E`. Other types are rejected at startup.
- **Game Boy Color:** the PPU is DMG-only. CGB-compatible ROMs (flag `$80`) run as
  DMG games. **CGB-only ROMs are not supported:** they start in CGB mode but have no
  CGB graphics (VRAM/WRAM banks, attributes, palettes) or CGB DMA.
- **Super Game Boy:** the `sgb`/`sgb2` models only set the post-boot state; SGB
  borders, palettes and commands are not emulated.
- Graphics are drawn per scanline, so mid-scanline register changes are not
  reproduced.
- Illegal opcodes hard-lock the CPU, as on hardware.

## Tests

| Suite | Command | Result |
|-------|---------|--------|
| Rust unit and integration tests | `cargo test` | 157 passed, 0 ignored |
| Blargg | `./run_all_tests.sh blargg` | 52/52 |
| Mooneye (`acceptance`, `emulator-only`, `misc`) | `./run_all_tests.sh mooneye` | 111/111 |

`./run_all_tests.sh all` runs both ROM suites (163 ROMs; about 8 s on the machine in
[VALIDACAO.md](VALIDACAO.md)). It rebuilds the
release binary first, requires the expected number of ROMs, passes `--model` from
the test-ROM naming conventions and prints the output of failing ROMs. CI runs the
Rust tests and both ROM suites.

What the suites do and do not show:

- Blargg and Mooneye check CPU instructions, timing, interrupts, timer, OAM DMA,
  PPU timing, MBCs and APU registers. They are not visual or audio compatibility
  tests.
- `tests/acid2_test.rs` runs [dmg-acid2](https://github.com/mattcurrie/dmg-acid2)
  and compares the frame pixel by pixel with the hardware reference.
- `tests/apu_pitch_test.rs` measures the frequency of each sound channel from the
  generated samples.
- Four games were played through boot, menus, gameplay, saving and a one-hour
  session; see [VALIDACAO.md](VALIDACAO.md). That is evidence for those scenarios,
  not a compatibility list.

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
| acceptance               | 41    | 41     | 0      | 0       |
| acceptance/bits          | 3     | 3      | 0      | 0       |
| acceptance/instr         | 1     | 1      | 0      | 0       |
| acceptance/interrupts    | 1     | 1      | 0      | 0       |
| acceptance/oam_dma       | 3     | 3      | 0      | 0       |
| acceptance/ppu           | 12    | 12     | 0      | 0       |
| acceptance/serial        | 1     | 1      | 0      | 0       |
| acceptance/timer         | 13    | 13     | 0      | 0       |
| emulator-only/mbc1       | 13    | 13     | 0      | 0       |
| emulator-only/mbc2       | 7     | 7      | 0      | 0       |
| emulator-only/mbc5       | 8     | 8      | 0      | 0       |
| misc                     | 6     | 6      | 0      | 0       |
| misc/bits                | 1     | 1      | 0      | 0       |
| misc/ppu                 | 1     | 1      | 0      | 0       |

<details>
<summary>Full Mooneye test list</summary>

| Test | Status |
|------|--------|
| acceptance/add_sp_e_timing | ✅ Passed |
| acceptance/bits/mem_oam | ✅ Passed |
| acceptance/bits/reg_f | ✅ Passed |
| acceptance/bits/unused_hwio-GS | ✅ Passed |
| acceptance/boot_div-dmg0 | ✅ Passed |
| acceptance/boot_div-dmgABCmgb | ✅ Passed |
| acceptance/boot_div-S | ✅ Passed |
| acceptance/boot_div2-S | ✅ Passed |
| acceptance/boot_hwio-dmg0 | ✅ Passed |
| acceptance/boot_hwio-dmgABCmgb | ✅ Passed |
| acceptance/boot_hwio-S | ✅ Passed |
| acceptance/boot_regs-dmg0 | ✅ Passed |
| acceptance/boot_regs-dmgABC | ✅ Passed |
| acceptance/boot_regs-mgb | ✅ Passed |
| acceptance/boot_regs-sgb | ✅ Passed |
| acceptance/boot_regs-sgb2 | ✅ Passed |
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
| acceptance/ppu/hblank_ly_scx_timing-GS | ✅ Passed |
| acceptance/ppu/intr_1_2_timing-GS | ✅ Passed |
| acceptance/ppu/intr_2_0_timing | ✅ Passed |
| acceptance/ppu/intr_2_mode0_timing_sprites | ✅ Passed |
| acceptance/ppu/intr_2_mode0_timing | ✅ Passed |
| acceptance/ppu/intr_2_mode3_timing | ✅ Passed |
| acceptance/ppu/intr_2_oam_ok_timing | ✅ Passed |
| acceptance/ppu/lcdon_timing-GS | ✅ Passed |
| acceptance/ppu/lcdon_write_timing-GS | ✅ Passed |
| acceptance/ppu/stat_irq_blocking | ✅ Passed |
| acceptance/ppu/stat_lyc_onoff | ✅ Passed |
| acceptance/ppu/vblank_stat_intr-GS | ✅ Passed |
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
| misc/bits/unused_hwio-C | ✅ Passed |
| misc/boot_div-A | ✅ Passed |
| misc/boot_div-cgb0 | ✅ Passed |
| misc/boot_div-cgbABCDE | ✅ Passed |
| misc/boot_hwio-C | ✅ Passed |
| misc/boot_regs-A | ✅ Passed |
| misc/boot_regs-cgb | ✅ Passed |
| misc/ppu/vblank_stat_intr-C | ✅ Passed |

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
