# DOGL

**Dynamic Orchestration Graph Language** — an open language for **business processes** and **orchestration programs**, from **RPA** to **Hyperautomation hub**. DOGL describes processes, orchestration, integrations, and data flows (control flow and information exchange). The scope also includes adapters, data transfer, message broker management, data storage, and related concepts — not all parts are documented in the notation guide yet. BPMN 2.0–compatible and extensible.

![DOGL mascot — Beagle](assets/dogl-mascot.png)

Process files use the **.dogl** extension.

## Why DOGL

- **Human-friendly, git-friendly, no-code / low-code** — Plain text, readable by analysts; version and review as code; start with shapes and flows only, add detail when needed.
- **Machine-friendly and AI-friendly** — Easy to parse, unambiguous structure (AST); convenient for tools, runtimes, and AI (analysis, generation, refactoring).
- **Graph-based** — Processes are directed graphs (nodes = elements, edges = flows). This model is a well-established foundation for workflow control flow and verification (e.g. Petri nets, workflow nets, BPMN); DOGL is aligned with it and supports validation and export to diagrams.
- **One source, many uses** — Same `.dogl` file for diagrams, validation, execution, and integration; single JSON AST for Rust, Python, JS, Java, C#.
- **Analyst-first** — Designed so analysts can write and change processes without heavy tools; comments, traceability, and optional complexity (basics without codes, then optional codes and expressions).
- **Extensible** — From simple flows to DMN decisions, call activities, and (planned) adapters, data transfer, and message brokers.

## Quick start

No codes, no expressions — just shapes and flows. Four shapes: `()` event, `[]` task, `<>` gateway, `{}` artifact. Connect with `=>` on an indented line under each element. Names in **PascalCase**.


```dogl
mod HelloProcess

() Start
    => Task
[] Task
    => End
() End
```

*Same process — text (above) and diagram (below) are equivalent.*

![DOGL simple process 1](assets/simple1.jpg)

Save as `.dogl`. 

## Technical

- BPMN 2.0 concepts (events, tasks, gateways, flows); DMN for decisions
- Fast, predictable Rust parser; single JSON AST for Python, JS, Java, C#, Rust

## Usage (Rust)

```rust
use dogl::parse;

let source = std::fs::read_to_string("process.dogl")?;
let ast = parse(&source)?;
```

## License

Dual-licensed under **[MIT](LICENSE-MIT)** or **[Apache-2.0](LICENSE-APACHE)** at your option.
