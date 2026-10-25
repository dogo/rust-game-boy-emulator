# Análise do `gb_emu`

_Data: 2026-10-25_

Em resumo: é um emulador de Game Boy clássico (DMG) muito preciso para um projeto de estudo. Tem cerca de 12,8k linhas de Rust e 248 commits, de 20/11/2025 até agora.

## O que foi verificado rodando

- **Blargg: 52/52 passam.** Inclui `cpu_instrs`, `instr_timing`, `mem_timing`, `oam_bug`, `dmg_sound` e `cgb_sound`.
- **Mooneye: 111/111 passam** nos grupos acceptance, emulator-only e misc. Os números do README batem com o resultado.
- **`cargo test`:** 70 testes unitários, todos passam.
- **Velocidade:** o `cpu_instrs` completo roda em 1,5s headless, umas 35x mais rápido que o hardware real.

Passar no Mooneye `acceptance/ppu` e no `oam_bug` inteiro é raro em emulador hobby. A maioria trava no timing do modo 3 ou nas IRQs de STAT.

## Pontos fortes

- **CPU por microcódigo** (`src/GB/microcode/`): cada opcode é uma lista de `MicroAction`, e cada acesso à memória avança o barramento em 4 T-cycles (`src/GB/bus.rs:661`). É por isso que os testes de timing de memória passam.
- **Detalhes de hardware bem cobertos:** HALT bug, corrupção de OAM, atraso de DMA, leitura e escrita de wave RAM no meio do ciclo, e registradores pós-boot para DMG0/MGB/SGB/CGB/AGB.
- **MBC1, 2, 3 (com RTC) e 5**, saves `.sav` e um debugger interativo.

## Problemas e dívidas

1. **CGB é praticamente inexistente.** O PPU só tem DMG; não há banco de VRAM, paletas CGB nem double-speed de verdade. Uma ROM só-CGB entra em `cgb_mode` (`src/GB/CPU.rs:39`) e roda num PPU que não suporta isso. ROMs com flag 0x80 rodam em modo DMG, o que está correto.
2. **Código legado sobrando.** O `src/GB/instructions/` hoje só serve para DI, EI, HALT e STOP, mas o `execute_next` ainda chama `decode()` em todo opcode só para pegar o nome (`src/GB/CPU.rs:294`), e o caminho de fallback do CB (`src/GB/CPU.rs:308`) nunca é executado. Dá para levar DI, EI, HALT e STOP para o microcódigo e apagar o módulo.
3. **Opcodes ilegais viram NOP** (`src/GB/instructions/mod.rs:28`). No hardware real (0xD3, 0xDB etc.) a CPU trava. Nenhum jogo comercial depende disso, mas é uma imprecisão.
4. **Seleção do modelo de boot pelo nome do arquivo** (`src/main.rs:6`): é um atalho para os testes do Mooneye. Funciona, mas vaza para o caminho normal de uso.
5. **`bus.tick` chama `apu.tick_m_cycle()` num loop** (`src/GB/bus.rs:640`). O próprio comentário reconhece isso. É um possível ganho de desempenho, mas hoje não faz falta.
6. **Clippy:** cerca de 35 avisos, todos cosméticos (`if` que dá para colapsar, falta de `Default` etc.). Nenhum é bug.
7. **Repositório:** `icon.bmp` tem 4,7 MB e está versionado. Um PNG teria uns 50 KB. As ROMs e os saves estão corretamente no `.gitignore`.

## O que fazer em seguida

Ou suporte a CGB, que é o salto grande de funcionalidade, ou limpar os itens 2 e 4. O núcleo DMG já está no nível dos melhores emuladores hobby.

## Limitações desta análise

Não foram testados os jogos de verdade com a interface SDL nem a qualidade do áudio audível; só as suítes headless. O APU e o PPU não foram lidos linha por linha.
