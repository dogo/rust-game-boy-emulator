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
  Os 14 cenários de hardware e diagnóstico estão em `tests/known_regressions.rs`; os três de interface estão em `src/GB/sdl_runner.rs`. Foram confirmados falhando quando executados com `--ignored`. Permanecem ignorados na suíte normal até as tarefas indicadas em cada motivo de `#[ignore]`; retirar a marcação ao corrigir cada bug.

- [x] **V04 — Executar ROMs no CI.**
  Atualizar `.github/workflows/ci.yml` para preparar os submódulos e ROMs necessários, executar testes Rust e as suítes Blargg/Mooneye e preservar os diagnósticos das falhas. Concluído quando regressão de ROM reprova o CI e ausência de fixtures não resulta em sucesso.

Critério de saída: testes Rust e 163 ROMs existentes continuam aprovados, com build atual e contagem explícita. Mudanças justificadas na seleção de ROMs devem atualizar a documentação e a contagem esperada.

Etapa 0 implementada e validada localmente: 73 testes Rust ativos passaram, 17 regressões conhecidas foram reproduzidas separadamente e as 163 ROMs passaram. O script rejeitou build com falha e suíte sem fixtures em verificações isoladas. O workflow foi configurado, mas sua primeira execução hospedada no GitHub ainda precisa confirmar a instalação SDL3 e o download Mooneye naquele ambiente.

## Etapa 1 — Proteger progresso e estabilizar a interface

- [ ] **S01 — Substituir saves somente após escrita bem-sucedida.**
  Em `src/GB/bus.rs` e nos consumidores em `src/main.rs`, gravar em temporário no mesmo diretório e substituir o destino após concluir. Manter o save anterior se a escrita falhar e informar o erro. Testar ida e volta e falha antes da substituição; definir a necessidade de sincronização em disco se houver requisito de resistência a queda de energia.

- [ ] **S02 — Preservar RTC com HALT e sem RAM.**
  Em `src/GB/mbc/mbc3.rs`, respeitar HALT ao aplicar tempo offline e salvar o relógio mesmo quando a RAM está vazia. Cobrir save parado por 60 segundos, relógio ativo e cartucho com RTC+bateria sem RAM; preservar a leitura dos formatos antigos já suportados.

- [ ] **S03 — Tratar estado RTC inválido.**
  Validar os dados do save antes de aceitá-los e definir o comportamento dos valores fora do intervalo escritos pela ROM. Eliminar underflow em `60 - sec` e testar segundos=60/61, dados truncados e timestamps extremos sem esperas reais. Erro de carga deve ser comunicado e não levar à substituição silenciosa do save original.

- [ ] **I01 — Corrigir parada e retomada por breakpoint.**
  Em `src/GB/sdl_runner.rs` e `src/GB/debugger.rs`, preservar os ciclos de frame parcial, comunicar a pausa à interface e entrar no mesmo fluxo de interação usado por F12. Testar breakpoint no primeiro opcode, no meio do frame, step, continue e encerramento durante a pausa em debug e release.

- [ ] **I02 — Simplificar a entrega de frames entre threads.**
  Corrigir a intercalação entre leitura e escrita de `ready_idx`. Preferir um buffer do frame mais recente protegido por `Mutex`, suficiente para a imagem atual, se mantiver o comportamento necessário. Validar publicação concorrente e ausência de frame parcialmente copiado; não adicionar uma dependência só para preservar o triple buffer.

- [ ] **I03 — Preservar a ordem ou o estado final dos botões.**
  Em `src/GB/sdl_runner.rs`, substituir as máscaras independentes de pressionar/soltar por representação que preserve o resultado dos eventos, mantendo os pedidos de interrupção e a retomada de STOP. Testar pressionar→soltar, soltar→pressionar entre frames, múltiplos botões e tecla mantida.

Critério de saída: os cenários acima têm regressões automatizadas; uma sessão gráfica permite pausar, inspecionar, continuar e sair com save íntegro.

## Etapa 2 — Corrigir a emulação DMG

