# DOGL cheat sheet

Quick ref. Full guide: [DSL_syntax.md](DSL_syntax.md).

---

# BASIC (page 1)

No codes. Default flow = first in text. Identifiers in **PascalCase**.

## File start

```dogl
collab ProcessName
```

## Elements (no codes)

| Symbol | Element  | Notes |
|--------|----------|--------|
| `()`   | Event    | Type from inputs/outputs (no in = start, no out = end) |
| `[]`   | Task     | Generic task |
| `<>`   | Gateway  | Default **OR** (one or more paths) |
| `{}`   | Artifact | Data/document |

## Flows

Only **`=>`**. Each connection on a **new line, indented** under the element. Use **only the element name** (e.g. `=> ReviewOrder`). **Default flow** from a gateway = **first** outgoing flow in the text.

## Behavior placeholder

**`@do`** + text (e.g. `@do check amount`) — documents intent. **Does not execute** until you add a qualifier later (e.g. `@do.exec: ...`). Gateway routing = DMN (match + default `_`), not a separate condition.

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

## Comments

- `//` line comment  
- `[[ ... ]]` annotation

---

# ADVANCED (page 2)

Optional codes and flows. Three commands: **@do** · **@dmn** · **@call**.

## Optional element codes

| Element  | Optional codes |
|----------|-----------------|
| Events   | `(s)` start · `(i)` intermediate · `(e)` end |
| Tasks    | `[]` · `[m]` manual · `[u]` user · `[st]` service · `[rt]` receive · `[se]` send · `[sc]` script · `[bu]` business rule · `[sm]` send msg · `[rm]` receive msg · `[call]` call activity |
| Gateways | `<>` OR · `<x>` XOR · `<p>` AND · `<i>` OR · `<c>` complex · `<eb>` event-based |
| Artifacts| `{}` · `{d}` `{db}` `{f}` `{r}` `{doc}` `{msg}` `{e}` `{c}` |

## Optional flows

| Arrow | Meaning |
|-------|---------|
| `=>d` | Default flow (exactly one required at DMN gateway) |
| `->`  | Message flow (between pools) |
| `.>`  | Data association (artifact ↔ activity) |

## Expressions

| @ | Purpose |
|---|---------|
| **@do** | Placeholder: `@do text`. Executable: `@do.exec`, `@do.timer`, `@do.timeout`, `@do.notify`, etc. Gateway routing = DMN only. |
| **@dmn:** | Gateway: DMN decision (ID or path) |
| **@call:** | Call activity: another process (ID or path); use on `[call]` task |
| `@~...` | Disable (e.g. `@~do.exec: ...`) |

## Structure (optional)

| Symbol | Level |
|--------|--------|
| `==` | Pool |
| `--` | Lane |
| `\|\|` | Stage |

## DMN decision (match-style)

Result = **element name** (where the gateway routes to). Write only the conditions you need per row; no padding. Default (catch-all) row = **`=>d ElementName`** (required; same line is catch-all rule and default flow). “any”.

```dogl
dmn RouteByClientType

  ClientType = "New"     => SalesRepTask
  ClientType = "VIP"     => VIPAccountManagerTask
  =>d AccountManagerTask
```

The table **does** decide where to go. For that to work at runtime, the input (e.g. ClientType) must already be in the **process instance** — set or loaded by some earlier step. **Two options:** (1) **Inline** — rules directly under the gateway (no `@dmn`). (2) **Reference** — `dmn DecisionName` elsewhere, gateway has `@dmn: "DecisionName"`.

Reference from gateway: `@dmn: "RouteByClientType"`. Flows are inferred from the table. For inline syntax and several inputs (columns), see Part 3. The gateway’s outgoing flows are inferred from the table.