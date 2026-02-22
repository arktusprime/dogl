# Part 2 — Optional codes and flows

After [Part 1 (basics)](01-basics.md) you can add **optional codes** to elements and use **extra flow types**. All of this is optional; basics are enough for many models.

**See also:** [Index (DSL_syntax.md)](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Why codes are optional

In Part 1 we used only shapes: `()`, `[]`, `<>`, `{}`. You can keep that. When you need an **explicit type** (e.g. “this is a start event” or “this is an XOR gateway”), you add a **code** inside the symbol. Codes are optional and can be introduced gradually.

---

## Events (optional codes)

| Syntax | Type         | When to use |
|--------|--------------|-------------|
| `()`   | (inferred)   | As in Part 1: start / intermediate / end from connectivity. |
| `(s)`  | Start        | Explicit start event. |
| `(i)`  | Intermediate | Explicit intermediate event. |
| `(e)`  | End          | Explicit end event. |

---

## Tasks (optional codes)

| Syntax   | Type          | Use case |
|----------|---------------|----------|
| `[]`     | Task          | Generic (default). |
| `[m]`    | Manual        | Human work, no system. |
| `[u]`    | User          | User task in an app. |
| `[st]`   | Service       | Automated service call. |
| `[rt]`   | Receive       | Wait for message/trigger. |
| `[se]`   | Send          | Send message. |
| `[sc]`   | Script        | Run script. |
| `[bu]`   | Business rule | Rule engine. |
| `[sm]`   | Send message  | Outbound message. |
| `[rm]`   | Receive message | Inbound message. |
| `[call]` | Call activity | Invokes another process (see Part 3). |

---

## Gateways (optional codes)

| Syntax | Type        | Behavior |
|--------|-------------|----------|
| `<>`   | (default)   | **OR** (inclusive: one or more paths). |
| `<x>`  | Exclusive   | Exactly one path (XOR). |
| `<p>`  | Parallel    | All paths (AND). |
| `<i>`  | Inclusive   | One or more paths (OR). |
| `<c>`  | Complex     | Custom condition logic. |
| `<eb>` | Event-based | Branch on event. |

---

## Artifacts (optional codes)

| Symbol  | Meaning |
|---------|---------|
| `{}`    | Artifact (default). |
| `{d}`   | Data. |
| `{db}`  | Database. |
| `{f}`   | File. |
| `{r}`   | Report. |
| `{doc}` | Document. |
| `{msg}` | Message. |
| `{e}`   | Email. |
| `{c}`   | Collection. |

---

## Flows beyond basics

In Part 1 we used only **`=>`** (sequence flow). You can add:

| Syntax | Use |
|--------|-----|
| **`=>d`** | **Default flow** from a gateway when it is not the first outgoing flow in the text. |
| **`->`** | **Message flow** between pools/participants. |
| **`.>`** | **Data association** (links an artifact to an activity). |

Rules unchanged: use **only the element identifier** in flows (e.g. `=> ReviewOrder`). **PascalCase** for identifiers.

**Next:** [Part 3 — Expressions and DMN](03-expressions-and-dmn.md) (@do, @dmn, @call; executable detail; DMN decision tables).
