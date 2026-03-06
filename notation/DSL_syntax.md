# DOGL syntax guide

This guide explains the **surface syntax** of DOGL.

It is intentionally distinct from the canonical semantic model used by the platform.

Read the notation guide for authoring syntax. Interpret those forms through the BPMN-aligned semantic concepts and ownership boundaries used by the core model.

---

## Reading order

| Part | Content |
| --- | --- |
| [Part 1 - Basics](01-basics.md) | `collab`, basic shapes, simple `=>` connections, PascalCase identifiers, `@do` as placeholder |
| [Part 2 - Optional codes and flows](02-optional-codes-and-flows.md) | optional event/task/gateway codes, `=>d`, `->`, `.>` |
| [Part 3 - Expressions and DMN](03-expressions-and-dmn.md) | `@do`, `@dmn`, `@call`, DMN-like routing syntax |
| [Part 4 - Organization and practices](04-organization-and-practices.md) | participant-like grouping, lanes, stage-like authoring structure, comments, reuse guidance |

---

## At a glance

- **Start a collaboration:** `collab Name`
- **Basic node forms:** `()` `[]` `<>` `{}`
- **Basic connection:** `=>`
- **Optional connection forms:** `=>d`, `->`, `.>`
- **Optional commands:** `@do`, `@dmn`, `@call`
- **Optional structure:** `==`, `--`, `||`

---

## Important distinction

The notation guide may use compact authoring forms such as `collab`, `==`, or `||`, but canonical semantics should still be interpreted through BPMN-aligned concepts such as:

- `DoglFile`
- `Collaboration`
- `Participant`
- `Process`
- `LaneSet`
- `Lane`
- `FlowNode`
- `SequenceFlow`
- `MessageFlow`

DOGL-specific constructs such as stage-like or quadrant-like grouping should be treated as explicit extensions rather than as BPMN-native canonical types.

---

## Quick reference

Use the [cheat sheet](cheat-sheet.md) for the compressed version of the notation.
