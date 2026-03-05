# Part 3 - Expressions and DMN

After [Part 1](01-basics.md) and [Part 2](02-optional-codes-and-flows.md), you can add richer behavioral annotations and decision-oriented syntax.

**See also:** [Index](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Scope of this part

This part describes notation for:

- element-level expressions;
- decision references;
- call-activity references;
- DMN-like routing tables in surface syntax.

This part does **not** prove that the runtime behavior is already implemented in the current library core. Execution and orchestration remain later platform or extension concerns.

---

## Expressions live on nodes, not on arrows

Expressions attach to element syntax, not to the connection token itself.

Examples:

- `@do`
- `@dmn`
- `@call`
- qualified forms such as `@do.exec`

At the semantic level, these should be modeled as explicit expression or integration attachments on semantic nodes, not as ad hoc flow metadata.

---

## `@do`

`@do` is the general notation for intent or behavior attachment.

Two important readings:

- `@do text` is a lightweight placeholder and documentation aid;
- qualified forms such as `@do.exec: ...` are more execution-oriented notation, but they should still be treated as language constructs or extensions rather than proof of a complete runtime engine in the current core.

Example:

```dogl
[] ReviewOrder @do check amount
[] ValidateOrder @do.exec: validateOrder(order.id)
```

---

## `@dmn`

`@dmn` links routing logic to an explicit decision definition or decision table.

Example:

```dogl
<x> RouteOrder @dmn: "OrderRouting"
    =>d ManualReview
```

Important architectural rule:

- DMN is not a BPMN flow-node family;
- DMN-related constructs should remain explicit DOGL decision integrations or extensions rather than being disguised as BPMN-native node types.

---

## `@call`

`@call` links a call-activity-like element to another process or reusable process definition.

Example:

```dogl
[call] RunRefundProcess @call: "RefundProcess"
    => Done
```

This should be interpreted through BPMN-aligned `CallActivity` semantics rather than as a generic free-form command.

---

## Qualified `@do.*` forms

Qualified forms may be used when the notation needs more specific intent.

Examples:

- `@do.exec: ...`
- `@do.timer: ...`
- `@do.message: ...`
- `@do.signal: ...`
- `@do.error: ...`
- `@do.timeout: ...`
- `@do.notify: ...`

These forms are useful as surface notation, but they should be documented as language-level attachments, not as guarantees that all corresponding runtime behaviors already exist in the platform.

Disable a form with `@~`, for example:

```dogl
@~do.exec: oldHandler()
```

---

## DMN-style routing tables

A gateway may use DMN-like routing syntax either inline or by reference.

### Inline style

```dogl
<x> RouteOrder
  order.amount > 10000 => HighValueReview
  customer.tier = "gold" => AutoApprove
  =>d ManualReview
```

### Referenced style

```dogl
<x> RouteOrder @dmn: "OrderRouting"
    =>d ManualReview

dmn OrderRouting
  order.amount > 10000 => HighValueReview
  customer.tier = "gold" => AutoApprove
  =>d ManualReview
```

Architectural interpretation:

- the notation may describe DMN-style routing;
- the semantic model should keep this as explicit decision integration;
- gateway and decision semantics should not be collapsed into one generic runtime-only concept.

---

## Notes on semantics

When reading examples in this part, keep the layering clear:

- syntax-facing structures preserve the written commands and tables;
- resolver and lowering phases interpret them;
- the semantic model stays BPMN-aligned where BPMN provides the right concept;
- DOGL-only decision or behavior constructs remain explicit extensions where BPMN has no exact equivalent.

---

**Next:** [Part 4 - Organization and practices](04-organization-and-practices.md)
