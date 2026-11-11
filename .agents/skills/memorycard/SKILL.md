---
name: memorycard
description: >-
  Use this skill to create, inspect, update, or track tasks using the MemoryCard CLI.
  Also use reactively when the user requests a new feature, bugfix, or coding task
  without mentioning tasks, to check if a relevant task exists or offer to create one.
---

# MemoryCard — Gerenciamento de Tasks e Projetos

Este documento orienta o agente sobre como verificar a disponibilidade do **MemoryCard**, orientar a instalação caso necessário e operá-lo via linha de comando (`memorycard`) para gerenciar tarefas, acompanhar progresso e alinhar demandas com o usuário.

---

## 1. Pré-requisito: Verificação e Instalação do MemoryCard

Antes de executar operações do MemoryCard pela primeira vez em uma sessão, o agente deve verificar se o comando está acessível no terminal:

```bash
memorycard --version
```

### 1.1 Se o comando NÃO for reconhecido no terminal
O agente **não deve executar comandos de clonagem ou instalação silenciosamente**. Pare e faça esta pergunta direta ao usuário:

> *"O comando `memorycard` não foi detectado no PATH do seu terminal. Você já possui o repositório do MemoryCard clonado na sua máquina?"*

---

### 1.2 Fluxo baseado na resposta do usuário

#### Caso o usuário responda SIM (já possui o repositório clonado)
Oriente o usuário a acessar a pasta do repositório e rodar o `npm link`:

> *"Perfeito! Basta acessar a pasta do repositório no seu terminal e rodar `npm link` (certificando-se de que rodou `npm install` antes, se necessário):*
> ```bash
> cd <caminho-onde-voce-clonou>/memorycard
> npm install
> npm link
> ```
> *Isso tornará o comando `memorycard` acessível em qualquer lugar do seu terminal. Me avise assim que rodar para continuarmos!"*

#### Caso o usuário responda NÃO (ainda não clonou o repositório)
Forneça o link do repositório oficial no GitHub e as instruções passo a passo para clonar e linkar:

> *"Sem problemas! Você pode clonar o repositório oficial em [https://github.com/covil-works/memorycard](https://github.com/covil-works/memorycard) e disponibilizar o comando executando:*
> ```bash
> git clone https://github.com/covil-works/memorycard.git
> cd memorycard
> npm install
> npm link
> ```
> *Assim que concluir esses passos, o comando `memorycard` estará pronto para uso global. Me avise quando terminar para darmos sequência!"*

---

### 1.3 Validação e Continuidade
Assim que o usuário confirmar que executou os passos:
1. Valide no terminal rodando:
   ```bash
   memorycard --version
   ```
2. Se o comando responder com a versão, prossiga com o fluxo de trabalho normalmente.
3. Se o usuário preferir **não** instalar ou não utilizar o MemoryCard, siga diretamente com o desenvolvimento sem insistir.

---

## 2. Comportamento Reativo: Alinhamento Prévio de Task

O agente opera as tasks **sob comando do usuário**, mas deve agir de forma reativa no seguinte cenário:

> **Gatilho:** Quando o usuário solicitar uma implementação, nova feature, refatoração ou correção de bug **sem mencionar explicitamente tasks ou o MemoryCard**.

Nesse momento, antes de iniciar o código:

1. **Consultar o estado atual:**
   Execute silenciosamente `memorycard current` ou `memorycard list` para verificar se já existe uma task em andamento ou aberta sobre o assunto.

2. **Cenário A — Já existe uma task relacionada:**
   Pergunte ao usuário:
   > *"Encontrei a task #`<id>` ('`<título>`') em aberto relacionada a isso no MemoryCard. Deseja que eu atualize e siga por ela?"*

3. **Cenário B — Não há nenhuma task relacionada:**
   Pergunte ao usuário:
   > *"Não encontrei nenhuma task aberta sobre isso no MemoryCard. Deseja que eu crie uma task para acompanhar o trabalho ou prefere seguir direto sem registrar task?"*

4. **Diretriz de resposta:**
   - Se o usuário optar por **não** usar task: prossiga diretamente com a implementação sem insistir.
   - Se o usuário confirmar criar ou atualizar: execute os comandos correspondentes conforme a tabela abaixo.

---

## 3. Mapeamento de Intenções $\rightarrow$ Comandos CLI

Consulte esta tabela para saber exatamente qual comando executar quando o usuário pedir ações de gerenciamento:

