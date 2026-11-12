# Padrões Visuais e Elementos de Interface (UI)

Este documento centraliza as regras e padrões visuais aplicáveis à interface gráfica do Mirro. Novas regras de apresentação, iconografia, cores e componentes visuais devem ser adicionadas a este arquivo.

---

### [visual-standards-001] Proibição Absoluta de Emojis na Interface

* **Severidade:** Obrigatório
* **Descrição:** É estritamente proibido o uso de emojis em qualquer elemento da interface do aplicativo (botões, listas, modais, toolbars ou rótulos). Caso seja necessária a representação de um símbolo ou ícone, devem ser utilizados glifos técnicos Unicode formais, vetores dedicados (SVG/linhas) ou texto puramente funcional.
* **Justificativa:** Emojis conferem aparência amadora, quebram a consistência estética do sistema, possuem renderização imprevisível em diferentes sistemas operacionais e violam a identidade sóbria, técnica e de alto desempenho da aplicação.
* **Incorreto (Anti-padrão):**
  - Botão com caractere de emoji gráfico (exemplo: ícone de câmera colorida ou smartphone do sistema).
  - Mensagens de status acompanhadas de emojis informais (exemplo: símbolos de alerta amarelo ou celular estilizado).
  - Rótulos com emojis de ferramentas ou lápis para edição.
* **Correto (Padrão Aprovado):**
  - Glifo técnico monocromático ou caractere tipográfico padrão: `icon: "⛶"`, `icon: "✎"`, `icon: "≡"`, `icon: "◀"`, `icon: "●"`
  - Rótulo de texto direto em caixa alta: `text: "CAPTURA DE TELA"`, `text: "RENOMEAR"`, `text: "NÃO AUTORIZADO"`

---

### [visual-standards-002] Paleta Monocromática e Estados de Foco Invertidos

* **Severidade:** Obrigatório
* **Descrição:** A interface deve operar estritamente em preto e branco. O fundo geral da aplicação é preto (`#000000`). Os botões interativos devem possuir fundo branco (`#ffffff`) com texto preto (`#000000`) em repouso. Ao receber foco ou hover do mouse, devem inverter imediatamente para fundo preto (`#000000`), texto branco (`#ffffff`) e borda branca de 1px (`#ffffff`).
* **Justificativa:** Cria alto contraste operacional, clareza cirúrgica de estado e reduz distrações cognitivas durante sessões prolongadas de depuração e espelhamento.
* **Incorreto (Anti-padrão):**
  - Botões coloridos (azul, vermelho, verde, laranja).
  - Sombras difusas decorativas sem propósito funcional.
* **Correto (Padrão Aprovado):**
  - Botão padrão: `background: #ffffff`, `color: #000000`, `border-color: #ffffff`.
  - Botão em foco/hover: `background: #000000`, `color: #ffffff`, `border-color: #ffffff`.

---

### [visual-standards-003] Alinhamento e Navegação da Barra Lateral (Navbar)

* **Severidade:** Obrigatório
* **Descrição:** A barra lateral esquerda opera como uma navbar vertical hierarquizada. Todos os itens de navegação devem ser alinhados estritamente ao topo e à esquerda. A primeira seção deve conter a aba "Configurações" (acompanhada de glifo técnico monocromático de engrenagem) e a seção seguinte deve apresentar "Dispositivos" e a lista de conexões ativas. A seleção de um item na lateral altera dinamicamente o painel da direita.
* **Justificativa:** Estabelece um padrão de navegação Master-Detail limpo e intuitivo, eliminando elementos flutuantes ou centralizados verticalmente sem motivo.
* **Incorreto (Anti-padrão):**
  - Centralizar verticalmente os botões da barra lateral.
  - Espalhar opções de configuração em rodapés escondidos.
* **Correto (Padrão Aprovado):**
  - `VerticalLayout` no container da sidebar com botões alinhados ao topo (`vertical-stretch: 0`) com `Configurações` seguido por `Dispositivos`, e a seção rolável de dispositivos expandida até a base da janela (`vertical-stretch: 1`).

---

### [visual-standards-004] Supressão de Mockups Vazio e Estado de Escaneamento