- [ ] **D01 — Resolver corretamente prioridade entre sprites e fundo.**
  Em `src/GB/PPU.rs`, escolher o pixel vencedor entre sprites antes de aplicar a prioridade do background. Cobrir menor X, empate por ordem de OAM, transparência e sprite prioritário oculto pelo fundo. Apenas inverter a ordem do loop não resolve todos os casos.

- [ ] **D02 — Limpar a máscara quando o background é desabilitado.**
  Limpar `bg_priority` junto com a linha e impedir que o desenho dos sprites destrua a informação original do fundo. Testar um sprite de baixa prioridade após desligar o BG, inclusive com sobreposição de sprites; coordenar com D01.

- [ ] **D03 — Aplicar bloqueios de VRAM/OAM na linha zero.**
  Restringir a exceção de ligar o LCD ao período em que ela realmente está ativa. Revisar os quatro caminhos de bloqueio de leitura/escrita. Cobrir a primeira linha após ligar o LCD e a linha zero de frames posteriores, com LCD ligado/desligado e modos 2/3.

- [ ] **D04 — Corrigir recorte e contador vertical da window.**
  Aceitar a janela parcialmente visível com `WX < 7` e usar o contador de linhas efetivamente desenhadas, em vez de apenas `LY-WY`. Testar WX=6/7, ocultação e retomada via WX, condições de WY e reset por frame; tratar peculiaridades dos limites conforme evidência de hardware.

- [ ] **D05 — Completar reload de TIMA mesmo após desabilitar o timer.**
  Em `src/GB/timer.rs`, processar o reload pendente independentemente de TAC. Cobrir overflow seguido de TAC=0, atraso de quatro T-cycles, interrupção e escritas em TIMA/TMA durante o atraso. Preservar os testes Mooneye já aprovados.

- [ ] **D06 — Modelar conflitos de barramento durante OAM DMA.**
  Em `src/GB/bus.rs`, aplicar as restrições aos acessos da CPU, separando-os dos acessos internos do DMA. Cobrir ROM, WRAM, HRAM, início, término e reinício da transferência. Confirmar os resultados exatos dos conflitos no modelo DMG; não generalizar as regras para CGB.

- [ ] **D07 — Ler os dois grupos do joypad corretamente.**
  Em `src/GB/joypad.rs`, combinar as linhas quando os dois grupos estiverem selecionados. Testar nenhum grupo, cada grupo isolado e ambos, com combinações de direção e ação; preservar a leitura ativa em zero.

- [ ] **D08 — Tratar cartuchos sem controlador com RAM e tipos desconhecidos.**
  Em `src/GB/mbc/mod.rs`, `none.rs` e seus consumidores, dar suporte aos tipos 0x08/0x09 ou rejeitá-los explicitamente até a implementação. Recusar controladores desconhecidos em vez de cair silenciosamente em ROM simples. Para declarar suporte, testar leitura/escrita da RAM e persistência do tipo com bateria.

- [ ] **D09 — Separar STOP de HALT.**
  Em `src/GB/CPU.rs`, não avançar o divisor como se a CPU estivesse em HALT. Cobrir divisor parado, entrada/saída de STOP e retomada por joypad; verificar também o consumo do byte de padding da instrução. Manter as regras de troca de velocidade CGB separadas do caminho DMG.

- [ ] **D10 — Representar o travamento dos opcodes ilegais.**
  A primeira análise identificou que eles são tratados como NOP. Confirmar o comportamento do modelo alvo e implementar estado de travamento sem confundir com HALT, STOP ou opcode desconhecido. Cobrir os opcodes ilegais e demonstrar que a CPU não continua executando instruções seguintes.

Critério de saída: regressões de cada tarefa passam, assim como as suítes existentes. Para gráficos, adicionar comparações de pixels que distingam os objetos envolvidos; testes com tiles de cores iguais não detectam inversão de prioridade.

## Etapa 3 — Melhorar diagnóstico e limpar código

- [ ] **L01 — Usar os nomes das instruções executadas no debugger e no trace.**
  Revisar `src/GB/debugger.rs`, `trace.rs` e os metadados de microcódigo. Uma instrução implementada como `3E 42` deve aparecer como `LD A,d8`, com seu operando, e prefixos CB precisam do nome correto. Cobrir também tamanhos e avanço do disassembly.

