# Part 3 — Expressions and DMN

After [Part 1 (basics)](01-basics.md) and [Part 2 (optional codes and flows)](02-optional-codes-and-flows.md) you can add **behavior and conditions** (expressions) and **DMN decision tables**. This part is for executable or decision-heavy models.

**See also:** [Index (DSL_syntax.md)](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Expressions: only on the element

**Only the element** carries expressions — the flow arrow (`=>`) does not. For diagram-only models you can omit expressions or keep using **`@do`** + text as a placeholder (Part 1). **`@do` without a dot does not execute**; it only documents intent.

For executable behavior or linking to decisions and subprocesses, DOGL uses three **native BPMN-style commands**:

| Command   | Use |
|-----------|-----|
| **@do**   | General behavior. Use **`@do text`** as placeholder; for execution use **`@do.qualifier: ...`** (see Advanced below). |
| **@dmn**  | At a **gateway**: reference to a DMN decision (ID or path). Routing is defined by the decision table. |
| **@call** | **Call activity**: this element invokes another process. Value = process ID or path. Use on a `[call]` task. |

**Examples:**

```dogl
<x> RouteOrder @dmn: "OrderRouting"
    =>d ManualReview

[call] RunRefundProcess @call: "RefundProcess"
    => Done
```

---

## Advanced — @do.* (executable detail)

All executable detail is under **@do** with a **dot and a qualifier**. Use only when you need executable behavior.

| Form | Purpose |
|------|---------|
| `@do.exec: ...` | Execute code / call service |
| `@do.timer: ...` | Time-based trigger (events) |
| `@do.message: ...` | Message trigger |
| `@do.signal: ...` | Signal trigger |
| `@do.error: ...` | Error handling |
| `@do.webhook: ...` | Webhook trigger |
| `@do.store: ...` · `@do.update: ...` · `@do.query: ...` · `@do.field: ...` | Data operations |
| `@do.timeout: ...` · `@do.track: ...` · `@do.listen: ...` · `@do.notify: ...` | Utility |

Disable an expression: prefix with **`@~`** (e.g. `@~do.exec: old()`).

**Example:**

```dogl
[st] ValidateOrder @do.exec: validateOrder(order.id)
    => AmountGateway

<x> AmountGateway
  order.amount > 1000   => HighValueReview
  =>d StandardReview
[] HighValueReview
    => FollowUp
[] StandardReview
    => FollowUp

(i) TimerEvent @do.timer: 24h
    => FollowUp

[u] ReviewTask @do.timeout: 2d @do.notify: manager@company.com
    => Done
```

---

## DMN decision table at a gateway

A gateway can use a **DMN decision table** for routing. **Outgoing flows are always taken from the table** (the set of element names on the right-hand side of the rules); you do not list them at the gateway. You **must** specify exactly one default flow: **`=>d`** *DefaultTarget* (required for DMN match).

You can define the table in two ways:

### Option 1: Inline (table under the gateway)

Put the decision rules **directly under the gateway**, indented. Same rule syntax as in a separate `dmn` block (conditions on the left, **`=>`** element name on the right). No **`@dmn:`** on the gateway. Good for small, one-off decisions.

```dogl
<x> RouteOrder
  order.amount > 10000                 => HighValueReview
  customer.tier = "gold"               => AutoApprove
  =>d ManualReview
```

### Option 2: Reference (separate decision block)

Define the table in a **`dmn DecisionName`** block (elsewhere in the file or in another file). At the gateway, use **`@dmn:`** with the decision ID or path. Good for reusable or large tables.

```dogl
<x> RouteOrder @dmn: "OrderRoutingTable"
    =>d ManualReview
```

---

## DMN decision notation (defining the table)

Decision tables use **match style**: conditions on the left, **`=>`** and the result on the right. Variables come from the process instance (set or loaded by earlier steps); no separate declaration.

Each row: conditions **`=>`** target element name. For the default (catch-all) row use **`=>d ElementName`** — that line is both the catch-all rule and the required default flow. The **first matching rule** wins; put the most specific rules first, default last.

**Right-hand side** is the target element — the process element the gateway routes to. Use the element name (e.g. `HighValueReview`), not the type.

- **Standalone block:** `dmn DecisionName` — then rules one per line, indented. Reference from gateway with **`@dmn: "DecisionName"`**; the gateway must have exactly one **`=>d`** *DefaultTarget*.
- **Inline:** same rule syntax under the gateway (no `dmn` header, no `@dmn`); the catch-all row is the required **`=>d`** *DefaultTarget* (see Option 1 above).

**Example (standalone block):**

```dogl
dmn OrderRouting

  order.amount > 10000                 => HighValueReview
  customer.tier = "gold"               => AutoApprove
  =>d ManualReview
```

The gateway’s outgoing flows are **inferred from** this table. At runtime the engine evaluates rules in order using the process instance; the first matching rule gives the target element.

**Next:** [Part 4 — Organization and practices](04-organization-and-practices.md) (pools, lanes, stages; full example; comments; reuse; good practices).
