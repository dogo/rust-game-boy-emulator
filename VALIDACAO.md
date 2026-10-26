# Validação em jogos

Registro da etapa 4 do [ROADMAP.md](ROADMAP.md). Ausência de defeito registrado aqui não é declaração de compatibilidade: vale só para os cenários executados.

## Q01 — Jogos e cenários (25/10/2026)

ROMs fornecidas pelo usuário. Hashes registrados, não conferidos com o No-Intro.

| Jogo | Arquivo | SHA-1 | Cartucho | Modo |
|---|---|---|---|---|
| Legend of Zelda: Link's Awakening (Rev 2) | `Legend of Zelda, The - Link's Awakening (USA, Europe) (Rev 2).gb` | `5ab63def958728933571c3b4f6af54db14f3f8b2` | $03 MBC1+RAM+bateria, 512 KB, RAM 8 KB | DMG |
| Kirby's Pinball Land | `Kirby's Pinball Land (USA, Europe).gb` | `06efdb138ff56cd9522dece44adadd3fae169c76` | $06 MBC2+bateria, 256 KB | DMG |
| Pokémon Blue (SGB Enhanced) | `Pokemon - Blue Version (USA, Europe) (SGB Enhanced).gb` | `d7037c83e1ae5b39bde3c30787637ba1d4c48ce2` | $13 MBC3+RAM+bateria, 1 MB, RAM 32 KB | DMG (flag SGB ignorada) |
| Barbie: Ocean Discovery | `Barbie - Ocean Discovery (USA) (GB Compatible).gbc` | `fab988692a19d53e71fc754dea3a01506b104d9f` | $19 MBC5, 1 MB | DMG (CGB-compatível, flag $80) |

Execução: `examples/frame_capture.rs`, sem janela, modelo DMG pós-boot, com o roteiro de botões e esperas de cada linha e screenshots inspecionados visualmente. Saves lidos de cópias; nenhum `.sav` original foi gravado.

| Cenário | Zelda | Kirby | Pokémon | Barbie |
|---|---|---|---|---|
| Boot e telas iniciais | ✅ intro com sprite do barco | ✅ intro | ✅ intro Game Freak | ✅ telas legais e logotipos |
| Tela título e menus | ✅ | ✅ | ✅ CONTINUE/NEW GAME | ✅ New Game/Continue |
| Save existente carregado | ✅ arquivo 1 com nome e corações | não verificado (título só oferece START NEW GAME) | ✅ resumo do jogador | — sem RAM |
| Jogo em andamento | ✅ casa, Link e Tarin | ✅ mesa de pinball, placar | ✅ Pallet Town, jogador e NPC | não executado |
| Sobreposição de sprites | ✅ Link e Tarin | ✅ vários sprites na seleção de mesa | ✅ jogador e NPC | — |
| Placar/HUD (window) | ✅ HUD inferior | ✅ placar inferior | ✅ menu START e caixas de texto | — |
| Controles (roteiro) | ✅ andar nas 4 direções | ✅ START/A | ✅ andar, menus, A/B | ✅ START |
| Save pelo jogo e reabertura | não executado | não executado | ✅ SAVE no menu; ao reabrir, o jogador está na nova posição | — |
| Sessão prolongada | ✅ 217.188 frames (~1 h emulada) andando em quadrado; sem erro, diálogo renderizado no fim | não executado | não executado | não executado |

Defeitos observados: nenhum nos cenários acima. No Pokémon, o primeiro A após a tela título chega antes de o menu aceitar entrada; com o roteiro ajustado o jogo responde normalmente, então foi tratado como limite do roteiro, não do emulador.

Não coberto pela Q01: a interface SDL (entrada real, latência, VSync), o áudio e a comparação com referências de imagem — ver Q02. As cores dos screenshots são as quatro tonalidades de cinza do `sdl_runner`.


## Q02 — Imagem, áudio e latência (25/10/2026, parcial)

