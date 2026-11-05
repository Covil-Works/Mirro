# Registro Oficial de Decisões — Sessão /grill-me (Mirro MVP)

> **Data da Sessão:** 29 de Outubro de 2026  
> **Participantes:** Usuário e Assistente  
> **Objetivo:** Mapear a árvore de design completa, alinhar requisitos e eliminar ambiguidades antes da implementação do **Mirro (MVP)**.

---

## 1. Visão Geral da Arquitetura em 3 Peças

Durante a sessão, estabeleceu-se o modelo conceitual do sistema:

```
┌────────────────────────────────────────────────────────┐
│               1. ADB (O Carteiro USB)                  │
│  Detecta quais celulares estão plugados no computador  │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│                 2. Mirro (O Nosso App)                 │
│  Painel de controle em Rust com botões, lista de       │
│  aparelhos e configurações de espelhamento             │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│               3. scrcpy (O Motor de Vídeo)             │
│  Transmite a tela do celular em alta velocidade e      │
│  envia cliques de mouse e teclas sem atraso            │
└────────────────────────────────────────────────────────┘
```

---

## 2. As 7 Rodadas do Design Tree (Perguntas, Opções e Decisões)

### ❓ Rodada 1 — Framework para Interface Gráfica (GUI) em Rust puro

- **Contexto:** Necessidade de interface 100% nativa em Rust, com baixo consumo de memória, sem WebViews/Electron nem navegadores embutidos.
- **Opções apresentadas:**
  - *Opção A — `egui`:* Immediate-mode, extremamente leve, foco funcional/técnico.
  - *Opção B — `Iced`:* Arquitetura estilo Elm, código mais estruturado e burocrático.
  - *Opção C — `Slint`:* Linguagem de design declarativa própria (`.slint`), visual moderno, suporte nativo a transições fluidas e temas.
- **Resposta do Usuário:**
  > *"Slint, pela personalização."*
- **Decisão Final:** **Slint (com motor Rust)**.

---

### ❓ Rodada 2 — Distribuição de Dependências (ADB e scrcpy)

- **Contexto:** O aplicativo precisa dos executáveis `adb.exe` e `scrcpy.exe` para operar. Definir se o usuário precisa instalar ou se vêm juntos.
- **Opções apresentadas:**
  - *Opção A (Recomendada) — Embutido / Plug and Play:* Já inclui as versões testadas dos binários na pasta do app. Zero configuração pelo usuário e 100% offline.
  - *Opção B — Híbrido:* Checa no PC; se não achar, faz download na primeira execução.
  - *Opção C — Apenas Sistema:* Exige que o usuário instale e configure no `PATH`.
- **Resposta do Usuário:**
  > *"Embutido / Plug and Play (Já vem com tudo pronto, funciona direto sem configurar nada)"*
- **Decisão Final:** **Binários Embutidos (Plug and Play)**.

---

### ❓ Rodada 3 — Comportamento do Painel Durante o Espelhamento

- **Contexto:** Enquanto a tela de um celular estiver aberta, definir o que acontece com a janela principal de controle.
- **Opções apresentadas:**
  - *Opção A (Recomendada) — Painel Aberto:* Permite ver outros celulares e alternar em 1 clique.
  - *Opção B — Minimizar para a bandeja do sistema (perto do relógio).*
  - *Opção C — Mini-dock flutuante compacta.*
- **Resposta do Usuário:**
  > *"O painel fica aberto, mas se eu quiser, posso minimizar ele. A tela do celular não duplica, ou seja, se eu abrir a tela só do celular, o foco da tela desse celular, vai ficar só no celular flutuante, no painel, ele aparece ainda, só que com a tela em blur genérico com um aviso do tipo 'Janela flutuante aberta'. Ou algo assim. Se eu quiser fechar a janela flutuante do celular, posso fechar e a tela volta pro celular do painel."*
- **Decisão Final:** **Painel Aberto & Minimizável com Efeito de Blur no Card Ativo**.

---

### ❓ Rodada 4 — Exibição do Celular no Painel (Fase 1 vs Fase 2)

- **Contexto:** Como representar os celulares dentro do painel principal sem sobrecarregar a máquina.
- **Opções apresentadas:**
  - *Opção A (Recomendada para Fase 1) — Cartões Visuais com Status e Blur:* Cartões com informações do aparelho, miniatura/captura recente e efeito de blur quando a janela flutuante do scrcpy estiver aberta.
  - *Opção B — Vídeo ao Vivo Embutido no Painel (Fase 2 direto):* Renderização direta de frames de vídeo dentro da janela do Slint via FFI em C.
- **Resposta do Usuário:**
  > *"Opção A (Fase 1 com cartões visuais bonitos, status com blur e scrcpy na janela flutuante)"*
- **Decisão Final:** **Cartões Visuais com Status e Blur (scrcpy em janela flutuante)**.

---

### ❓ Rodada 5 — Configuração Padrão da Janela Flutuante

