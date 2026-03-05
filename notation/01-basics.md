# Part 1 - Basics

Read this first. This part introduces the simplest surface syntax of DOGL.

**See also:** [Index](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## What this part covers

This part explains the **surface notation** only:

- `collab`
- the four basic shapes
- simple `=>` connections
- PascalCase identifiers
- `@do` as a non-executable placeholder

This part does **not** define the canonical semantic model. Internally, DOGL lowers syntax-facing structures into BPMN-aligned semantic concepts such as `Collaboration`, `Participant`, `Process`, `FlowNode`, and `SequenceFlow`.

---

## File start: `collab`

A DOGL source starts a collaboration block with `collab`:

```dogl
collab ProcessName
```

One file may contain multiple `collab` blocks. At the semantic level, these lower into one or more `Collaboration` instances inside `DoglFile`.

---

## Four basic shapes

At the syntax level, the simplest shapes are:

| Symbol | Surface meaning | Semantic direction |
| --- | --- | --- |
| `()` | Event | Lowers to an event-shaped `FlowNode` |
| `[]` | Task | Lowers to an activity-shaped `FlowNode` |
| `<>` | Gateway | Lowers to a gateway-shaped `FlowNode` |
| `{}` | Artifact-like item | Lowers to an artifact or data-related structure depending on later typing |

Examples:

- `() Start`
- `[] ReviewOrder`
- `<> CheckAmount`
- `{} OrderData`

Identifiers should use **PascalCase**.

---

## Basic connections: `=>`

In the basic notation, `=>` means a process-internal connection:

```dogl
() Start
    => ReviewOrder
```

Rules:

1. Write each outgoing connection on a new indented line under the source node.
2. Use only the target identifier after the arrow.
3. In the simplest reading, the first gateway branch is treated as the default if nothing more explicit is written.

At the semantic level, this kind of connection lowers to `SequenceFlow` inside a `Process`.

---

## About structure in this part

This part intentionally stays flat. Later parts introduce optional surface syntax for participant and lane organization.

Important distinction:

- syntax may allow compact authoring conveniences;
- canonical semantics still normalize into BPMN-aligned types;
- DOGL-only concepts such as stage-like or quadrant-like grouping, if retained, are extensions rather than core BPMN semantic structure.

---

## Placeholder behavior: `@do`

`@do` with plain text is a placeholder for intent, not executable behavior:

```dogl
[] ReviewOrder @do check amount
```

This is documentation-oriented syntax at this stage. It should not be read as proof that execution semantics already exist in the core platform.

---

## Example

```dogl
collab OrderProcess

() Start
    => ReviewOrder
[] ReviewOrder
    => CheckAmount
<> CheckAmount
    => ApproveOrder
    => RejectOrder
[] ApproveOrder
    => End
() End
```

This example is about surface syntax. A later phase of the platform resolves it into a BPMN-aligned semantic model.

---

**Next:** [Part 2 - Optional codes and flows](02-optional-codes-and-flows.md)
