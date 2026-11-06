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
  - `VerticalLayout { alignment: start; }` no container da sidebar com botões alinhados à esquerda: `Configurações` no topo seguido por `Dispositivos`.

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