- **Contexto:** Parâmetros do scrcpy ao abrir o espelhamento de tela.
- **Opções apresentadas:**
  - *Opção A (Recomendada) — Modo Produtividade:* Sempre no topo (`always-on-top`), tela física do celular desligada (`--turn-screen-off` para economizar bateria e não esquentar), áudio reproduzido no PC (com botões de liga/desliga no painel).
  - *Opção B — Espelho Tradicional:* Janela comum do Windows, tela física acesa duplicando a imagem, áudio tocando no próprio celular.
- **Resposta do Usuário:**
  > *"Modo Produtividade (Sempre no topo, tela física apagada para economizar bateria e áudio no PC)"*
- **Decisão Final:** **Modo Produtividade Ativo por Padrão**.

---

### ❓ Rodada 6 — Identificação de Aparelhos e Apelidos

- **Contexto:** O Android muitas vezes reporta códigos técnicos de fábrica (ex: `SM-G991B`). Como apresentar os aparelhos para o usuário.
- **Opções apresentadas:**
  - *Opção A (Recomendada) — Nomes Amigáveis + Apelidos Personalizados:* Sugere o nome comercial, permite que o usuário dê um apelido amigável e salva em arquivo local para lembrar nas próximas conexões.
  - *Opção B — Direto ao Ponto:* Exibe apenas o código técnico bruto do aparelho.
- **Resposta do Usuário:**
  > *"Ele pode salvar com o nome padrão, depois me pergunta como quer que eu chame. De preferência, sempre que eu adicionar um novo, ele me pergunta como eu quero chamar eu dou um nome, mas por padrão, ele pega o nome que ele conseguiu arrancar tipo 'sm-q94r' ou algo assim."*
- **Decisão Final:** **Nome técnico inicial com prompt para definir apelido personalizado salvo localmente**.

---

### ❓ Rodada 7 — Controle de Navegação do Android (Voltar, Home, Recentes)

- **Contexto:** Como o usuário comanda as ações básicas do Android no computador.
- **Opções apresentadas:**
  - *Opção A (Recomendada) — Botões Virtuais no Painel + Cliques do Mouse:* Botões clicáveis no painel ([ Voltar ], [ Início ], [ Recentes ], [ Ligar/Desligar tela ]) somados aos cliques naturais do mouse na janela (botão direito = Voltar; botão do meio = Início).
  - *Opção B — Apenas Atalhos de Teclado:* Sem botões visuais, exigindo atalhos como `Alt + B`, `Alt + H`.
- **Resposta do Usuário:**
  > *"Botões Virtuais no Painel + Cliques Naturais do Mouse (Voltar com botão direito, Início com o meio e botões na tela)"*
- **Decisão Final:** **Botões no Painel + Cliques Naturais de Mouse**.

---

## 3. Comportamentos de Borda Acordados

1. **Desconexão Inesperada de Cabo USB:**
   - O aplicativo não trava e não exibe erros fatais na tela.
   - Encerra de forma limpa o processo scrcpy, remove o efeito de blur do card correspondente e atualiza o estado para *"Desconectado"*.
2. **Celular Bloqueado ou Não Autorizado:**
   - O painel exibe aviso didático claro orientando o usuário a desbloquear a tela do smartphone e tocar em *"Permitir depuração USB"*.
3. **Alternância Ultra-Rápida de Dispositivos (Fast Switching):**
   - Ao clicar em um novo celular, a sessão anterior é finalizada instantaneamente e a nova sobe de imediato, garantindo sempre a regra de **1 espelhamento ativo por vez**.

---

## 4. Identidade do Projeto

- **Nome Oficial:** **Mirro** (apenas Mirro).

---

## 5. Resumo da Tabela de Decisões

| Dimensão | Decisão Acordada | Impacto Técnico |
| :--- | :--- | :--- |
| **GUI Framework** | Slint + Rust | Arquivos `.slint` para UI e lógica de eventos em Rust. Zero WebViews. |
| **Dependências** | Embutidas (Bundled) | Pasta `tools/` com `adb.exe` e `scrcpy.exe` portáteis distribuídos juntos. |
| **Janela de Controle** | Painel aberto / minimizável | Mantém a visão geral de todos os aparelhos plugados no hub USB. |
| **Card do Aparelho** | Blur dinâmico | Card ganha máscara com aviso *"Janela flutuante aberta"* durante o uso. |
| **Janela Flutuante** | Modo Produtividade | `--always-on-top`, `--turn-screen-off`, áudio roteado para alto-falantes do PC. |
| **Nomes dos Celulares** | Apelidos com persistência | Arquivo `config.json` local salvando `serial -> alias`. |
| **Navegação** | Botões virtuais + Mouse | Ações visuais no painel e mapeamento do botão direito/meio no scrcpy. |
| **Troca de Celular** | 1 Clique (Fast Switching) | Encerramento rápido do processo anterior e subida imediata do novo. |