* **Severidade:** Obrigatório
* **Descrição:** Quando nenhum smartphone estiver conectado ao computador via USB, é estritamente proibido renderizar mockups, molduras ou chassis de celular vazios. A área principal da direita deve exibir uma tela de conexão limpa e objetiva, contendo um botão de ação primária em destaque para "Escanear Dispositivos" e instruções técnicas de conexão.
* **Justificativa:** Reduz ruído visual desnecessário, melhora a experiência de uso inicial e foca a atenção na ação requerida para iniciar a sessão.
* **Incorreto (Anti-padrão):**
  - Exibir a moldura de telefone com tela preta ou botões de hardware inativos quando não há aparelho conectado.
* **Correto (Padrão Aprovado):**
  - Exibir painel centralizado de detecção com o botão "ESCANEAR DISPOSITIVOS" e orientações de depuração USB.

---

### [visual-standards-005] Alinhamento Vertical e Containers de Ícones

* **Severidade:** Obrigatório
* **Descrição:** Todos os ícones renderizados ao lado de rótulos de texto, listas ou botões devem possuir container com dimensões explícitas (ex.: `Rectangle { width: 16px; height: 16px; ... }`) e estarem contidos em layouts com alinhamento vertical explícito no eixo transversal (`cross-axis-alignment: center;`). É proibido delegar o alinhamento de ícones à propriedade inexistente `vertical-alignment` em elementos `Image` ou a coordenadas manuais `y:` como filho direto de layouts.
* **Justificativa:** No framework Slint, elementos com altura fixa dentro de `HorizontalLayout` padrão (`stretch`) são posicionados no topo (`y: 0`), enquanto textos com `vertical-alignment: center` ocupam o centro vertical da linha. Sem `cross-axis-alignment: center;` e containers padronizados, ícones sobem para o topo do componente, quebrando o alinhamento óptico com a tipografia adjacente.
* **Incorreto (Anti-padrão):**
  - Elemento `Image` solto com `vertical-alignment: center;` dentro de `HorizontalLayout` sem `cross-axis-alignment: center;`.
  - Inserir `y: (parent.height - self.height) / 2;` em elementos filhos diretos de `HorizontalLayout` ou `VerticalLayout` (o layout sobrescreve posições manuais).
* **Correto (Padrão Aprovado):**
  - `HorizontalLayout { alignment: start; cross-axis-alignment: center; spacing: 10px; Rectangle { width: 16px; height: 16px; Image { source: icon; width: 16px; height: 16px; } } Text { text: label; vertical-alignment: center; } }`

---

### [visual-standards-006] Margens Laterais e Espaçamento Interno em Botões (Padding)

* **Severidade:** Obrigatório
* **Descrição:** É estritamente proibido renderizar botões com margens laterais coladas ou apertadas em relação ao texto interno. Todo componente de botão (`MonoButton` ou equivalente) deve implementar margem interna horizontal mínima padronizada (`padding-left` e `padding-right` de no mínimo 16px a 18px para botões regulares e 12px para botões compactos) e dimensionamento de largura mínima proporcional ao conteúdo (`min-width: label.min-width + (padding * 2)`). Em botões que preenchem toda a largura disponível de um container (largura 100%), o texto deve permanecer centralizado com folga segura em relação às bordas.
* **Justificativa:** Botões com texto colado às bordas aparentam desleixo visual, quebram a hierarquia óptica, prejudicam a legibilidade e diminuem a precisão de clique. Um padding horizontal generoso confere sofisticação, clareza técnica e elegância à estética monocromática do Mirro.
* **Incorreto (Anti-padrão):**
  - Botão com bordas coladas no texto: `[Copiar e colar]`, `[EDITAR]` ou `[SALVAR]` sem folga lateral.
  - Botão dentro de layout horizontal sem `min-width` ou sem padding, assumindo apenas as dimensões mínimas da fonte.
* **Correto (Padrão Aprovado):**
  - Botão com margens arejadas e simétricas: `[   Copiar e colar   ]`, `[   EDITAR   ]`, `[   ESPELHAR   ]`.
  - Componente com propriedade `padding_horizontal` (padrão 18px), `HorizontalLayout` interno e `min-width: label.min-width + (root.padding_horizontal * 2)`.

---

### [visual-standards-007] Responsividade e Quebra Obrigatória de Linha (Word Wrap)

