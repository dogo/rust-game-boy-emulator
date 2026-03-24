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

## Blargg Test Status

| Teste                                    | Status          |
|------------------------------------------|-----------------|
| **cpu_instrs**                           | ✅ Passou       |
| 01-special                               | ✅ Passou       |
| 02-interrupts                            | ✅ Passou       |
| 03-op sp,hl                              | ✅ Passou       |
| 04-op r,imm                              | ✅ Passou       |
| 05-op rp                                 | ✅ Passou       |
| 06-ld r,r                                | ✅ Passou       |
| 07-jr,jp,call,ret,rst                    | ✅ Passou       |
| 08-misc instrs                           | ✅ Passou       |
| 09-op r,r                                | ✅ Passou       |
| 10-bit ops                               | ✅ Passou       |
| 11-op a,(hl)                             | ✅ Passou       |
| **halt_bug**                             | ✅ Passou       |
| **instr_timing**                         | ✅ Passou       |
| **interrupt_time**                       | ✅ Passou       |
| **mem_timing**                           | ✅ Passou       |
| 01-read_timing                           | ✅ Passou       |
| 02-write_timing                          | ✅ Passou       |
| 03-modify_timing                         | ✅ Passou       |
| **mem_timing-2**                         | ✅ Passou       |
| 01-read_timing                           | ✅ Passou       |
| 02-write_timing                          | ✅ Passou       |
| 03-modify_timing                         | ✅ Passou       |
| **oam_bug**                              | ❌ Falhou       |
| 1-lcd_sync                               | ✅ Passou       |
| 2-causes                                 | ✅ Passou       |
| 3-non_causes                             | ✅ Passou       |
| 4-scanline_timing                        | ✅ Passou       |
| 5-timing_bug                             | ✅ Passou       |
| 6-timing_no_bug                          | ✅ Passou       |
| 7-timing_effect                          | ⏱️ Timeout     |
| 8-instr_effect                           | ❌ Falhou       |
| **dmg_sound**                            | ❌ Falhou       |
| 01-registers                             | ✅ Passou       |
| 02-len ctr                               | ❌ Falhou       |
| 03-trigger                               | ❌ Falhou       |
| 04-sweep                                 | ❌ Falhou       |
| 05-sweep details                         | ❌ Falhou       |
| 06-overflow on trigger                   | ✅ Passou       |
| 07-len sweep period sync                 | ✅ Passou       |
| 08-len ctr during power                  | ❌ Falhou       |
| 09-wave read while on                    | ❌ Falhou       |
| 10-wave trigger while on                 | ❌ Falhou       |
| 11-regs after power                      | ❌ Falhou       |
| 12-wave write while on                   | ❌ Falhou       |
| **cgb_sound**                            | ❌ Falhou       |
| 01-registers                             | ✅ Passou       |
| 02-len ctr                               | ❌ Falhou       |
| 03-trigger                               | ❌ Falhou       |
| 04-sweep                                 | ❌ Falhou       |
| 05-sweep details                         | ❌ Falhou       |
| 06-overflow on trigger                   | ❌ Falhou       |
| 07-len sweep period sync                 | ✅ Passou       |
| 08-len ctr during power                  | ❌ Falhou       |
| 09-wave read while on                    | ❌ Falhou       |
| 10-wave trigger while on                 | ✅ Passou       |
| 11-regs after power                      | ✅ Passou       |
| 12-wave                                  | ❌ Falhou       |

## Resources

- [Game Boy Emulation Guide](https://aquova.net/emudev/gb/index.html)
- [Pandocs - Game Boy Technical Reference](https://gbdev.io/pandocs/) — comprehensive and up-to-date documentation on Game Boy hardware, instructions, quirks, and details.
