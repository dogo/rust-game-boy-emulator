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

## Desempenho observado durante a Q01

Sem janela, com renderização de linhas ligada, `--release`: 750–1.100 fps (12–18× a velocidade real) nas execuções curtas e 1.036 fps (17,3×) na sessão de uma hora. Medição formal na Q03.
