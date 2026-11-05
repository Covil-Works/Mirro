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