* **Severidade:** Obrigatório
* **Descrição:** Todos os blocos de texto descritivo, instruções, parágrafos explicativos e legendas devem possuir quebra de linha ativada explicitamente (`wrap: word-wrap;`). Em layouts horizontais contendo textos adjacentes a botões, seletores ou controles, o container de texto deve receber expansão flexível (`horizontal-stretch: 1;`) enquanto o controle adjacente recebe largura fixa (`horizontal-stretch: 0;`). É estritamente proibido deixar textos longos com comportamento padrão de linha única infinita (`no-wrap`).
* **Justificativa:** No framework Slint, o padrão de renderização de texto é linha única horizontal. Sem `wrap: word-wrap;` e contenção flexível, textos explicativos forçam a expansão irrestrita do layout, empurrando controles para fora da área visível e quebrando a responsividade da janela em resoluções menores.
* **Incorreto (Anti-padrão):**
  - Texto explicativo de configuração ou instrução sem a propriedade `wrap: word-wrap;`.
  - Layout horizontal dividindo texto e toggle sem atribuir `horizontal-stretch: 1;` ao bloco de texto.
* **Correto (Padrão Aprovado):**
  - `HorizontalLayout { spacing: 16px; VerticalLayout { horizontal-stretch: 1; Text { text: "..."; wrap: word-wrap; } } MonoToggle { horizontal-stretch: 0; } }`
  - Painéis de configuração e formulários contidos em `ScrollView` com rolagem vertical suave.

---

### [visual-standards-008] Widget Flutuante Complementar (Companion) em Formato Geométrico

* **Severidade:** Obrigatório
* **Descrição:** O widget flutuante auxiliar de controle durante o espelhamento deve operar em dois estados geométricos bem definidos: estado colapsado (círculo com diâmetro de 52px) e estado expandido (pílula vertical de 52px de largura e 270px de altura com extremidades semicirculares). O widget deve flutuar com posicionamento relativo contínuo anexado ao canto superior direito da janela de espelhamento (com folga óptica de 10px, sem tocar diretamente no frame do scrcpy). A transparência deve ser delegada ao compositor DWM do Windows (`DwmExtendFrameIntoClientArea`), permitindo renderização suave com anti-aliasing por GPU pelo Slint, com dimensionamento físico proporcional à escala DPI do monitor (`GetDpiForWindow`). É estritamente proibido o uso de máscaras binárias GDI (`SetWindowRgn`), que destroem o anti-aliasing e causam bordas serrilhadas.
* **Justificativa:** Garante atalhos de navegação e controle confortáveis (Voltar, Início, Recentes, Ligar/Desligar tela e Encerrar) sem cortes de layout em monitores de alta densidade de pixels (125%, 150%, 200% DPI) e com bordas curvas suaves de alto padrão estético.
* **Incorreto (Anti-padrão):**
  - Utilizar recorte GDI `SetWindowRgn` que gera contornos pontilhados e serrilhados sem anti-aliasing.
  - Fixar dimensões físicas brutas sem multiplicar pelo fator de escala DPI do monitor, espremendo o layout e cortando botões.
  - Colar o widget diretamente sobre o vídeo do espelhamento, obstruindo a interface do smartphone.
  - Deixar o widget flutuante órfão na tela após o encerramento do processo scrcpy.
* **Correto (Padrão Aprovado):**
  - Janela sem moldura (`no-frame: true`), fundo transparente (`background: transparent`), composição DWM com margens negativas (`DwmExtendFrameIntoClientArea`) e sincronização de Z-order com o espelhamento.
  - Conversão dinâmica de medidas lógicas para pixels físicos usando `GetDpiForWindow`.
  - Transição de estado acionada por clique entre círculo compacto e pílula vertical.
  - Sincronização automática de ciclo de vida com o processo e janela do scrcpy.

---

### [visual-standards-009] Identidade Visual do Smartphone Mockado com Desfoque e Cards de Estado

