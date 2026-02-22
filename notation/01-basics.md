# Part 1 — Simple concepts (basics)

Read this first. No codes, no expressions — only the core ideas. Complexity increases in later parts.

**See also:** [Index (DSL_syntax.md)](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md) · [Quick start](QUICK_START.md)

---

## What DOGL is

DOGL describes **processes**, **orchestration**, **integrations**, and **data flows** in plain text — including who does what and how information is exchanged. The scope also extends to **adapters**, **data transfer**, **message broker management**, **data storage**, and similar topics (some of these are not yet documented in the guide). Readable, diff-friendly, editable in any editor, versionable in git. A file can contain **one or more** collabs. For diagram-only models this part is enough; everything else is optional.

---

## File start: collab

A **collab** is introduced with **`collab`** and a name:

```dogl
collab ProcessName
```

You can have **several `collab`** in one file; each starts a new collab. Use a name that identifies the process or process group (e.g. `OrderProcess`, `Onboarding`). All elements and flows that follow belong to this collab until the next `collab` or end of file.

**In the AST**, a collab contains a set of **Pools** (and their lanes, stages, and elements). If you don’t use pools (Part 4), the collab still has one implicit pool with one lane and one stage.

---

## Four elements (no codes)

In the basic notation you use **only the shape** — no letters inside. The meaning comes from the symbol:

| Symbol | Element  | Meaning |
|--------|----------|---------|
| `()`   | **Event**   | Something that happens. Type is inferred: no incoming flow = **start**; no outgoing = **end**; otherwise **intermediate**. |
| `[]`   | **Task**    | Work to be done. A generic task (no subtype). |
| `<>`   | **Gateway** | Splits or merges the flow. Default is **OR** (inclusive: one or more paths can be taken). |
| `{}`   | **Artifact**| Data or document used in the process. |

You write the symbol followed by the **element name** (identifier), e.g. `() Start`, `[] ReviewOrder`, `<> CheckAmount`, `{} OrderData`. Names should be in **PascalCase** (e.g. `ReviewOrder`, not `review_order`).

---

## Connecting elements: flows (basics)

To show that control passes from one element to another, use the **sequence flow** arrow: **`=>`**.

**Rules:**

1. Write each flow on a **new line**, **indented** under the element it comes from.
2. After `=>` write **only the name** of the target element (e.g. `=> ReviewOrder`). Do not repeat the element type.
3. **Default flow** from a gateway: if you don’t mark one explicitly, the **first** outgoing flow in the text is treated as the default. In basics you don’t need a special symbol for that.

Every element you reference in a flow must be **declared** in the same pool (with its symbol and name). If you don't declare pools, everything is in one collab and the collab has one implicit pool anyway. **Sequence flow** (`=>`) cannot cross pools — BPMN allows it only within one pool; cross-pool is message flow (`->`, see Part 2). End events have no outgoing flows.

---

## Element identifiers: PascalCase

Use **PascalCase** for all element names: `Start`, `ReviewOrder`, `CheckAmount`, `ApproveOrder`. This keeps names consistent and easy to read in flows.

---

## Behavior placeholder: @do

You can attach a short note to an element with **`@do`** and some text, e.g. `@do check amount`. This is a **placeholder** for future executable behavior: it documents intent but **does not execute**. To make behavior executable later, you add a qualifier (e.g. `@do.exec: ...`) or, at a gateway, a DMN table (Part 3).

---

## Full example (basics only)

Here is a complete process using only what we’ve described: collab, four shapes without codes, and `=>` flows. The gateway has two outgoing flows; the first is the default.

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

**Next:** [Part 2 — Optional codes and flows](02-optional-codes-and-flows.md) (event/task/gateway/artifact codes; default flow, message flow, data association).