| Intenção do Usuário | O que o agente deve fazer | Comando MemoryCard |
| :--- | :--- | :--- |
| **"Onde a gente parou?"** / **"Qual o status do trabalho?"** | Consultar a task ativa e contexto recente | `memorycard current`<br>*(ou `memorycard resume <id>` / `memorycard show <id>`)* |
| **"Cria uma task para [X]"** | Criar nova task com título e descrição opcional | `memorycard create "<título>" -d "<descrição>"` |
| **"Vamos detalhar o passo a passo da task"** | Adicionar itens ao checklist da task | `memorycard todo add <id> "<texto do passo>"` |
| **"Marca o passo [N] como feito"** | Marcar o checklist item N como concluído | `memorycard todo done <id> <n>` *(índice 1-based)* |
| **"Desmarca o passo [N]"** | Desmarcar o checklist item N | `memorycard todo undo <id> <n>` |
| **"Remove o passo [N]"** | Remover item do checklist | `memorycard todo remove <id> <n>` |
| **"Inicia a task [ID]"** / **"Move para em andamento"** | Mudar status da task para `in-progress` | `memorycard move <id> in-progress` |
| **"Finaliza a task [ID]"** / **"Conclui a task"** | Mudar status da task para `done` | `memorycard move <id> done` |
| **"Move a task [ID] para [status]"** | Mover para qualquer status/coluna válida | `memorycard move <id> <status>` |
| **"Registra uma nota/comentário na task"** | Registrar decisão ou histórico relevante | `memorycard comment <id> "<texto>"` |
| **"Lista as tasks"** / **"Quais tasks temos?"** | Listar todas as tasks do projeto | `memorycard list` |
| **"Quais são as tasks em aberto?"** | Filtrar tasks por status | `memorycard list -s todo`<br>`memorycard list -s in-progress` |
| **"Quais são as tasks mais antigas/recentes?"** | Listar com ordenação | `memorycard list --sort updated_at` |
| **"Mostra os detalhes da task [ID]"** | Exibir task completa com checklist e comentários | `memorycard show <id>` |
| **"Edita o título ou descrição da task [ID]"** | Atualizar campos da task | `memorycard edit <id> -t "<novo título>" -d "<nova desc>"` |
| **"Exclui a task [ID]"** | Excluir a task permanentemente | `memorycard delete <id> --yes` |
| **"Abre o board no navegador"** | Abrir a interface visual local do MemoryCard | `memorycard open` |
| **"Qual o projeto atual?"** | Consultar informações do projeto atual | `memorycard project` |

---

## 4. Guia Rápido de Referência da CLI

O comando `memorycard` está disponível no PATH do ambiente de terminal.

### 4.1 Consulta e Contexto
- `memorycard project` — Mostra o nome, ID, slug, modelo e colunas do projeto atual.
- `memorycard list [-s <status>] [--sort <updated_at|alphabetical|custom>]` — Lista as tasks.
- `memorycard show <id>` — Exibe o conteúdo completo da task.
- `memorycard current` — Retorna a task em `in-progress` atualizada mais recentemente.
- `memorycard resume <id>` — Retorna a task em formato resumido, ideal para retomar contexto.

### 4.2 Criação e Edição de Tasks
- `memorycard create [title] [-t <title>] [-d <description>] [-s <status>] [-m <model>]`
  - Cria uma nova task com ID numérico incremental automático.
  - Exemplo: `memorycard create "Tema escuro" -d "Implementar suporte ao tema dark via CSS variables"`
- `memorycard edit <id> [-t <title>] [-d <desc>] [-s <status>]`
  - Edita apenas os campos informados.
- `memorycard move <id> <status>`
  - Move a task para outra coluna (ex: `todo`, `in-progress`, `done`).
- `memorycard delete <id> [--yes|--force]`
  - Exclui a task. Em scripts ou comandos do agente, use `--yes` para evitar prompt interativo de confirmação.

### 4.3 Checklist (`todo`)
> [!IMPORTANT]
> Os índices `<n>` do checklist são **1-based** (o primeiro item é `1`, o segundo é `2`, etc.), correspondendo à ordem visual exibida no comando `memorycard show <id>`.

- `memorycard todo add <id> "<texto>"` — Adiciona item ao checklist.
- `memorycard todo done <id> <n>` — Marca item como concluído `[x]`.
- `memorycard todo undo <id> <n>` — Desmarca item `[ ]`.
- `memorycard todo remove <id> <n>` — Remove o item do checklist.

### 4.4 Comentários
- `memorycard comment <id> "<texto>"` — Adiciona comentário com timestamp local automático à seção `## Comments` da task.

### 4.5 Interface Visual
- `memorycard open` — Inicia o servidor local (caso necessário) e abre o board do projeto no navegador.
- Cada alteração feita pelo CLI é refletida em tempo real na interface web através de Server-Sent Events (SSE).