* **Severidade:** Obrigatório
* **Descrição:** A tela interna do smartphone mockado na área principal de trabalho deve renderizar a captura real da tela do dispositivo conectado com aplicação de desfoque Gaussiano (blur) em segundo plano, acompanhada de película escura suave para garantir contraste. Em repouso (não espelhado), exibe card central translúcido contendo o nome do dispositivo em destaque (tipografia maior) e o codinome/modelo técnico em corpo menor, sem botão de ação interno (a ação de espelhar permanece centralizada na barra superior da área de trabalho). Quando a sessão de espelhamento for iniciada, o card de transmissão ativa deve ser exibido em estética estritamente monocromática, suprimindo qualquer elemento gráfico colorido (como indicadores circulares ou radares verdes) e mantendo exclusivamente o botão "ENCERRAR SESSÃO".
* **Justificativa:** Proporciona identificação visual instantânea do dispositivo conectado através de sua própria tela, mantém harmonia óptica com o tema preto e branco da aplicação e evita duplicações desnecessárias de botões de controle na interface.
* **Incorreto (Anti-padrão):**
  - Exibir animações ou círculos coloridos verdes no card de transmissão dentro do telefone mockado.
  - Inserir botão de espelhamento dentro do telefone mockado quando a barra superior já possui controle primário de espelhar.
  - Deixar a tela do mockup com fundo estático inteiramente preto quando houver captura de tela disponível.
* **Correto (Padrão Aprovado):**
  - Capturar a tela via ADB assim que o aparelho for espelhado/conectado, aplicar redimensionamento com filtro bilinear e desfoque Gaussiano, exibindo como plano de fundo da tela interna.
  - Card central em repouso exibindo apenas nome em destaque e codinome técnico em corpo menor.
  - Card de transmissão monocromático exibindo "TRANSMISSÃO ATIVA", textos explicativos e o botão "ENCERRAR SESSÃO".

---

### [visual-standards-010] Botões de Ação por Ícones Vetoriais com Rótulo no Cabeçalho e Atalhos Rápidos

* **Severidade:** Obrigatório
* **Descrição:** Os botões de hardware e controle no painel lateral de funções do dispositivo conectado devem utilizar exclusivamente ícones vetoriais monocromáticos (sem texto fixo interno no botão), dispostos em linhas horizontais simétricas com expansão flexível uniforme: seção "Navegação" (Voltar, Recentes e Início lado a lado), seção "Botões" (Tela On/Off, Aumentar Volume e Diminuir Volume lado a lado) e seção "Ações" (Captura de Tela e Menu lado a lado). Ao receber hover do mouse, o nome da ação deve ser exibido estritamente no cabeçalho da respectiva seção (à direita do título da seção), sendo proibido o uso de tooltips flutuantes em caixas sobrepostas. O espaçamento vertical entre seções deve manter proporção 2:1 em relação ao espaçamento entre o cabeçalho e a linha de botões. A seção "Atalhos Rápidos" disponibiliza seletores de ativação/desativação para cada controle fixado no widget flutuante complementar (Companion).
* **Justificativa:** Elimina caixas de tooltip flutuantes que sobrepõem bordas e quebram a estética limpa do painel, fornecendo feedback de identificação imediato e estável no topo de cada seção com cadência e ritmo espacial impecáveis.
* **Incorreto (Anti-padrão):**
  - Utilizar caixas flutuantes de tooltip soltas acima dos botões com posições absolutas ou negativas.
  - Empilhar botões com textos extensos em coluna única ("VOLTAR", "INÍCIO", "RECENTES", "TELA ON/OFF") saturando o layout verticalmente.
  - Deixar botões apenas com ícone sem fornecer indicação textual no cabeçalho ao passar o mouse.
* **Correto (Padrão Aprovado):**
  - Linha de cabeçalho da seção com altura fixa exibindo o título da seção à esquerda e o nome do botão em foco à direita em tipografia monocromática branca.
  - Componente `IconButton` com SVG vetorial, inversão monocromática de foco (fundo preto/branco), cantos arredondados de 6px e espaçamento vertical proporcional de 10px em relação ao cabeçalho.

---

### [visual-standards-011] Sincronização de Z-Order e Minimização do Widget Companion

