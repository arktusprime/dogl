# Part 4 — Organization and practices

After [Part 1](01-basics.md), [Part 2](02-optional-codes-and-flows.md), and [Part 3](03-expressions-and-dmn.md) you have all elements, flows, and expressions. This part adds **structure** (pools, lanes, stages), a **full example**, **comments**, **reuse/BPMN**, and **good practices**.

**See also:** [Index (DSL_syntax.md)](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Organization: pools, lanes, stages (optional)

Structure is a **chain of optional levels**. Each level has a default (one implicit container) if you do not declare it; but **once you declare that level anywhere, every element must be placed inside some container of that level**.

### Optionality chain

| Level   | Required? | Default if not declared | Rule if declared |
|---------|-----------|--------------------------|------------------|
| **collab** | **Yes**   | — | Every file has at least one `collab`; all content belongs to a collab. |
| **pool**   | No       | 1 implicit pool         | If any `==` pool is declared in the collab, **every** element must lie inside some pool. |
| **lane**   | No       | 1 implicit lane per pool| If any `--` lane is declared in a pool, **every** element in that pool must lie inside some lane. |
| **stage**  | No       | 1 implicit stage        | Stage applies across lanes. You need not declare it in every lane; but if **any** `\|\|` stage is declared (in the collab/pool), **every** element must lie inside some stage. |

| Symbol | Level  | Meaning (example) |
|--------|--------|--------------------|
| `==`   | Pool   | Participant / organization |
| `--`   | Lane   | Role / department in a pool |
| `\|\|` | Stage  | Phase / stage in the process |

Elements live at **lane × stage** (each such intersection is a “quadrant”). The parser infers one implicit pool, one implicit lane, and one implicit stage when you write elements directly under `collab` without declaring `==`, `--`, or `\|\|`.

**Example without structure:**

```dogl
collab SimpleProcess

(s) Start
    => DoWork
[u] DoWork
    => End
(e) End
```

**Example with structure:**

```dogl
collab OrderProcess

== CustomerService
    -- Agent
        || Initiation
            (s) StartOrder
                => ReviewApplication
            [u] ReviewApplication
                => ProcessData
                => ApproveRequest
        || Processing
            [st] ProcessData
                => OrderDone
            (e) OrderDone
    -- Supervisor
        || Processing
            [u] ApproveRequest
                => Approved
            (e) Approved

== Warehouse
    -- Picker
        || Fulfillment
            [m] PickItems
                => QualityCheck
    -- Manager
        || Fulfillment
            [u] QualityCheck
                => FulfillmentDone
            (e) FulfillmentDone
```

---

## Full example

Order flow with two pools, optional codes, expressions, and data associations:

```dogl
collab OrderProcessing

== CustomerService
    -- Agent
        || Initiation
            (s) OrderReceived @do.message: order.created
                => ReviewOrder
            [u] ReviewOrder @do.exec: validateOrderData(order)
                => OrderValidation
            {d} OrderData
                .> ReviewOrder

        || Processing
            <x> OrderValidation
                order.isValid   => ProcessPayment
                =>d FixOrderIssues
            [u] FixOrderIssues
                => ManagerApproval

    -- Manager
        || Processing
            [u] ManagerApproval
                => Approved
            (e) Approved

== Warehouse
    -- Picker
        || Fulfillment
            [st] ProcessPayment @do.exec: processPayment(order.total)
                => UpdateInventory
            [st] UpdateInventory @do.exec: reserveItems(order.items)
                => PrepareShipment
            [u] PrepareShipment @do.timeout: 1d
                => OrderComplete
            (e) OrderComplete @do.notify: customer@email.com
            {db} InventoryDB
                .> UpdateInventory
```

---

## Comments and annotations

**Line comments** with `//`:

```dogl
// This is a comment
[u] ReviewTask   // Task-specific comment
```

**Annotations** (documentation) with `[[ ]]`:

```dogl
[[ Requires manager approval for orders over $10,000 ]]
[u] ManagerApproval
```

---

## Reuse and BPMN compatibility

**Call activity** (`@call` on a `[call]` task) maps to BPMN Call Activity: one process invokes another. For reusable **fragments** (the same sequence in several processes), the notation should stay compatible with BPMN (e.g. collapsed subprocess, referenced process). A dedicated reuse construct (e.g. reference or include) is under consideration and will align with BPMN semantics.

---

## Good practices

1. **Identifiers** — Use **PascalCase** for element names (e.g. `ReviewOrder`, `ProcessPayment`).
2. **Names** — Use clear, descriptive names for events, tasks, gateways.
3. **Structure** — Use pools and lanes to reflect participants and roles.
4. **Behavior** — Use `@do text` as placeholder; use `@do.*` for executable detail; use `@dmn` at gateways and `@call` for subprocesses.
5. **Errors** — Use `@do.error` (and event types) for error handling.
6. **Docs** — Use `[[ ]]` and `//` so that intent is obvious in review and diff.

---

**Back to:** [Index (DSL_syntax.md)](DSL_syntax.md)
