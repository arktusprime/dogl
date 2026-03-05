# DOGL cheat sheet

Quick reference for surface syntax. Full guide: [DSL_syntax.md](DSL_syntax.md).

---

## Basic surface syntax

```dogl
collab ProcessName

() Start
    => ReviewOrder
[] ReviewOrder
    => End
() End
```

### Basic shapes

| Syntax | Surface meaning |
| --- | --- |
| `()` | event-like node |
| `[]` | task-like node |
| `<>` | gateway-like node |
| `{}` | artifact-like item |

### Basic connection

| Syntax | Meaning |
| --- | --- |
| `=>` | process-internal connection |

---

## Optional syntax

### Codes

| Syntax | Direction |
| --- | --- |
| `(s)` | `StartEvent` |
| `(i)` | intermediate event |
| `(e)` | `EndEvent` |
| `[u]` | `UserTask` |
| `[st]` | `ServiceTask` |
| `[bu]` | `BusinessRuleTask` |
| `[call]` | `CallActivity` |
| `<x>` | `ExclusiveGateway` |
| `<p>` | `ParallelGateway` |
| `<i>` | `InclusiveGateway` |
| `<eb>` | `EventBasedGateway` |

### Connections

| Syntax | Canonical direction |
| --- | --- |
| `=>d` | default `SequenceFlow` |
| `->` | `MessageFlow` |
| `.>` | `DataAssociation` |

### Commands

| Syntax | Meaning |
| --- | --- |
| `@do` | placeholder or behavior attachment |
| `@dmn` | decision reference |
| `@call` | call-activity reference |
| `@~...` | disabled form |

### Structure

| Syntax | Surface role |
| --- | --- |
| `==` | participant-like grouping |
| `--` | lane-like grouping |
| `\|\|` | stage-like DOGL extension |

---

## Semantic reminder

This sheet is about **syntax**.

Canonical semantics should be read through BPMN-aligned types such as:

- `DoglFile`
- `Collaboration`
- `Participant`
- `Process`
- `FlowNode`
- `SequenceFlow`
- `MessageFlow`

DOGL-specific constructs such as stage-like grouping remain extensions rather than BPMN-native core semantics.