* **Severidade:** Obrigatório
* **Descrição:** O widget flutuante de ações rápidas (Companion) deve operar com vínculo direto de propriedade nativa (Owner Window via `GWLP_HWNDPARENT`) com a janela de espelhamento do scrcpy. A ordem no eixo Z (profundidade) e o estado de visibilidade devem permanecer estritamente unificados: se a janela de espelhamento for enviada para segundo plano (atrás de outro aplicativo), o companion deve ir para trás junto; se for minimizada (via barra de tarefas, atalho de sistema ou botão de minimizar), o companion deve ser ocultado imediatamente; ao ser restaurada ou trazida para frente, o companion deve ressurgir acoplado a ela. É estritamente proibido forçar `HWND_TOPMOST` ou `WS_EX_TOPMOST` incondicionalmente no loop de rastreamento.
* **Justificativa:** Manter o widget flutuante permanentemente no topo sobrepondo outros programas de terceiros gera poluição visual, perturba o fluxo de trabalho do usuário e desassocia visualmente o controle da tela do smartphone à qual pertence.
* **Incorreto (Anti-padrão):**
  - Chamar `SetWindowPos` com `HWND_TOPMOST` em loops de polling contínuo sem `SWP_NOZORDER`.
  - Fixar `always-on-top: true;` diretamente no componente Slint do companion.
  - Deixar o botão de ações rápidas flutuando na frente de outros navegadores ou programas quando a janela do espelhamento estiver em segundo plano ou minimizada.
* **Correto (Padrão Aprovado):**
  - Configurar `SetWindowLongPtrW(comp_hwnd, GWLP_HWNDPARENT, scrcpy_hwnd)` para estabelecer relação formal de propriedade pelo Windows.
  - No reposicionamento contínuo, utilizar `SetWindowPos` com a flag `SWP_NOZORDER | SWP_NOACTIVATE`.
  - Aplicar `WS_EX_TOPMOST` e `HWND_TOPMOST` exclusivamente se a opção de espelhamento "Sempre no topo" estiver ativada pelo usuário.
  - Ocultar o widget via `ShowWindow(comp_hwnd, SW_HIDE)` ao detectar `IsIconic` ou `!IsWindowVisible` na janela principal de espelhamento.

---

### [visual-standards-012] Identidade Visual e Iconografia Unificada de Janelas

* **Severidade:** Obrigatório
* **Descrição:** Todas as janelas do ecossistema do aplicativo — a janela principal do Mirro (`AppWindow`), o widget flutuante de ações rápidas (`CompanionWindow`) e a janela nativa de espelhamento do scrcpy — devem utilizar o logotipo oficial unificado do aplicativo (`ui/icons/mirro-icon.svg` e seus equivalentes renderizados `.ico`/`.png`). A atribuição do ícone deve ocorrer no nível do framework Slint (`icon: @image-url("icons/mirro-icon.svg");`) e via Win32 API (`WM_SETICON` com `ICON_BIG` e `ICON_SMALL`) diretamente no handle de cada janela (`HWND`), inclusive configurando o diretório de ícone do scrcpy e aplicando o ícone diretamente ao seu handle.
* **Justificativa:** Elimina ícones padrão genéricos do sistema operacional ou de bibliotecas externas (como o ícone de engrenagem do SDL ou a pena do scrcpy), garantindo coerência visual, reconhecimento de marca e acabamento profissional na barra de tarefas e na barra de títulos do Windows.
* **Incorreto (Anti-padrão):**
  - Manter o ícone nativo do SDL/scrcpy na janela de espelhamento.
  - Deixar o widget flutuante companion ou a janela principal sem ícone definido na barra de tarefas.
  - Utilizar ícones distintos ou formatos incompatíveis entre as janelas do sistema.
* **Correto (Padrão Aprovado):**
  - Gerar o vetor oficial SVG a partir da matriz de design com transparência externa e traçado nítido.
  - Declarar `icon: @image-url("icons/mirro-icon.svg");` nas janelas Slint (`AppWindow` e `CompanionWindow`).
  - Aplicar `set_window_icon(hwnd)` nas instâncias Win32 identificadas para `AppWindow`, `CompanionWindow` e na janela do processo scrcpy.

---

### [visual-standards-013] Notificação Toast e Configuração de Diretório de Capturas

