# Part 2 - Optional codes and flows

After [Part 1](01-basics.md) you can refine the surface notation with optional codes and additional connection forms.

**See also:** [Index](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Why codes are optional

The basic shapes are intentionally lightweight. When you need a more explicit BPMN-like surface form, you can add a code inside the shape.

These codes are still surface syntax. Canonical semantics should be interpreted through BPMN-aligned families such as `Event`, `Activity`, `Gateway`, `SequenceFlow`, `MessageFlow`, `Association`, and `DataAssociation`.

---

## Events

| Syntax | Surface meaning | BPMN-aligned direction |
| --- | --- | --- |
| `()` | inferred event | event-shaped `FlowNode` |
| `(s)` | start | `StartEvent` |
| `(i)` | intermediate | `IntermediateCatchEvent` or `IntermediateThrowEvent`, depending on meaning |
| `(e)` | end | `EndEvent` |

---

## Tasks

| Syntax | Surface meaning | BPMN-aligned direction |
| --- | --- | --- |
| `[]` | generic task | `Task` or generic activity |
| `[m]` | manual | `ManualTask` |
| `[u]` | user | `UserTask` |
| `[st]` | service | `ServiceTask` |
| `[rt]` | receive | `ReceiveTask` |
| `[se]` | send | `SendTask` |
| `[sc]` | script | `ScriptTask` |
| `[bu]` | business rule | `BusinessRuleTask` |
| `[call]` | call activity | `CallActivity` |

Some shorthand variants may remain convenient in notation, but canonical semantics should normalize to BPMN-valid task families rather than informal custom names.

---

## Gateways

| Syntax | Surface meaning | BPMN-aligned direction |
| --- | --- | --- |
| `<>` | default gateway form | gateway-shaped `FlowNode` |
| `<x>` | exclusive | `ExclusiveGateway` |
| `<p>` | parallel | `ParallelGateway` |
| `<i>` | inclusive | `InclusiveGateway` |
| `<c>` | complex | `ComplexGateway` |
| `<eb>` | event-based | `EventBasedGateway` |

---

## Artifact-like and data-like forms

Surface shorthand such as `{d}` or `{db}` is useful in notation, but explanatory prose should not collapse all `{...}` forms into one generic artifact bucket.

Recommended semantic interpretation:

| Syntax | Surface meaning | Canonical direction |
| --- | --- | --- |
| `{}` | generic artifact-like item | `Artifact` family if no stronger meaning exists |
| `{d}` | data object | `DataObjectReference` |
| `{db}` | data store | `DataStoreReference` |
| `{f}` | file-like data | data-related structure or explicit extension |
| `{doc}` | document-like artifact | `Artifact` or explicit extension, depending on meaning |
| `{msg}` | message-like item | explicit message-related artifact or integration-oriented extension |

The important rule is to keep canonical semantic taxonomy explicit instead of treating all `{...}` forms as the same semantic type.

---

## Connections beyond basics

In addition to `=>`, the notation may use:

| Syntax | Surface meaning | Canonical direction |
| --- | --- | --- |
| `=>d` | default branch | `SequenceFlow` marked as default |
| `->` | participant-crossing message relation | `MessageFlow` |
| `.>` | data linkage | `DataAssociation` |

Important distinction:

- `SequenceFlow` is process-owned control flow;
- `MessageFlow` is collaboration-owned interaction flow;
- `DataAssociation` is data linkage, not control flow.

These may share a common abstraction in code, but they should not be explained as one undifferentiated semantic `Flow`.

---

**Next:** [Part 3 - Expressions and DMN](03-expressions-and-dmn.md)
