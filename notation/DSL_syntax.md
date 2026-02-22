# DOGL syntax guide (index)

Text-based, BPMN-aligned notation for **processes**, **orchestration**, **integrations**, and **data flows** — including control flow and information exchange. The scope also includes **adapters**, **data transfer**, **message broker management**, **data storage**, and the like; not all of these are covered in the current parts yet. The guide is split into **four parts** with **increasing complexity**. Read in order if you are new; use the links to jump to a topic.

**Quick ref:** [cheat-sheet.md](cheat-sheet.md) (BASIC + ADVANCED)

---

## Reading order (simple → advanced)

| Part | Content |
|------|--------|
| **[Part 1 — Basics](01-basics.md)** | Simple concepts in detail: collab, four elements without codes `()` `[]` `<>` `{}`, flows with `=>` only, PascalCase, `@do` placeholder. One full example. |
| **[Part 2 — Optional codes and flows](02-optional-codes-and-flows.md)** | Optional letter codes for events, tasks, gateways, artifacts. Flows beyond basics: `=>d`, `->`, `.>`. |
| **[Part 3 — Expressions and DMN](03-expressions-and-dmn.md)** | Commands `@do`, `@dmn`, `@call`. Advanced `@do.*` (executable detail). DMN decision tables (at gateway and notation). |
| **[Part 4 — Organization and practices](04-organization-and-practices.md)** | Pools, lanes, stages. Full example. Comments and annotations. Reuse and BPMN compatibility. Good practices. |

---

## At a glance

- **Basics:** `collab Name` → elements `()` `[]` `<>` `{}` with PascalCase names → connect with `=>` (indented under each element). Default flow from gateway = first in text.
- **Optional:** Add codes `(s)`, `[u]`, `<x>`, `{d}` etc.; add `=>d`, `->`, `.>` when needed.
- **Expressions:** `@do` (placeholder or `@do.exec` etc.), `@dmn` (gateway), `@call` (call activity). Only on the element, not on the arrow.
- **Structure:** Optional `==` pool, `--` lane, `||` stage.

Start with [Part 1](01-basics.md) for a detailed introduction to the simplest concepts.
