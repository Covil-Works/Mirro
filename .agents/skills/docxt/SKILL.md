---
name: docxt
description: Inicializa e governa a documentação do repositório em /docs usando AGENTS.md, índice temático por arquivo e regras com IDs sequenciais únicos.
---

# Skill: docxt

## Diretrizes Universais de Execução
1. **Caminhos Relativos:** Todas as referências e escritas devem ser relativas à raiz do repositório (ex.: `/docs/ui/patterns.md`). Nunca gere ou utilize caminhos absolutos do sistema operacional.
2. **Precedência:** O que está documentado em `/docs` é a verdade absoluta do projeto e se sobrepõe a qualquer código legado existente.
3. **Não-Destrutividade:** Nunca apague, sobrescreva ou altere regras existentes sem autorização explícita do usuário.
4. **Adição Sob Demanda:** Nunca crie regras por iniciativa própria sem comando explícito (ex.: "anota isso", "adicione à documentação", "/docxt add").

---

## Modos de Operação

### Modo 1: `init` (ou `/docxt init`)
Execute esta rotina para criar o scaffold inicial de documentação do repositório.

**Ações ordenadas:**
1. Crie o arquivo `AGENTS.md` na raiz do projeto contendo:
   - Apontamento de que a leitura prévia de `/docs/index.md` é obrigatória antes de qualquer implementação.
   - Declaração de precedência das regras de `/docs` sobre código legado.
   - Proibição de alterar regras sem consulta prévia ao usuário.
   - Apontamento para `/docs/AGENTS.md` para diretrizes de escrita.
2. Crie a pasta `/docs`.
3. Crie `/docs/index.md` contendo:
   - Apenas o mapeamento das pastas de contexto e seus respectivos arquivos temáticos em `kebab-case`.
   - Para cada arquivo, apenas uma descrição concisa do escopo/tema que ele cobre (sem listar regras individuais).
4. Crie `/docs/AGENTS.md` contendo:
   - Regra de nomes em `kebab-case`.
   - Proibição de caminhos absolutos.
   - Princípio de agrupamento (não criar um arquivo por regra).
   - Especificação do formato mínimo de regra: Título, ID Único (`[nome-do-arquivo]-[sequencial_3_digitos]`) e Descrição Breve.
   - Regra de ciclo de vida (novas adições apenas sob demanda e substituições apenas com confirmação).

---

### Modo 2: `add` (ou `/docxt add <instrução>`)
Execute quando o usuário solicitar o registro ou anotação de uma nova regra.

**Ações ordenadas:**
1. Inspecione `/docs/index.md` e localize o contexto e arquivo temático mais adequado.
2. Se o arquivo já existir:
   - Identifique o último ID sequencial utilizado no arquivo.
   - Incremente a numeração em formato de 3 dígitos (ex.: `001` -> `002`).
   - Insira a nova regra ao final do arquivo com: Título, ID e Descrição concisa.
3. Se a regra exigir um novo arquivo ou contexto:
   - Crie o novo arquivo `.md` em `kebab-case` na pasta apropriada.
   - Insira a regra com o ID inicial `001`.
   - Atualize `/docs/index.md` adicionando a linha do novo arquivo com um resumo de uma linha sobre o tema coberto.
4. Informe ao usuário o arquivo modificado e o ID gerado.

---

### Modo 3: Tratamento de Conflitos
Execute sempre que uma tarefa de código solicitada pelo usuário contradisser uma regra existente em `/docs`:

**Ações ordenadas:**
1. Pause a execução e cite explicitamente o ID e o resumo da regra conflitante.
2. Pergunte ao usuário se a regra antiga deve ser substituída ou mantida como exceção.
3. Se confirmada a substituição: remova a regra antiga e crie a nova com um **novo ID sequencial**. Nunca reutilize o ID da regra anterior.
