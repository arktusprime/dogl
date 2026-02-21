# DOGL

**Dynamic Orchestration Graph Language** — an open notation for describing processes, orchestration, integrated processes, and data flows. DOGL can model control flow and information exchange (who sends what to whom). The scope also includes **adapters**, **data transfer**, **message broker management**, **data storage**, and related concepts — not all of these parts are documented in the notation guide yet. BPMN 2.0–compatible, extensible, and friendly to analysts and developers.

![DOGL mascot — Beagle](assets/dogl-mascot.png)

Process files use the **.dogl** extension.

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

## Features

- Human-readable text, suitable for git and code review
- Processes, orchestration, integrated processes, and data flows (information exchange)
- Planned / partial: adapters, data transfer, message broker management, data storage
- BPMN 2.0 concepts (events, tasks, gateways, flows)
- Fast, predictable Rust parser
- Single JSON AST for use from Python, JS, Java, C#, and Rust

## Usage (Rust)

```rust
use dogl::parse;

let source = std::fs::read_to_string("process.dogl")?;
let ast = parse(&source)?;
```

## License

Dual-licensed under **[MIT](LICENSE-MIT)** or **[Apache-2.0](LICENSE-APACHE)** at your option.