- [ ] **L02 — Remover o barramento legado sem consumidores internos.**
  Confirmar referências de `src/GB/RAM.rs`, inclusive export em `src/GB.rs`, testes e eventuais consumidores externos. Remover módulo e export se a API não for necessária. A remoção estimada de 148 linhas só vale após essa confirmação; manter `MemoryBus` como implementação efetiva.

- [ ] **L03 — Reduzir o caminho legado de instruções.**
  Depois de L01, revisar a necessidade de `instructions::decode()` em toda instrução e do fallback CB. Remover caminhos comprovadamente inalcançáveis. Migrar DI/EI/HALT/STOP somente se isso simplificar o conjunto, preservando seus efeitos e temporização; não apagar o módulo antes de atualizar todos os consumidores e testes.

- [ ] **L04 — Separar seleção de boot dos nomes de arquivo de teste.**
  Em `src/main.rs` e `run_all_tests.sh`, manter a escolha de modelo por nome restrita à infraestrutura de testes, usando seleção explícita quando necessário. Renomear uma ROM de jogo não deve mudar o hardware emulado; as variantes Mooneye devem continuar recebendo o modelo esperado.

- [ ] **L05 — Tratar avisos e reduzir o ícone sem ampliar dependências.**
  Medir os avisos atuais do Clippy e corrigir os que melhorarem clareza ou revelarem erro, sem refatoração mecânica ampla. Verificar dimensões e tamanho de `icon.bmp`; reduzir o bitmap se possível. Só mudar o formato se o carregamento existente o suportar sem adicionar infraestrutura desnecessária.

Critério de saída: debugger e trace descrevem instruções reais, não existem referências quebradas às remoções e nenhuma limpeza altera o comportamento das suítes aprovadas.

## Etapa 4 — Validar uso real antes de otimizar

- [ ] **Q01 — Registrar uma matriz pequena de jogos e cenários.**
  Usar ROMs disponíveis legalmente e identificar jogo, versão/hash, controlador e modo. Cobrir boot, menus, sobreposição de sprites, placar/window, controles, save/reabertura e uma sessão prolongada. Registrar defeitos observados sem converter ausência de teste em declaração de compatibilidade.

- [ ] **Q02 — Verificar imagem, áudio e latência na interface.**
  Comparar screenshots com referências confiáveis e ouvir os quatro canais, transições, pausa e retomada. Observar estalos, afinação, falta/excesso de amostras e resposta dos controles. Transformar divergências reproduzíveis em testes pequenos antes de alterar a APU.

- [ ] **Q03 — Medir os caminhos de execução.**
  Registrar máquina, perfil de build, ROM e tempo de execução para headless e uso gráfico. Medir consumo de CPU, ritmo de frames e buffer de áudio. Investigar o loop de `apu.tick_m_cycle()` em `bus.tick` apenas se ele aparecer como custo relevante; processamento em lote deve preservar a ordem dos eventos e resultados de áudio.

- [ ] **Q04 — Atualizar a documentação com o que foi validado.**
  Atualizar README com comandos, controles, limites DMG/CGB, suítes e resultados atuais. Distinguir testes de registradores/timing de compatibilidade visual e sonora. Manter contagens de testes sincronizadas com as execuções reais.

Critério de saída: há evidência de uso gráfico e sonoro e uma medição reproduzível. Otimizações só entram quando corrigem um gargalo medido e mantêm a precisão.

## Etapa 5 — CGB como expansão opcional

O PPU atual é DMG. Aprovar testes CGB de áudio ou snapshots de boot não demonstra suporte a jogos CGB. Até existir suporte suficiente, informar claramente a limitação e recusar execução normal de ROMs exclusivamente CGB; preservar os caminhos necessários aos testes de hardware.

- [ ] **C01 — Definir e comunicar os modos suportados.**
  Manter ROMs compatíveis com flag 0x80 no caminho DMG enquanto necessário e diferenciar ROMs exclusivamente CGB. Testar a política de aceitação e documentá-la antes de anunciar suporte.

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