* **Severidade:** Obrigatório
* **Descrição:** O aplicativo deve fornecer retorno visual explícito (toast monocromático temporizado) ao registrar uma captura de tela, informando que a imagem foi salva no celular e no computador com o caminho de destino. O painel de configurações deve disponibilizar campo de visualização com truncamento óptico (`overflow: elide`), botão para seleção de pasta nativa via diálogo de sistema, botão para abertura imediata no Windows Explorer e opção de restauração do caminho padrão (`Imagens/Mirro`). É estritamente proibido salvar capturas silenciosamente sem indicar a pasta ao usuário ou utilizar cores chamativas no toast.
* **Justificativa:** Elimina a incerteza do usuário quanto ao sucesso do print no celular e à localização do arquivo no sistema de arquivos do computador, mantendo a consistência estética com o tema escuro minimalista.
* **Incorreto (Anti-padrão):**
  - Salvar capturas de forma oculta sem feedback visual de confirmação na janela.
  - Exibir toasts com ícones coloridos ou emojis (exemplo: balões verdes ou amarelos).
  - Deixar o caminho de salvamento fixo em código sem opção de escolha pelo usuário.
* **Correto (Padrão Aprovado):**
  - Toast flutuante com fundo escuro translúcido (`#050505fa`), borda branca fina de 1px, ícone monocromático vetorial e temporizador automático de 4 segundos.
  - Seção dedicada nas Configurações com botões "ALTERAR PASTA", "ABRIR NO EXPLORER" e "RESTAURAR PADRÃO".

---

### [visual-standards-014] Painel Lateral de Arquivos em /mirro e Transferência por Arraste (Drag and Drop)

* **Severidade:** Obrigatório
* **Descrição:** O gerenciamento de arquivos do diretório `/mirro` no smartphone deve ser posicionado à direita do painel de botões de ações na janela principal do aplicativo, mantendo altura unificada (580px) com a moldura do celular e com o painel de navegação. A lista de arquivos e pastas deve ser minimalista e de alta densidade óptica: linhas compactas (26px de altura) com ícones vetoriais monocromáticos (ícone SVG minimalista para pastas e identificação direta pelo nome e extensão para arquivos, ex.: `screenshot.jpg`), sem cards volumosos ou emblemas pesados. A transferência de arquivos deve ser suportada de forma transparente por arrastar e soltar (Drag and Drop) tanto na janela de espelhamento do scrcpy (via `--push-target=/sdcard/mirro/`) quanto na janela do aplicativo (via `WM_DROPFILES` e `DragAcceptFiles` processados na thread de UI e propagados para janelas filhas), enviando diretamente para `/sdcard/mirro/` após assegurar a existência da pasta no sistema de arquivos do Android.
* **Justificativa:** Proporciona transferência de arquivos simplificada e rápida sem a necessidade de comandos manuais no terminal ou janelas modais complexas de gerenciamento, mantendo uma visualização limpa, padrão e minimalista de arquivos alinhada à estética técnica do Mirro.
* **Incorreto (Anti-padrão):**
  - Renderizar cards pesados, badges multicoloridos ou caixas volumosas para cada arquivo individual.
  - Salvar arquivos arrastados em pastas genéricas como `/sdcard/Download/` sem centralizar em `/mirro`.
  - Exibir ícones coloridos ou emojis na lista de itens.
  - Falhar silenciosamente se o diretório `/sdcard/mirro` ainda não tiver sido criado no dispositivo.
* **Correto (Padrão Aprovado):**
  - Assegurar a criação automática de `/sdcard/mirro` antes do espelhamento ou da transferência de arquivos.
  - Renderizar painel lateral dedicado com 250px de largura e 580px de altura, exibindo itens em linhas compactas e elegantes com ícone SVG de pasta e nome do arquivo com hover sutil.
  - Disponibilizar botões de ação rápida ("ATUALIZAR" e "ENVIAR") e suporte a Drag and Drop nativo em ambas as janelas.

---

### [visual-standards-015] Iconografia de Arquivos, Menu Suspenso de Contexto e Visualizador/Editor Modal

