# DOGL

**Dynamic Orchestration Graph Language** is a library-first platform for describing orchestration definitions in plain text.

The current architectural direction is:

- a human-readable `.dogl` source format;
- a syntax-facing language front end with source fidelity and diagnostics;
- a BPMN-aligned semantic model;
- validation, machine-readable interchange, and BPMN adapters around a stable core;
- embeddable language, render, and editor surfaces as the platform grows.

## Status

The project is in early development (`0.0.0`).

The current priority is architectural convergence:

- repository and crate structure;
- syntax and semantic boundaries;
- BPMN-aligned AST and semantic-domain structures;
- stable contracts for later parsing, validation, rendering, and import/export work.

Runtime, execution, hyperautomation, and broad integration concerns remain future extension areas rather than the current architectural center.

![DOGL mascot - Beagle](assets/dogl-mascot.png)

## Quick start

At the surface-syntax level, a simple DOGL file can look like this:

```dogl
collab HelloProcess

() Start
    => Review
[] Review
    => End
() End
```

This is syntax, not the canonical semantic model.

Inside the platform, syntax-facing structures are lowered into a BPMN-aligned semantic shape built around concepts such as:

- `DoglFile`
- `Collaboration`
- `Participant`
- `Process`
- `FlowNode`
- `SequenceFlow`
- `MessageFlow`

See:

- [notation/DSL_syntax.md](notation/DSL_syntax.md) for the notation guide;
- [rd/arch/design88.md](rd/arch/design88.md) for project structure;
- [rd/arch/arch88.md](rd/arch/arch88.md) for architecture;
- [rd/arch/design88-1-AST.md](rd/arch/design88-1-AST.md) for the BPMN-aligned AST and semantic structure.

## Why DOGL

- **Readable source**: plain-text process definitions that are easier to review and diff than large XML artifacts.
- **Library-first architecture**: the primary product is an embeddable platform, not a monolithic application.
- **Clear semantic layering**: syntax-facing structures, semantic lowering, validation, and adapters are treated as separate concerns.
- **BPMN-aligned core**: the semantic model uses BPMN-valid concepts where BPMN provides the right meaning.
- **Extensible platform direction**: rendering, editor capabilities, bindings, import/export, and future runtime work can grow around the same core.

## Current platform shape

The intended project structure is converging toward:

- `dogl-language` for the embeddable language core;
- `dogl-render` for the embeddable render core;
- `dogl-editor` for the embeddable editor core;
- `dogl-adapters` for BPMN and other external integrations;
- additional delivery surfaces such as CLI, WASM, and bindings.

Not every host application will need every part. The architecture is designed so consumers can embed only the layers they need.

## Scope notes

DOGL may eventually support broader workflow and automation scenarios, but the current documentation and implementation work should be read through this rule:

- the authoritative current scope is the library platform and its semantic foundations;
- future runtime, orchestration, and hyperautomation capabilities are extension tracks, not proof that those parts are already implemented.

## License

Dual-licensed under **[MIT](LICENSE-MIT)** or **[Apache-2.0](LICENSE-APACHE)** at your option.
