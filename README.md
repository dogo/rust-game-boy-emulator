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
| 7-timing_effect                          | ⏱️ Timeout    |
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

## Resources

- [Game Boy Emulation Guide](https://aquova.net/emudev/gb/index.html)
- [Pandocs - Game Boy Technical Reference](https://gbdev.io/pandocs/) — comprehensive and up-to-date documentation on Game Boy hardware, instructions, quirks, and details.