* **Severidade:** Obrigatório
* **Descrição:** A área de arquivos da janela principal deve adotar iconografia vetorial em miniatura de 14px posicionada estritamente à esquerda de cada nome de item (`folder.svg` para pastas e `file.svg` para arquivos). O cabeçalho de ações da área de arquivos deve utilizar botões por ícone vetorial monocromático compacto (ícone padrão de atualização `refresh.svg` com setas circulares e ícone de upload `upload.svg` com seta para cima), reservando botão textual adicional exclusivamente para a ação contextual "COLAR" quando houver item na área de transferência. O clique com o botão direito sobre qualquer item deve abrir um menu suspenso de contexto nativo com opções funcionais ("ABRIR / EDITAR" ou "ABRIR PASTA", "COPIAR", "RECORTAR", "COLAR", "SELECIONAR", "BAIXAR NO PC" e "APAGAR"), com fechamento automático ao clicar fora. A abertura de arquivos de texto, código e imagem deve exibir um modal centralizado de visualização e edição direta com suporte a salvar alterações de volta no smartphone via ADB e botão para abertura no computador com o aplicativo padrão do Windows.
* **Justificativa:** Fornece paridade de recursos com gerenciadores de arquivos clássicos de desktop, eliminando a barreira de comandos de terminal para manipulação direta de arquivos do celular, garantindo leitura e edição imediatas com retorno visual confiável e consistência técnica monocromática.
* **Incorreto (Anti-padrão):**
  - Utilizar botões de texto extensos como "ATUALIZAR" e "ENVIAR" no cabeçalho quando ícones vetoriais universais economizam espaço e harmonizam o design.
  - Exibir ícone apenas para diretórios, deixando arquivos sem miniatura identificadora à esquerda do nome.
  - Abrir arquivos silenciosamente sem interface gráfica de visualização ou confirmação.
  - Utilizar cores ou emojis no menu de contexto ou nas ações de exclusão.
* **Correto (Padrão Aprovado):**
  - Miniaturas padronizadas de 14px contidas em containers alinhados (`cross-axis-alignment: center`) para pastas e arquivos.
  - Botões de cabeçalho por ícones monocromáticos `refresh.svg` e `upload.svg`.
  - Menu suspenso flutuante com estados de hover monocromáticos e suporte a copiar, recortar (com opacidade visual reduzida no item), colar, selecionar e apagar.
  - Modal integrado de visualização/edição (`TextEdit` para textos/códigos e `Image` para imagens) com botões "SALVAR NO CELULAR", "ABRIR NO PC" e "FECHAR".

---

### [visual-standards-016] Centralização Bidirecional do Palco Central de Dispositivos (Mockup, Funções e Arquivos)

* **Severidade:** Obrigatório
* **Descrição:** O conjunto central de elementos da área de trabalho do dispositivo (moldura mockada do smartphone, painel lateral de botões de hardware e painel lateral de arquivos em `/mirro`) deve permanecer rigorosamente centralizado no plano bidirecional (tanto horizontalmente quanto verticalmente) no espaço útil compreendido entre o divisor da barra superior e o rodapé da janela. O container intermediário deve receber expansão flexível vertical (`vertical-stretch: 1;`) com alinhamento centrado (`alignment: center;`), e o layout horizontal dos blocos deve implementar alinhamento transversal centrado (`cross-axis-alignment: center;`) e alinhamento principal centrado (`alignment: center;`). É expressamente proibido permitir que o palco central fique ancorado no topo com acúmulo assimétrico de espaço vazio na porção inferior ao redimensionar ou maximizar a janela.
* **Justificativa:** Em resoluções elevadas (Full HD, 2K, 4K) ou janelas maximizadas, elementos fixados no topo criam desproporção óptica, cansaço visual e sensação de interface incompleta, enquanto a centralização bidirecional mantém simetria harmoniosa e foco operacional limpo no centro da visão do usuário.
* **Incorreto (Anti-padrão):**
  - Posicionar o palco de 580px imediatamente abaixo do cabeçalho sem container expansível centralizador, deixando grandes faixas vazias na base da tela.
  - Alinhar apenas horizontalmente (`alignment: center`) enquanto a coordenada vertical permanece colada no topo (`y` inicial sem distribuição de margem inferior).
* **Correto (Padrão Aprovado):**
  - Barra superior e divisor com `vertical-stretch: 0;`.
  - Container intermediário `VerticalLayout` com `vertical-stretch: 1;` e `alignment: center;`.
  - Container interno `HorizontalLayout` com `alignment: center;`, `cross-axis-alignment: center;` e espaçamento padronizado de 24px entre o mockup, o painel de funções e o painel de arquivos.
