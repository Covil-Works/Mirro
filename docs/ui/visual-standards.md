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
  - Janela sem moldura (`no-frame: true`), fundo transparente (`background: transparent`), sempre no topo (`always-on-top: true`) e composição DWM com margens negativas (`DwmExtendFrameIntoClientArea`).
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

### [visual-standards-010] Botões de Ação por Ícones Vetoriais com Tooltips e Atalhos Rápidos Configuráveis

* **Severidade:** Obrigatório
* **Descrição:** Os botões de hardware e controle no painel lateral de funções do dispositivo conectado devem utilizar exclusivamente ícones vetoriais monocromáticos (sem texto fixo interno no botão), dispostos em linhas horizontais simétricas com expansão flexível uniforme: seção "Navegação" (Voltar, Recentes e Início lado a lado), seção "Botões" (Tela On/Off, Aumentar Volume e Diminuir Volume lado a lado) e seção "Ações" (Captura de Tela e Menu lado a lado). Cada botão deve obrigatoriamente exibir um tooltip flutuante contextual e atualizar o cabeçalho da respectiva seção ao receber hover do mouse. A seção "Atalhos Rápidos" substitui áreas genéricas e deve disponibilizar seletores de ativação/desativação para cada controle fixado no widget flutuante complementar (Companion) da tela espelhada.
* **Justificativa:** Otimiza o aproveitamento vertical da interface com alinhamento cirúrgico de grade, previne acionamentos acidentais por meio de confirmação em tooltip e permite ao usuário customizar a régua de comandos do widget flutuante conforme seu fluxo de trabalho.
* **Incorreto (Anti-padrão):**
  - Empilhar botões com textos extensos em coluna única ("VOLTAR", "INÍCIO", "RECENTES", "TELA ON/OFF") saturando o layout verticalmente.
  - Deixar botões apenas com ícone sem fornecer tooltip ou indicação visual de hover ao passar o mouse.
  - Deixar botões do widget flutuante com conjunto fixo sem persistência das preferências do usuário.
* **Correto (Padrão Aprovado):**
  - Componente `IconButton` com SVG vetorial, inversão monocromática de estado, proporção simétrica na linha e tooltip instantâneo ao passar o cursor.
  - Seção "Atalhos Rápidos" com lista rolável contendo ícone, rótulo e checkbox, salvando as escolhas no arquivo de configuração e atualizando o CompanionWindow em tempo real.
