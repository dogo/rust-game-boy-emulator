# Roadmap do emulador Game Boy

Data da consolidação: 25/10/2026.

Objetivo: tornar o emulador DMG confiável para jogar, preservar progresso e facilitar manutenção. Manter a arquitetura de CPU por microcódigos, barramento e dispositivos; corrigir os comportamentos conhecidos antes de ampliar o hardware suportado.

Este plano reúne o [ANALISE.md](ANALISE.md) e a avaliação independente realizada nesta conversa. A segunda avaliação foi concluída antes da leitura da primeira. As tarefas concluídas estão marcadas abaixo; as regressões ignoradas representam bugs ainda abertos.

## Ponto de partida

| Verificação | Resultado reproduzido na avaliação independente |
|---|---|
| Testes Rust | 73 aprovados; 17 regressões conhecidas ignoradas |
| Blargg | 52/52 aprovados |
| Mooneye | 111/111 aprovados |
| Casos adicionais de regressão | 17 cenários registrados no projeto e ignorados até as correções correspondentes |
| Interface e áudio reais | Não validados em sessões longas |

Os 70 testes Rust citados na primeira análise passaram a 72 na segunda e a 73 após a etapa 0. O desempenho de aproximadamente 35 vezes o hardware e a quantidade de avisos do Clippy foram relatados na primeira análise, mas não medidos novamente. Não usar esses números como garantia atual.

As suítes aprovadas são uma base de regressão, não uma certificação de compatibilidade visual, sonora ou CGB. Alguns casos adicionais se referem ao mesmo problema; 17 cenários não significam 17 causas independentes.

## Ordem de execução

| Etapa | Prioridade | Resultado esperado |
|---|---|---|
| 0. Validação confiável | P0 | Cada execução testa o código atual e não aprova entrada inválida ou suíte vazia |
| 1. Saves e estabilidade da interface | P0 | Persistência protegida e sessões sem os erros conhecidos de pausa, buffers e controles |
| 2. Correção do hardware DMG | P1 | Gráficos, timer, DMA, cartuchos e STOP cobertos por regressões específicas |
| 3. Diagnóstico e limpeza | P2 | Debugger útil e remoção de código obsoleto sem perda de comportamento |
| 4. Validação de uso e desempenho | P2 | Evidência de funcionamento em jogos, vídeo e áudio reais |
| 5. Expansão CGB | P3, opcional | Suporte novo com escopo e testes próprios |

P0 deve preceder alterações grandes. As tarefas da etapa 2 podem ser feitas por subsistema; as tarefas de limpeza não devem atrasar correções de perda de dados ou de execução.

## Etapa 0 — Tornar a validação confiável

- [x] **V01 — Executar sempre o binário atualizado.**
  Em `run_all_tests.sh`, executar o build incremental de release antes das ROMs e interromper se a compilação falhar. Concluído quando uma alteração no código é necessariamente recompilada e um build inválido impede a suíte de começar.

- [x] **V02 — Eliminar aprovações falsas.**
  Em `src/main.rs`, retornar erro de processo para ROM inválida; no script, recusar suíte sem ROMs e distinguir falha, timeout e erro de execução. Verificar com ROM de header inválido, diretório vazio e uma ROM válida de cada protocolo.

- [x] **V03 — Incorporar as regressões encontradas.**
  Os 14 cenários de hardware e diagnóstico estão em `tests/known_regressions.rs`; os três de interface estão em `src/GB/sdl_runner.rs`. Foram confirmados falhando quando executados com `--ignored`. Os cenários ainda não corrigidos permanecem ignorados na suíte normal; retirar a marcação ao corrigir cada bug.

- [x] **V04 — Executar ROMs no CI.**
  Atualizar `.github/workflows/ci.yml` para preparar os submódulos e ROMs necessários, executar testes Rust e as suítes Blargg/Mooneye e preservar os diagnósticos das falhas. Concluído quando regressão de ROM reprova o CI e ausência de fixtures não resulta em sucesso.

Critério de saída: testes Rust e 163 ROMs existentes continuam aprovados, com build atual e contagem explícita. Mudanças justificadas na seleção de ROMs devem atualizar a documentação e a contagem esperada.

