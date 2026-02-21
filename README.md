# DOGL

**Dynamic Orchestration Graph Language** — an open notation for describing processes and orchestration. BPMN 2.0–compatible, extensible, and friendly to analysts and developers.

![DOGL mascot — Beagle](assets/dogl-mascot.png)

Process files use the **.dogl** extension.

## Features

- Human-readable text, suitable for git and code review
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