**Imagem.** [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) v1.0 (Matt Currie, MIT), executada com `examples/frame_capture` por 120 frames: o screenshot bate pixel a pixel com `img/reference-dmg.png` do projeto (0 de 23.040 pixels diferentes). Agora é teste automático (`tests/acid2_test.rs`, ROM e licença em `tests/fixtures/dmg-acid2/`); com o PPU de antes da etapa 2 (commit `215cae5`), 842 pixels diferiam.

**Afinação.** Medida nas amostras geradas a 44,1 kHz como no `sdl_runner` (`tests/apu_pitch_test.rs`), contra as fórmulas do Pan Docs:

| Canal | Fórmula | Antes | Depois |
|---|---|---|---|
| 1 e 2 (pulso) | 131072 / (2048 − x) Hz | período de N+1 M-cycles: x=2000 tocava 2675 Hz em vez de 2730,67 Hz (−2%; ~−11% em x=2040) | correto |
| 3 (onda) | 65536 / (2048 − x) Hz | correto | correto |
| 4 (ruído) | 262144 / (r × 2^s) ticks/s | 4× mais lento (duas oitavas abaixo): o divisor em T-cycles era contado em M-cycles | correto |

Amostras sem NaN nem valores fora de [−1, 1] nas medições. As 24 ROMs de som do Blargg continuam passando; elas testam registradores, comprimento, sweep e wave RAM, não a afinação.

**Escuta.** Com a correção, o usuário ouviu o jogo na interface SDL e não notou problemas. Pausa/retomada por F12 e os controles já tinham sido usados na sessão gráfica da etapa 1. Não houve medição objetiva de estalos nem de latência.

## Q03 — Desempenho (25/10/2026)

Máquina: AMD Ryzen 7 5800X3D (8 núcleos/16 threads, até 4,55 GHz), 31 GB, Linux 7.3.0 (CachyOS, Wayland), rustc 1.93.0, perfil `release` padrão do Cargo.

**Sem janela** (`examples/frame_capture`, renderização de linhas ligada, 3.600 frames, três execuções):

| Jogo | fps | × velocidade real |
|---|---|---|
| Zelda | 929–976 | 15,6–16,3 |
| Pokémon Blue | 884–923 | 14,8–15,4 |
| Kirby's Pinball Land | 846–859 | 14,2–14,4 |
| Barbie | 830–946 | 13,9–15,8 |

Cerca de 1 ms por frame, contra 16,7 ms de orçamento. No Zelda, retirar temporariamente os ticks do APU de `bus.tick` subiu para ~1.138 fps (o APU custa ~18% do tempo de emulação) e retirar a renderização subiu para ~1.000 fps (~6%). Como a emulação inteira ocupa ~6% de um núcleo em tempo real, o laço de `apu.tick_m_cycle()` não é um gargalo e não foi alterado. A suíte de 163 ROMs leva ~8 s.

**Com janela** (Zelda na intro, 20 s medidos em `/proc/<pid>/task/*/stat`):

| | Antes | Depois |
|---|---|---|
| Thread de render | 55% de um núcleo | 2% |
| Thread de emulação | 14% | 14% |
| Processo | 78% | 16,5% |

A hint de VSync usava o nome da macro (`"SDL_HINT_RENDER_VSYNC"`) em vez do valor `"SDL_RENDER_VSYNC"`, então o renderer nunca sincronizava e o laço de render girava sem parar. A hint `SDL_HINT_RENDER_SCALE_QUALITY`, do SDL2, não existe no SDL3, cujas texturas usam filtro linear por padrão: a imagem ampliada 3× saía borrada. Ambos corrigidos em `src/GB/sdl_runner.rs` (hint com o nome certo e `ScaleMode::Nearest` na textura). Os 14% da emulação são ~6% de trabalho e o restante a espera ativa do último ~1 ms de cada frame, mantida pela precisão do ritmo.

Ritmo e áudio numa sessão de 2 minutos: emulação a 59,7 FPS; buffer de áudio entre 31 e 83 ms, média por janela de 20 s entre 53 e 62 ms, sem deriva nem esvaziamento. O buffer não tem controle de taxa; sessões muito mais longas não foram medidas.
