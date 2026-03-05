# Part 4 - Organization and practices

After [Part 1](01-basics.md), [Part 2](02-optional-codes-and-flows.md), and [Part 3](03-expressions-and-dmn.md), you can add more organizational surface syntax and authoring guidance.

**See also:** [Index](DSL_syntax.md) · [Cheat sheet](cheat-sheet.md)

---

## Structure in the notation

DOGL surface syntax may organize content with:

- `==` for participant-like grouping
- `--` for lanes
- `||` for stage-like grouping

Recommended reading:

- `==` is the surface form closest to a BPMN participant boundary;
- `--` is the surface form closest to a BPMN lane;
- `||` is a DOGL-specific authoring or viewpoint construct if retained, not a canonical BPMN semantic type.

---

## Important semantic distinction

These structural forms are useful in authoring, but canonical semantics should normalize into BPMN-aligned core concepts:

- `collab` lowers toward `Collaboration`
- `==` lowers toward `Participant` and associated `Process`
- `--` lowers toward `LaneSet` and `Lane`
- `||` should remain an explicit DOGL extension unless its semantics are redefined more precisely

This means the notation can stay compact without forcing the semantic model to treat every authoring convenience as a first-class BPMN core concept.

---

## Example with structure

```dogl
collab OrderProcess

== CustomerService
    -- Agent
        || Intake
            (s) StartOrder
                => ReviewApplication
            [u] ReviewApplication
                => ValidateOrder
        || Processing
            <x> ValidateOrder
                => ProcessOrder
                =>d RejectOrder

== Warehouse
    -- Picker
        || Fulfillment
            [m] PickItems
                => Done
            (e) Done
```

This example demonstrates notation structure. It should not be read as proof that stage-like constructs are part of the canonical BPMN semantic core.

---

## Comments and annotations

Use line comments with `//`:

```dogl
// This is a comment
[u] ReviewTask
```

Use annotations with `[[ ... ]]`:

```dogl
[[ Requires manager approval for high-value orders ]]
[u] ManagerApproval
```

These are syntax-facing documentation constructs and should remain available for source fidelity and tooling.

---

## Reuse and BPMN compatibility

`@call` on a `[call]` task is the closest notation form to BPMN `CallActivity`.

For reuse beyond that:

- keep BPMN compatibility where possible;
- avoid inventing canonical semantic types that duplicate BPMN-native structure unnecessarily;
- treat DOGL-specific reuse helpers as explicit extensions if they are introduced later.

---

## Good practices

1. Use PascalCase identifiers consistently.
2. Use surface syntax for readability, but keep semantic reasoning BPMN-aligned.
3. Treat participant and lane organization as structural guidance, not as a reason to blur ownership boundaries.
4. Treat stage-like or quadrant-like constructs as DOGL-specific unless they are explicitly normalized differently later.
5. Keep comments and annotations close to the syntax they document.
6. Avoid assuming that every notation feature already has full runtime behavior behind it.

---

**Back to:** [Index](DSL_syntax.md)
