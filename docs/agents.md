# Guia de Especificação e Redação de Regras para Agentes

Este documento orienta os agentes de IA sobre como registrar, atualizar e manter regras técnicas e de design na documentação do projeto Mirro.

---

## 1. Proibição Absoluta de Emojis na Documentação

* **REGRA FUNDAMENTAL:** É estritamente proibido o uso de qualquer tipo de emoji em todos os arquivos de documentação dentro de `/docs` (incluindo títulos, listas, tabelas, blocos de código ou exemplos).
* A documentação técnica deve ser sóbria, formal e direta, sem ornamentos visuais, ilustrações informais ou ícones decorativos.
* Utilize apenas formatação padrão Markdown (negrito, itálico, listas, tabelas, links e citações).

---

## 2. Princípio de Agrupamento por Afinidade

* **PROIBIDO:** Criar um arquivo Markdown para cada regra individual.
* **OBRIGATÓRIO:** Agrupar múltiplas regras afins dentro do mesmo arquivo temático na pasta do tópico correspondente (exemplo: `/docs/ui/visual-standards.md`).
* Cada arquivo deve ser desenhado para comportar regras atuais e futuras do mesmo assunto.

---

## 3. Padrão de Identificador de Regras (Rule ID)

Toda regra deve possuir um identificador único e permanente para fins de rastreabilidade:

Formato: `<nome-do-arquivo>-<numero-sequencial-com-3-digitos>`

### Exemplos:
* No arquivo `visual-standards.md`:
  * `visual-standards-001`
  * `visual-standards-002`
  * `visual-standards-003`
* No arquivo `navigation.md`:
  * `navigation-001`
  * `navigation-002`

---

## 4. Estrutura Padrão de Cada Regra

Toda regra inserida em um documento deve seguir a estrutura abaixo:

```markdown
### [ID-DA-REGRA] Titulo Curto e Imperativo

* **Severidade:** Obrigatorio | Recomendado
* **Descricao:** Explicacao direta e objetiva do que deve ou nao ser feito. Foco na funcao e sem rodeios.
* **Justificativa:** Razao tecnica, de UX ou de arquitetura pela qual a regra existe.
* **Incorreto (Anti-padrao):**
  - Exemplo do que NAO fazer.
* **Correto (Padrao Aprovado):**
  - Exemplo do que DEVE ser feito.
```

---

## 5. Diretrizes Gerais de Redacao

1. **Sem Emojis:** Nunca insira emojis em cabeçalhos, marcadores ou textos explicativos.
2. **Tom Direto e Técnico:** Evite textos promocionais, adjetivos vagos ou prolixidade. Seja cirúrgico.
3. **Imutabilidade de IDs:** O ID atribuído a uma regra nunca deve ser reutilizado ou alterado após publicado, mesmo que a regra seja depreciada.
4. **Atualização do Índice:** Caso crie uma nova pasta de tópicos em `/docs`, atualize imediatamente o arquivo [`/docs/index.md`](file:///C:/Users/artue/OneDrive/Documentos/GitHub/mirro/docs/index.md).