Etapa 0 implementada e validada: 17 regressões conhecidas foram reproduzidas separadamente e as 163 ROMs passaram. O script rejeitou build com falha e suíte sem fixtures em verificações isoladas. O [CI hospedado](https://github.com/dogo/rust-game-boy-emulator/actions/runs/40236503676/attempts/2) confirmou a instalação SDL3, os testes Rust e as suítes Blargg/Mooneye.

## Etapa 1 — Proteger progresso e estabilizar a interface

- [x] **S01 — Substituir saves somente após escrita bem-sucedida.**
  Em `src/GB/bus.rs` e nos consumidores em `src/main.rs`, gravar em temporário no mesmo diretório e substituir o destino após concluir. Manter o save anterior se a escrita falhar e informar o erro. Testar ida e volta e falha antes da substituição; definir a necessidade de sincronização em disco se houver requisito de resistência a queda de energia.

  Concluído: o temporário é exclusivo, fica no mesmo diretório e é removido se a gravação ou a troca falhar. O save anterior e suas permissões são preservados; links simbólicos existentes continuam apontando para o mesmo destino. Uma falha de save agora encerra o processo com erro. Testes cobrem criação, substituição, recarga e falhas; o formato `.sav` não mudou. A troca não garante durabilidade após queda de energia porque arquivo e diretório não são sincronizados em disco.

- [x] **S02 — Preservar RTC com HALT e sem RAM.**
  Em `src/GB/mbc/mbc3.rs`, respeitar HALT ao aplicar tempo offline e salvar o relógio mesmo quando a RAM está vazia. Cobrir save parado por 60 segundos, relógio ativo e cartucho com RTC+bateria sem RAM; preservar a leitura dos formatos antigos já suportados.

  Concluído: o tempo offline só avança o RTC ativo; cartuchos MBC3 com RTC e bateria gravam os 5 registradores e o timestamp mesmo sem RAM. As regressões S02 estão ativas. Testes cobrem relógio parado e ativo após 60 segundos, ida e volta sem RAM e leitura do formato antigo sem timestamp. Blargg e Mooneye passaram (163/163). O formato `.sav` existente não mudou; validação de dados inválidos continua em S03.

- [x] **S03 — Tratar estado RTC inválido.**
  Validar os dados do save antes de aceitá-los e definir o comportamento dos valores fora do intervalo escritos pela ROM. Eliminar underflow em `60 - sec` e testar segundos=60/61, dados truncados e timestamps extremos sem esperas reais. Erro de carga deve ser comunicado e não levar à substituição silenciosa do save original.

  Concluído: saves MBC3 aceitam os formatos RAM, RAM+RTC e RAM+RTC+timestamp completos; registradores fora dos [intervalos documentados](https://github.com/gbdev/pandocs/blob/master/src/MBC3.md) e timestamps não positivos são rejeitados antes de alterar o estado. Timestamps futuros são tratados como ajuste do relógio do host, sem avanço offline. Por política do emulador, escritas fora da faixa em segundos, minutos e horas usam módulo 60/60/24; essa escolha ainda não foi comparada com hardware real. O avanço do RTC agora é calculado sem laço por minuto, inclusive para timestamps antigos. Uma carga inválida encerra o processo com erro e preserva o save. `MBC::load_ram` passou a retornar `Result<(), String>` e `MemoryBus::load_cart_ram` passou a retornar `io::Result<()>`; os bytes do formato `.sav` não mudaram. A suíte Rust e as 163 ROMs Blargg/Mooneye passaram.

- [x] **I01 — Corrigir parada e retomada por breakpoint.**
  Em `src/GB/sdl_runner.rs` e `src/GB/debugger.rs`, preservar os ciclos de frame parcial, comunicar a pausa à interface e entrar no mesmo fluxo de interação usado por F12. Testar breakpoint no primeiro opcode, no meio do frame, step, continue e encerramento durante a pausa em debug e release.

  Concluído: um breakpoint encerra o frame parcial sem descontar `CYCLES_PER_FRAME`; os ciclos acumulados continuam após a retomada. A thread principal verifica `debug_requested` a cada iteração, de modo que F12 e breakpoint usam o mesmo fluxo; ela só volta ao loop depois que a emulação limpa a pausa. Após `continue`, a instrução no PC atual executa mesmo que tenha breakpoint, como no GDB. `Debugger::debug_command_loop` recebe `running: &AtomicBool` e sai quando a interface encerra durante a pausa. As regressões I01 estão ativas e passam em debug e release; a suíte Rust e as 163 ROMs Blargg/Mooneye passaram. Sessão SDL manual com Zelda: F12, step, continue e saída funcionaram e o save foi gravado. Na retomada, o áudio volta a ter 80 ms de folga, como na inicialização (antes eram ~46 ms e o buffer ficava perto de zero). Sem terminal (EOF no stdin), o debugger retoma a emulação em vez de repetir o prompt indefinidamente. Enquanto o terminal do debugger está aberto, os eventos da janela não são processados.

- [x] **I02 — Simplificar a entrega de frames entre threads.**
  Corrigir a intercalação entre leitura e escrita de `ready_idx`. Preferir um buffer do frame mais recente protegido por `Mutex`, suficiente para a imagem atual, se mantiver o comportamento necessário. Validar publicação concorrente e ausência de frame parcialmente copiado; não adicionar uma dependência só para preservar o triple buffer.

  Concluído: `TripleBuffer` foi substituído por `LatestFrame`, um único frame protegido por `Mutex` com a marcação de frame novo; a cópia na publicação e na leitura acontece sob o lock, então não há índices para intercalar nem frame parcial. A interface continua recebendo só o frame mais recente, uma vez. O teste antigo dependia dos índices internos e foi trocado por dois testes de comportamento: entrega única com o mais novo prevalecendo e 200 publicações concorrentes sem frame misto ou fora de ordem. Nenhuma dependência nova; a suíte Rust passou em debug e release.

- [x] **I03 — Preservar a ordem ou o estado final dos botões.**
  Em `src/GB/sdl_runner.rs`, substituir as máscaras independentes de pressionar/soltar por representação que preserve o resultado dos eventos, mantendo os pedidos de interrupção e a retomada de STOP. Testar pressionar→soltar, soltar→pressionar entre frames, múltiplos botões e tecla mantida.

  Concluído: a interface mantém o estado final dos botões (`joypad_held`) e uma máscara dos pressionados desde o último frame (`joypad_pressed`). A cada frame, a emulação aplica só os botões que mudaram em relação ao `Joypad`, primeiro as solturas e depois os pressionamentos, para que a borda usada pelo STOP seja a última atualização. Um toque mais curto que um frame fica pressionado por um frame e solta no seguinte; soltar→pressionar no mesmo frame mantém o botão pressionado. A interrupção continua sendo pedida só na transição solto→pressionado. Testes cobrem os quatro cenários e a retomada de STOP com soltura simultânea (confirmado falhando com a ordem invertida). A suíte Rust e as 163 ROMs passaram. `Joypad` não mudou; a pendência de `has_new_press` permanecer verdadeira até a próxima mudança de entrada fica para D09.

Critério de saída: os cenários acima têm regressões automatizadas; uma sessão gráfica permite pausar, inspecionar, continuar e sair com save íntegro.

## Etapa 2 — Corrigir a emulação DMG

- [x] **D01 — Resolver corretamente prioridade entre sprites e fundo.**
  Em `src/GB/PPU.rs`, escolher o pixel vencedor entre sprites antes de aplicar a prioridade do background. Cobrir menor X, empate por ordem de OAM, transparência e sprite prioritário oculto pelo fundo. Apenas inverter a ordem do loop não resolve todos os casos.

  Concluído: em cada pixel, o primeiro sprite opaco na ordem DMG (menor X, depois menor índice de OAM) fica com o pixel; só então a flag de prioridade do fundo desse sprite é aplicada. Um sprite prioritário escondido pelo fundo continua escondendo os sprites abaixo dele, como descrito no [Pan Docs](https://gbdev.io/pandocs/OAM.html#drawing-priority). Os sprites não alteram mais `bg_priority`, o que também atende à parte de D02 sobre preservar a informação do fundo; a limpeza com BG desligado continua em D02. Testes com uma cor por sprite cobrem menor X, empate por OAM, transparência e sprite oculto pelo fundo; os quatro falham no código anterior. A suíte Rust e as 163 ROMs passaram. A seleção dos 10 sprites por linha continua por ordem de OAM, sem mudança.

- [x] **D02 — Limpar a máscara quando o background é desabilitado.**
  Limpar `bg_priority` junto com a linha e impedir que o desenho dos sprites destrua a informação original do fundo. Testar um sprite de baixa prioridade após desligar o BG, inclusive com sobreposição de sprites; coordenar com D01.

  Concluído: com o BG desligado, `render_bg_scanline` limpa `bg_priority` junto com a linha, então nenhum sprite fica escondido pela máscara de um frame anterior. A parte de não deixar os sprites destruírem a informação do fundo já tinha entrado em D01. Testes cobrem um sprite de baixa prioridade com o BG desligado e o mesmo caso com outro sprite sobreposto (o de baixa prioridade vence e aparece); os dois falham no código anterior. A suíte Rust e as 163 ROMs passaram.

- [x] **D03 — Aplicar bloqueios de VRAM/OAM na linha zero.**
  Restringir a exceção de ligar o LCD ao período em que ela realmente está ativa. Revisar os quatro caminhos de bloqueio de leitura/escrita. Cobrir a primeira linha após ligar o LCD e a linha zero de frames posteriores, com LCD ligado/desligado e modos 2/3.

  Concluído: os quatro caminhos (`cpu_vram_blocked`, `cpu_vram_write_blocked`, `cpu_oam_blocked`, `cpu_oam_write_blocked`) só usam as janelas especiais da linha 0 enquanto `lcd_on_timing_quirk` está ativo, isto é, na primeira linha após ligar o LCD. Nos frames seguintes, a linha 0 segue as regras normais dos modos 2 e 3. Testes cobrem a linha 0 de um frame posterior nos modos 2 e 3 (falha no código anterior), a primeira linha após ligar o LCD e o LCD desligado (sem bloqueio). A suíte Rust e as 163 ROMs passaram, incluindo os testes Mooneye de `lcdon_timing`.

- [x] **D04 — Corrigir recorte e contador vertical da window.**
  Aceitar a janela parcialmente visível com `WX < 7` e usar o contador de linhas efetivamente desenhadas, em vez de apenas `LY-WY`. Testar WX=6/7, ocultação e retomada via WX, condições de WY e reset por frame; tratar peculiaridades dos limites conforme evidência de hardware.

  Concluído: `wy_pos` passou a ser o contador interno de linhas da window e só avança nas linhas em que ela é desenhada; ocultar via WX > 166 ou desligar o bit 5 de LCDC no meio do frame pausa o contador, e a retomada continua da próxima linha. A condição de WY trava quando `LY == WY` em qualquer momento do frame, mesmo com a window desligada, e só é limpa no VBlank, como descrito no [Pan Docs](https://gbdev.io/pandocs/Scrolling.html#window). Com `WX < 7`, a window começa cortada à esquerda. Os resets ao desligar a window foram removidos. Testes cobrem WX=0/6/7, ocultação e retomada via WX, pausa com a window desligada, WY alterado depois do disparo, WY nunca igualado e o reset no VBlank; os novos falham no código anterior. A suíte Rust e as 163 ROMs passaram. Não modelados, por falta de evidência e do renderizador ser por linha: as peculiaridades de WX=0 com SCX, de WX=166 e mudanças de WX no meio da linha. Ainda não verificado visualmente em jogo.

- [x] **D05 — Completar reload de TIMA mesmo após desabilitar o timer.**
  Em `src/GB/timer.rs`, processar o reload pendente independentemente de TAC. Cobrir overflow seguido de TAC=0, atraso de quatro T-cycles, interrupção e escritas em TIMA/TMA durante o atraso. Preservar os testes Mooneye já aprovados.

  Concluído: o reload pendente e os prazos ligados a ele (`tima_reloaded_until`) são processados por T-cycle mesmo com TAC=0; só a detecção de borda depende de TAC. Sem reload pendente e com o timer desligado, o laço por T-cycle continua sendo pulado. Isso revelou um erro que o comportamento antigo escondia: os estouros causados em `write_tac` agendavam o reload em `+4`, mas o barramento ainda avança os 4 T-cycles do M-cycle da escrita, então o reload caía no próprio M-cycle. Eles passaram a usar `+8`, como `reset_div`; com `+4`, o Mooneye `rapid_toggle` interrompia uma iteração antes (BC=FFDA em vez de FFD9). Testes em `tests/timer_test.rs` cobrem o overflow seguido de TAC=0 (sem reload em 3 T-cycles, reload e interrupção no 4º), TMA e TIMA escritos durante o atraso com o timer desligado, o estouro pelo glitch de desligar e o timer desligado sem contagem. A suíte Rust e as 163 ROMs passaram.

- [x] **D06 — Modelar conflitos de barramento durante OAM DMA.**
  Em `src/GB/bus.rs`, aplicar as restrições aos acessos da CPU, separando-os dos acessos internos do DMA. Cobrir ROM, WRAM, HRAM, início, término e reinício da transferência. Confirmar os resultados exatos dos conflitos no modelo DMG; não generalizar as regras para CGB.

  Concluído: `cpu_read`/`cpu_write` passam por `oam_dma_conflict`; os acessos internos do DMA e `read`/`write` diretos (inicialização, debugger) não mudaram. No DMG, há conflito quando o DMA já copiou o primeiro byte e a CPU acessa abaixo de `$FE00` o mesmo barramento da origem (principal: ROM, SRAM, WRAM; ou VRAM). A leitura retorna o byte que o DMA acabou de copiar. A escrita com origem em SRAM/WRAM se perde e faz AND no byte recém-copiado da OAM; com origem em ROM/VRAM, vai para o endereço que o DMA leu (em ROM, isso escreve nos registradores do MBC). O [Pan Docs](https://gbdev.io/pandocs/OAM_DMA_Transfer.html) só diz que a CPU acessa apenas a HRAM; os resultados exatos seguem o SameBoy (`is_addr_in_dma_use` em `Core/memory.c`), sem comparação própria com hardware. CGB (`cgb_mode` ou `cgb_hwio_enabled`) fica sem conflitos. Testes em `tests/dma_test.rs` cobrem WRAM, ROM, VRAM, HRAM, escrita perdida com AND na OAM, escrita redirecionada ao MBC, início, término, reinício e modo CGB; seis falham no código anterior. A suíte Rust e as 163 ROMs passaram.

- [x] **D07 — Ler os dois grupos do joypad corretamente.**
  Em `src/GB/joypad.rs`, combinar as linhas quando os dois grupos estiverem selecionados. Testar nenhum grupo, cada grupo isolado e ambos, com combinações de direção e ação; preservar a leitura ativa em zero.

  Concluído: `Joypad::read` faz AND das linhas de cada grupo selecionado (ativo em zero); antes, com os dois selecionados, só o D-pad aparecia. Um teste cobre nenhum grupo, D-pad, ação e ambos, com botões de bits iguais e diferentes nos dois grupos; ele e a regressão D07 falham no código anterior. A suíte Rust e as 163 ROMs passaram. A interrupção e a saída de STOP continuam usando o estado dos botões, sem depender da seleção.

- [x] **D08 — Tratar cartuchos sem controlador com RAM e tipos desconhecidos.**
  Em `src/GB/mbc/mod.rs`, `none.rs` e seus consumidores, dar suporte aos tipos 0x08/0x09 ou rejeitá-los explicitamente até a implementação. Recusar controladores desconhecidos em vez de cair silenciosamente em ROM simples. Para declarar suporte, testar leitura/escrita da RAM e persistência do tipo com bateria.

  Concluído: os tipos 0x08 (ROM+RAM) e 0x09 (ROM+RAM+bateria) usam `NoMBC::with_ram`, com até 8 KB em `$A000-$BFFF` e o tamanho do header ($0149); acima do tamanho declarado, leituras retornam 0xFF. Só o 0x09 grava `.sav`. `validate_header` recusa tipos fora de `mbc::is_supported_cart_type` e o processo sai com erro; `create_mbc` ainda cai em ROM simples para ROMs sem header, como as dos testes. Testes cobrem RAM sem bateria (sem save), ida e volta do save com bateria, RAM menor que a janela, a lista de tipos suportados e a recusa pela linha de comando. A suíte Rust e as 163 ROMs passaram; os quatro jogos locais usam tipos suportados. API nova: `mbc::is_supported_cart_type` e `NoMBC::with_ram`; o formato `.sav` dos outros tipos não mudou.

- [x] **D09 — Separar STOP de HALT.**
  Em `src/GB/CPU.rs`, não avançar o divisor como se a CPU estivesse em HALT. Cobrir divisor parado, entrada/saída de STOP e retomada por joypad; verificar também o consumo do byte de padding da instrução. Manter as regras de troca de velocidade CGB separadas do caminho DMG.

  Concluído: em STOP nada avança (timer, DIV, PPU e APU ficam parados) até uma linha P10-P13 ir a zero, ou seja, um botão pressionado em um grupo selecionado; antes, `bus.tick(4)` mantinha tudo rodando como em HALT. A entrada segue o [fluxograma de STOP do Pan Docs](https://gbdev.io/pandocs/Reducing_Power_Consumption.html) para o DMG: sem botão pressionado, entra em STOP e zera o DIV; com botão, vira HALT (sem interrupção pendente) ou NOP; o byte de padding só é pulado sem interrupção pendente. A troca de velocidade CGB continua no caminho anterior. `Joypad::has_new_press` e `prev_state` foram removidos, o que resolve a pendência deixada em I03; `MemoryBus::joypad_should_wake_from_stop` virou `joypad_line_low`, e o runner não precisa mais aplicar solturas antes dos pressionamentos. Testes em `tests/cpu_control_test.rs` cobrem os quatro caminhos do fluxograma, DIV e LY parados durante STOP, retomada por joypad e botão de grupo não selecionado; os cinco e a regressão D09 falham no código anterior. A suíte Rust e as 163 ROMs passaram. O fluxograma é uma imagem (`imgs/stop_diagram.svg`); os quatro resultados do DMG foram conferidos com os rótulos do SVG, sem teste em hardware. Os ramos de troca de velocidade e o glitch com IME não se aplicam ao DMG e não foram modelados.

- [x] **D10 — Representar o travamento dos opcodes ilegais.**
  A primeira análise identificou que eles são tratados como NOP. Confirmar o comportamento do modelo alvo e implementar estado de travamento sem confundir com HALT, STOP ou opcode desconhecido. Cobrir os opcodes ilegais e demonstrar que a CPU não continua executando instruções seguintes.

  Concluído: os 11 opcodes ilegais ($D3, $DB, $DD, $E3, $E4, $EB, $EC, $ED, $F4, $FC, $FD) marcam `CPU::locked`; a partir daí a CPU não busca mais instruções e nem interrupções a destravam, como o [Pan Docs](https://gbdev.io/pandocs/CPU_Instruction_Set.html) descreve ("hard-lock the CPU until the console is powered off"). O estado é separado de `halted`, `stopped` e do retorno `unknown` de `execute_next`. PPU e timer continuam rodando; o Pan Docs não fala do resto do sistema, então isso é uma suposição. Um teste percorre os 11 opcodes com IME e todas as interrupções pendentes e confirma PC parado, instrução seguinte não executada e timer avançando; ele falha sem o travamento. A suíte Rust e as 163 ROMs passaram. Campo novo público: `CPU::locked`; o debugger ainda não o exibe.

Critério de saída: regressões de cada tarefa passam, assim como as suítes existentes. Para gráficos, adicionar comparações de pixels que distingam os objetos envolvidos; testes com tiles de cores iguais não detectam inversão de prioridade.

## Etapa 3 — Melhorar diagnóstico e limpar código

- [x] **L01 — Usar os nomes das instruções executadas no debugger e no trace.**
  Revisar `src/GB/debugger.rs`, `trace.rs` e os metadados de microcódigo. Uma instrução implementada como `3E 42` deve aparecer como `LD A,d8`, com seu operando, e prefixos CB precisam do nome correto. Cobrir também tamanhos e avanço do disassembly.

  Concluído: `debugger::instruction_name` usa a mesma prioridade da CPU (tabela CB, microcódigo, decodificação legada) e é usado no disassembly, no estado após `step` e no trace; opcodes ilegais aparecem como `ILLEGAL`. A lista deles virou `instructions::ILLEGAL_OPCODES`, usada também pela CPU. STOP passou a ter 2 bytes no disassembly, e os segundos bytes de CB e STOP não aparecem mais como operando. Testes em `tests/debugger_test.rs` cobrem nome para todos os 256 opcodes e 256 CB, nomes de exemplo e tamanhos/avanço do disassembly; a regressão L01 está ativa. A suíte Rust (agora sem testes ignorados) e as 163 ROMs passaram. Os nomes vêm dos metadados existentes do microcódigo, com placeholders como `d8`/`a16` seguidos do valor.

- [x] **L02 — Remover o barramento legado sem consumidores internos.**
  Confirmar referências de `src/GB/RAM.rs`, inclusive export em `src/GB.rs`, testes e eventuais consumidores externos. Remover módulo e export se a API não for necessária. A remoção estimada de 148 linhas só vale após essa confirmação; manter `MemoryBus` como implementação efetiva.

  Concluído: a única referência era o export `pub mod RAM` em `src/GB.rs`; nada em `src`, testes, scripts, CI ou documentação usava o tipo, e o crate não é publicado nem tem dependentes. Módulo (149 linhas, incluindo um `save_cart_ram` sem a escrita segura de S01) e export removidos; `MemoryBus` segue como barramento. A API pública perde `GB::RAM`. A suíte Rust e as 163 ROMs passaram.

- [x] **L03 — Reduzir o caminho legado de instruções.**
  Depois de L01, revisar a necessidade de `instructions::decode()` em toda instrução e do fallback CB. Remover caminhos comprovadamente inalcançáveis. Migrar DI/EI/HALT/STOP somente se isso simplificar o conjunto, preservando seus efeitos e temporização; não apagar o módulo antes de atualizar todos os consumidores e testes.

  Concluído: a tabela CB cobre os 256 opcodes, então o fallback CB era inalcançável; e as funções legadas de DI/EI/HALT/STOP/ilegais só retornavam os 4 ciclos do fetch, sem efeito, porque os efeitos reais já estavam no `match` de `CPU::execute_next`. Foram removidos `CPU::decode`, `instructions::decode`, `Instruction`, `FlagBits`, `control.rs` e `helpers.rs`; `instructions` ficou com `ILLEGAL_OPCODES` e `control_name`, usado pelo debugger. A migração de DI/EI/HALT/STOP para microcódigo ficou para L06. O retorno `unknown` de `execute_next`, sempre falso, saiu: a função retorna só os ciclos (API pública alterada); debugger (`run`) e trace agora param quando a CPU trava num opcode ilegal. Testes novos confirmam que cada opcode tem microcódigo ou é instrução de controle, que a tabela CB é completa e que as instruções de controle gastam 4 ciclos. A suíte Rust e as 163 ROMs passaram.

- [x] **L04 — Separar seleção de boot dos nomes de arquivo de teste.**
  Em `src/main.rs` e `run_all_tests.sh`, manter a escolha de modelo por nome restrita à infraestrutura de testes, usando seleção explícita quando necessário. Renomear uma ROM de jogo não deve mudar o hardware emulado; as variantes Mooneye devem continuar recebendo o modelo esperado.

  Concluído: `infer_boot_model` saiu de `src/main.rs`; o emulador recebe `--model <dmg0|dmg|mgb|sgb|sgb2|cgb0|cgb|agb>` (`BootModel::from_name`) e, sem ela, usa DMG, ou CGB para ROMs só CGB. Modelo inválido encerra com erro. As convenções de nome das ROMs de teste (`-dmg0`, `-mgb`, `-S`, `-A` etc.) foram para `boot_model_for` em `run_all_tests.sh`, com as mesmas regras e ordem, e o script passa `--model`. Um teste de linha de comando usa uma ROM que só passa com os registradores pós-boot do DMG: chamada `game-mgb.gb` continua DMG (falha no código anterior), `--model mgb` muda o hardware e um modelo inválido é recusado. A suíte Rust e as 163 ROMs passaram; `boot_regs-mgb` falha sem `--model mgb` e passa com ele. O README ainda não documenta a opção (Q04).

- [x] **L05 — Tratar avisos e reduzir o ícone sem ampliar dependências.**
  Medir os avisos atuais do Clippy e corrigir os que melhorarem clareza ou revelarem erro, sem refatoração mecânica ampla. Verificar dimensões e tamanho de `icon.bmp`; reduzir o bitmap se possível. Só mudar o formato se o carregamento existente o suportar sem adicionar infraestrutura desnecessária.

  Concluído: `cargo clippy --all-targets` (Rust 1.94) tinha 47 avisos distintos (39 na lib, 7 nos testes, 1 no binário); nenhum revelava erro. Corrigidos os que melhoram a clareza: casts redundantes no `ADD SP,r8` e no runner, `div_ceil`/`contains` no dump de memória, iteração direta no checksum do header e o comentário de módulo de `timer.rs`, que estava com `///` e se anexava ao item seguinte. Restam 40, por escolha: `if` colapsáveis, `Default` para tipos com `new()`, padrões com faixa, `is_multiple_of`, laços por índice e `rom[1 * 0x4000]` nos testes de MBC (o `1 *` indica o banco). `icon.bmp` foi de 1254×1254 (4,7 MB) para 128×128 em BMP de 24 bits (49 KB), sem mudar o formato nem o carregamento; `Surface::load_bmp` o carrega. A suíte Rust e as 163 ROMs passaram.

- [x] **L06 — Migrar DI, EI, HALT e STOP para o microcódigo.**
  Hoje são as únicas instruções sem `MicroProgram`: gastam só o fetch e seus efeitos ficam no `match` de `CPU::execute_next` (IME, `ime_enable_next`, HALT bug, fluxograma de STOP, troca de velocidade CGB). Criar microprogramas e as `MicroAction`s necessárias para que esses efeitos saiam do `match`, mantendo o atraso de uma instrução do EI, o HALT bug, os quatro caminhos de STOP do DMG e a troca de velocidade CGB. Depois, avaliar se `instructions::control_name` ainda é necessário (os nomes passariam a vir do microcódigo) e se os opcodes ilegais também devem virar microprogramas que travam a CPU. Cobrir com os testes de `cpu_control_test.rs` e as 163 ROMs, que dependem da temporização dessas instruções.

  Concluído: `MicroAction::Control(CpuControl)` declara o efeito de controle e `microcode::execute` o devolve; `CPU::execute_next` o aplica, em vez de decidir pelo opcode. DI, EI, HALT, STOP e os 11 opcodes ilegais ganharam microprogramas em `microcode/control.rs` (com `ILLEGAL_OPCODES`), e o RETI declara `EnableInterruptsNow` em vez de ter o IME tratado à parte. A lógica de estado continua na CPU (atraso do EI, HALT bug, fluxograma de STOP, troca de velocidade CGB), porque depende de campos que o microcódigo não acessa. Todo opcode agora tem microprograma, então o módulo `instructions` e `control_name` foram removidos e o debugger tira todos os nomes do microcódigo. API: `microcode::execute` retorna `Option<CpuControl>`; `GB::instructions` deixou de existir. Testes novos cobrem o atraso do EI, o RETI e o HALT bug, além da cobertura completa de microprogramas; a suíte Rust e as 163 ROMs passaram, incluindo as 11 de temporização de DI/EI/HALT/RETI do Mooneye.

Critério de saída: debugger e trace descrevem instruções reais, não existem referências quebradas às remoções e nenhuma limpeza altera o comportamento das suítes aprovadas.

## Etapa 4 — Validar uso real antes de otimizar

- [x] **Q01 — Registrar uma matriz pequena de jogos e cenários.**
  Usar ROMs disponíveis legalmente e identificar jogo, versão/hash, controlador e modo. Cobrir boot, menus, sobreposição de sprites, placar/window, controles, save/reabertura e uma sessão prolongada. Registrar defeitos observados sem converter ausência de teste em declaração de compatibilidade.

  Concluído: matriz em [VALIDACAO.md](VALIDACAO.md) com quatro jogos (MBC1, MBC2, MBC3 e MBC5, um deles CGB-compatível no caminho DMG), hash, cartucho e modo. `examples/frame_capture.rs` roda a ROM sem janela com um roteiro de botões e grava screenshots e saves no diretório de saída, sem tocar nos `.sav` originais. Boot, menus, sprites sobrepostos, HUD/window e controles por roteiro funcionaram nos cenários executados; o save do Pokémon foi gravado pelo jogo e reaberto com a nova posição; uma hora emulada de Zelda rodou sem erro. Nenhum defeito observado. Lacunas registradas no próprio documento: células não executadas, save do Kirby não verificado e tudo o que depende da interface SDL e do áudio (Q02).

- [x] **Q02 — Verificar imagem, áudio e latência na interface.**
  Comparar screenshots com referências confiáveis e ouvir os quatro canais, transições, pausa e retomada. Observar estalos, afinação, falta/excesso de amostras e resposta dos controles. Transformar divergências reproduzíveis em testes pequenos antes de alterar a APU.

  Concluído (ver [VALIDACAO.md](VALIDACAO.md)): dmg-acid2 bate pixel a pixel com a referência de hardware. A medição de afinação revelou dois defeitos, agora cobertos por `tests/apu_pitch_test.rs` e corrigidos em `src/GB/APU.rs`: os canais de pulso tinham período de N+1 M-cycles (tom até ~11% baixo nas notas agudas) e o ruído rodava 4× mais lento, porque o divisor do Pan Docs, em T-cycles, era contado em M-cycles. O canal de onda já estava certo. Com a correção, o usuário ouviu o jogo na interface sem notar problemas; estalos e latência não foram medidos objetivamente.

- [x] **Q03 — Medir os caminhos de execução.**
  Registrar máquina, perfil de build, ROM e tempo de execução para headless e uso gráfico. Medir consumo de CPU, ritmo de frames e buffer de áudio. Investigar o loop de `apu.tick_m_cycle()` em `bus.tick` apenas se ele aparecer como custo relevante; processamento em lote deve preservar a ordem dos eventos e resultados de áudio.

  Concluído (ver [VALIDACAO.md](VALIDACAO.md)): sem janela, os quatro jogos rodam a 14–16× a velocidade real (~1 ms por frame). O APU responde por ~18% do tempo de emulação, mas a emulação inteira ocupa ~6% de um núcleo em tempo real, então o laço de `apu.tick_m_cycle()` não foi alterado. Com janela, o processo usava 78% de um núcleo porque a hint de VSync tinha o nome errado (`"SDL_HINT_RENDER_VSYNC"` em vez de `"SDL_RENDER_VSYNC"`) e o laço de render nunca bloqueava; com a correção, 16,5%. A hint de escala do SDL2 também não existia no SDL3, que filtra texturas em linear; a textura agora usa `ScaleMode::Nearest`. Em 2 minutos, 59,7 FPS e buffer de áudio entre 31 e 83 ms sem deriva.

- [x] **Q04 — Atualizar a documentação com o que foi validado.**
  Atualizar README com comandos, controles, limites DMG/CGB, suítes e resultados atuais. Distinguir testes de registradores/timing de compatibilidade visual e sonora. Manter contagens de testes sincronizadas com as execuções reais.

  Concluído: o README passou a ter compilação e execução (opções `--headless`, `--trace`, `--model`, boot ROM opcional e `examples/frame_capture`), controles, comandos do debugger, comportamento dos saves, tipos de cartucho aceitos e limites (PPU só DMG, ROMs só CGB sem suporte, SGB sem bordas/paletas, desenho por linha). A seção de testes traz as contagens desta execução (157 testes Rust sem ignorados, Blargg 52/52, Mooneye 111/111) e separa o que as suítes de registradores/timing verificam do que dmg-acid2, a afinação do APU e o [VALIDACAO.md](VALIDACAO.md) mostram. O parágrafo antigo sobre 14 regressões ignoradas saiu. Durante a revisão, a ROM do dmg-acid2 se mostrou fora do commit `45a11d7` porque o `.gitignore` excluía `*.gb`; a exceção `!tests/fixtures/**/*.gb` e a ROM entraram em `2465c3b`, verificado num clone limpo.

Critério de saída: há evidência de uso gráfico e sonoro e uma medição reproduzível. Otimizações só entram quando corrigem um gargalo medido e mantêm a precisão.

## Etapa 5 — CGB como expansão opcional

O PPU atual é DMG. Aprovar testes CGB de áudio ou snapshots de boot não demonstra suporte a jogos CGB. Até existir suporte suficiente, informar claramente a limitação e recusar execução normal de ROMs exclusivamente CGB; preservar os caminhos necessários aos testes de hardware.

- [x] **C01 — Definir e comunicar os modos suportados.**
  Manter ROMs compatíveis com flag 0x80 no caminho DMG enquanto necessário e diferenciar ROMs exclusivamente CGB. Testar a política de aceitação e documentá-la antes de anunciar suporte.

  Concluído: ROMs exclusivas de CGB (flag `$C0`) são recusadas com erro quando não há `--model`; com `--model cgb` elas rodam como antes, sem gráficos CGB. ROMs CGB-compatíveis (`$80`) continuam no caminho DMG sem aviso. O modelo padrão passou a ser sempre `dmg`. As 13 ROMs de teste só CGB (12 de `cgb_sound` e `interrupt_time`), que não têm sufixo de modelo no nome, recebem `--model cgb` pelo `boot_model_for` de `run_all_tests.sh`, que lê a flag do header. Um teste de linha de comando cobre a recusa, a execução com `--model cgb` e a ROM `$80` sem `--model`; a recusa falha no código anterior. README e ajuda do `--model` atualizados. A suíte Rust e as 163 ROMs passaram.

  Revisto em 26/10/2026: com os gráficos CGB (`74dd96c`), ROMs só CGB passaram a abrir em `cgb` quando não há `--model`, a mesma regra do `CPU::new`; um modelo não CGB explícito continua recusado. A leitura da flag no `boot_model_for` de `run_all_tests.sh` ficou redundante e saiu.

- [ ] **C02 — Implementar memória, atributos e paletas CGB.**
  Planejar bancos de VRAM/WRAM, registradores de seleção, atributos de tiles, paletas e regras de prioridade. Cada recurso deve ter teste específico e preservar o funcionamento DMG.

- [ ] **C03 — Implementar temporização e transferências CGB.**
  Tratar double-speed, relações de clock da CPU/PPU/APU, KEY1 e DMA CGB, incluindo GDMA/HDMA. Alternar somente uma flag de velocidade não basta; validar cada relação de tempo.

- [ ] **C04 — Publicar compatibilidade apenas após validação própria.**
  Executar suítes e cenários de jogos CGB com referências visuais e sonoras, registrando as limitações restantes. Manter regressões DMG durante toda a expansão.

Dependência: começar a implementação ampla de CGB depois da estabilização DMG e da validação de uso real. A comunicação da limitação e a política de aceitação de ROMs podem ser antecipadas.

## Regras para concluir uma tarefa

1. Reproduzir o problema e verificar todos os consumidores do código afetado.
2. Fazer a menor mudança completa, incluindo testes, fixtures, exports, scripts e documentação afetados.
3. Rodar a regressão específica e os testes do subsistema. Em alterações de hardware, preservar também os resultados das ROMs relevantes; antes de fechar uma etapa, executar a suíte completa.
4. Registrar resultado, limitação conhecida e qualquer mudança no formato de save ou na API pública.
5. Marcar a caixa somente após verificar o critério da tarefa. Criar o teste ou escrever o plano não conclui a correção.

Não fazem parte deste roadmap uma reescrita geral, novos frameworks, novas dependências por conveniência ou otimizações sem medição. A estrutura existente já atende ao objetivo; a prioridade é corrigir e validar seus comportamentos.
