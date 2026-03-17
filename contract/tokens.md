# DOGL tokens and rules

Formal list of lexical tokens for the DOGL notation. Each token has **rules** that define how to recognize it and how it may be used in context. This is the contract for lexer implementation and for parsers that consume tokens.

**Conventions:**

- **Lexical form**: how the token appears in source (literal or pattern).
- **Rules**: (1) how to lex it (longest match, boundaries), (2) where it may appear, (3) what may follow. The parser enforces structure; the lexer only produces a stream of tokens with positions (line, column, or offset).
- **Indentation**: same rules as Python (significant whitespace; Indent/Dedent tokens; no mixing tabs and spaces). Block hierarchy is defined by indent.
- **Bounds**: optional inline after the entity identifier (element, pool, lane, stage). Form: `{ x y w h }`. If omitted, auto-placement is applied (algorithm to be defined later).

---

## 1. Keywords

### `collab`

| Rule | Description |
|------|-------------|
| Lexical | Exact ASCII string `collab`. Case-sensitive. Not part of a longer identifier (e.g. `collabX` is identifier, not keyword + `X`). |
| Context | Starts a collab block. Must be at the beginning of a logical line (after optional indentation). |
| Follows | Whitespace, then an **Identifier** (collab name). Rest of line may contain `//` comment. |

### `dmn`

| Rule | Description |
|------|-------------|
| Lexical | Exact ASCII string `dmn`. Case-sensitive. Word boundary after `dmn`. |
| Context | Starts a standalone DMN decision block. |
| Follows | Whitespace, then an **Identifier** (decision name). Then indented lines with DMN rules (condition `=>` target or `=>d` target). |

### `layout`

| Rule | Description |
|------|-------------|
| Lexical | Exact ASCII string `layout`. Case-sensitive. Not part of a longer identifier. |
| Context | Starts an optional bottom-of-file layout block. |
| Follows | Newline, then an indented hierarchy grouped by pool and reusing the same structural and element markers as the main process syntax. |

---

## 2. Structure (pool, lane, stage)

### `==`

| Rule | Description |
|------|-------------|
| Lexical | Two consecutive ASCII `=` characters. |
| Context | Introduces a **pool**. At collab or block level. |
| Follows | Whitespace, then **Identifier** (pool name). Optional: **bounds** `{ x y w h }` (diagram position). If bounds omitted, auto-placement applies. |

### `--`

| Rule | Description |
|------|-------------|
| Lexical | Two consecutive ASCII `-` characters. Not more (e.g. `---` is not this token). |
| Context | Introduces a **lane** inside a pool. |
| Follows | Whitespace, then **Identifier** (lane name). Optional: **bounds** `{ x y w h }`. If omitted, auto-placement applies. |

### `||`

| Rule | Description |
|------|-------------|
| Lexical | Two consecutive ASCII `|` characters. |
| Context | Introduces a **stage** (phase). |
| Follows | Whitespace, then **Identifier** (stage name). Optional: **bounds** `{ x y w h }`. If omitted, auto-placement applies. |

---

## 3. Element symbols

Element tokens are the shape plus optional code. **Longest match** for codes (e.g. `=>d` before `=>`, `(s)` before `(`).

### Event

| Token (examples) | Lexical | Rules |
|------------------|---------|--------|
| `()` | Literal `()` | Inferred event (start/intermediate/end from connectivity). |
| `(s)` | Literal `(s)` | Start event. |
| `(i)` | Literal `(i)` | Intermediate event. |
| `(e)` | Literal `(e)` | End event. |

| Rule | Description |
|------|-------------|
| Lexical | One of the four forms above. No space between `(` and `s`/`i`/`e`/`)`. |
| Context | Starts an element line. After optional structure (pool/lane/stage) and indentation. |
| Follows | Whitespace, then **Identifier** (element name). Optional: **bounds** `{ x y w h }` (diagram position). Optional: one bracket-command such as `[do] value` or `[do.exec] value`. If bounds omitted, auto-placement applies. Then newline; next lines may be indented flows or command lines. |

### Task

| Token (examples) | Lexical | Rules |
|------------------|---------|--------|
| `[]` | Literal `[]` | Generic task. |
| `[m]`, `[u]`, `[st]`, `[rt]`, `[se]`, `[sc]`, `[bu]`, `[sm]`, `[rm]`, `[call]` | `[` + code + `]` | Code is a known task code; see notation Part 2. |

| Rule | Description |
|------|-------------|
| Lexical | `[` followed by zero or more lowercase letters/digits forming a known code, then `]`. Longest match (e.g. `[call]` not `[c]` + `all]`). |
| Context | Same as Event: element line under collab/pool/lane/stage. |
| Follows | Same as Event: Identifier (name), optional bounds `{ x y w h }`, optional expressions, newline, then optional indented flows. |

### Gateway

| Token (examples) | Lexical | Rules |
|------------------|---------|--------|
| `<>` | Literal `<>` | Default OR (inclusive). |
| `<x>`, `<p>`, `<i>`, `<c>`, `<eb>` | `<` + code + `>` | Code: x, p, i, c, eb. |

| Rule | Description |
|------|-------------|
| Lexical | `<` followed by zero or more lowercase letters (known gateway code), then `>`. Longest match. |
| Context | Element line. |
| Follows | Identifier (name), optional bounds `{ x y w h }`, optional `[dmn] DecisionName`. Outgoing flows either listed as indented `=>` / `=>d` lines or inferred from inline DMN rules under the gateway. |

### Artifact

| Token (examples) | Lexical | Rules |
|------------------|---------|--------|
| `{}` | Literal `{}` | Default artifact. |
| `{d}`, `{db}`, `{f}`, `{r}`, `{doc}`, `{msg}`, `{e}`, `{c}` | `{` + code + `}` | Known artifact code. |

| Rule | Description |
|------|-------------|
| Lexical | `{` followed by zero or more lowercase letters forming a known code, then `}`. Longest match. |
| Context | Element line. |
| Follows | Identifier (name), optional bounds `{ x y w h }`. May have data association `.>` from or to this artifact. |

---

## 4. Flows

### `=>`

| Rule | Description |
|------|-------------|
| Lexical | Two characters `=`, `>`. If `=>d` is possible, prefer **longest match** and emit `=>d` when followed by `d` and word boundary. |
| Context | On its own line, indented under an element. Sequence flow. |
| Follows | Whitespace, then **Identifier** (target element name). Optional `//` comment to end of line. |

### `=>d`

| Rule | Description |
|------|-------------|
| Lexical | Literal `=>d`. Must be recognized before `=>` when the next character is `d` and then whitespace or end of line. |
| Context | Default flow from a gateway. Exactly one required when gateway uses DMN (inline or referenced). |
| Follows | Whitespace, then **Identifier** (default target element name). |

### `->`

| Rule | Description |
|------|-------------|
| Lexical | Two characters `-`, `>`. Not to be split; distinct from `--` (lane). |
| Context | Message flow between pools. Indented under an element. |
| Follows | Whitespace, **Identifier** (target element name, in another pool). |

### `.>`

| Rule | Description |
|------|-------------|
| Lexical | Literal `.>` (dot then angle bracket). |
| Context | Data association: artifact to/from activity. |
| Follows | Whitespace, **Identifier** (target element name). |

---

## 5. Bracket commands (on elements)

### `[command] value`

| Rule | Description |
|------|-------------|
| Lexical | `[` + command name + `]`, where command name is `call`, `dmn`, `do`, or `do.` + qualifier (for example `do.exec`, `do.timer`, `do.notify`). After the closing `]`, whitespace may follow and then the command value. |
| Context | On an element line after the element identifier, or on its own line inside an indented element block. |
| Follows | For `[call]` and `[dmn]`: identifier or quoted **String**. For `[do]` and `[do.qualifier]`: free text to end of line. |

### Rules

- `@...` forms are legacy and **not valid** notation.
- `[call]` at the start of an element line is the task marker for a call activity: `[call] ChildProcess`.
- After a generic task, event, or gateway identifier, bracket commands attach behavior or references, for example `[] ReviewOrder [do] check amount` or `<x> RouteOrder [dmn] OrderRouting`.
- If additional commands are needed, write them on indented child lines under the element.

---

## 6. Identifiers and literals

### Identifier

| Rule | Description |
|------|-------------|
| Lexical | A run of letters, digits, and underscores. Convention: **PascalCase** for element/collab/pool/lane/stage names. Must not be a keyword (`collab`, `dmn`, `layout`) when those are recognized as keywords (keyword wins). |
| Context | Element names, collab name, pool/lane/stage names, DMN decision name, target of flows, layout entity ids. |
| Follows | Depends on context: newline, another identifier (e.g. in layout), `:`, or expression. |

### String

| Rule | Description |
|------|-------------|
| Lexical | Double quote `"`, then any characters except unescaped `"`, then `"`. Escape sequence (e.g. `\"`) if defined. |
| Context | Values for bracket commands such as `[dmn] "OrderRouting"` when quoting is needed. In layout, string values if the format uses them. |
| Follows | Whitespace, newline, or next token. |

### Number

| Rule | Description |
|------|-------------|
| Lexical | Decimal number: optional minus, digits, optional fractional part (`.` + digits). Used in **inline bounds** for x, y, w, h. |
| Context | Inside optional bounds after an entity identifier: `{ x y w h }`. |
| Follows | Whitespace (next number) or `}` (end of bounds). |

---

## 7. Layout modes and inline bounds

DOGL supports **two valid layout modes**:

1. **Inline bounds** after an entity identifier.
2. **Bottom `layout` block** at the end of the file.

Both modes describe the same semantic target: bounds for pools, lanes, stages, and elements.
If bounds are omitted in either mode, auto-placement is applied by the tool.

### 7.1 Inline bounds (optional)

**Bounds** are optional. After the **Identifier** of an element, pool, lane, or stage, you may write `{ x y w h }` (four numbers: position x, y and size w, h). If bounds are **not** specified, **auto-placement** is applied; the auto-placement algorithm is defined separately (e.g. by the renderer or tool).

**Example:**

```dogl
collab MyProcess
    () Start {100 100 40 60}
        => Task
    [] Task {200 100 80 40}
        => End
    () End {300 100 40 60}
```

Pool, lane, stage may also have optional bounds: `== PoolName {0 0 400 300}`, `-- LaneName {0 50 400 60}`, `|| StageName {100 0 120 300}`.

| Rule | Description |
|------|-------------|
| Lexical | `{` followed by four **Number**s (x, y, w, h) separated by whitespace, then `}`. No commas. |
| Context | Optional, immediately after the entity **Identifier** on the same line (element, pool, lane, or stage). |
| Follows | After `}`: whitespace, then optional bracket command (on elements) or newline. |

### 7.2 Bottom `layout` block

The second valid mode is a bottom-of-file `layout` block. It reuses the same grouping and element markers as the main syntax, but carries **bounds only**.

**Rules:**

- The `layout` block appears after the main process section.
- The block is grouped by pool using `==`.
- Inside each pool, lanes and stages use the same markers `--` and `||`.
- Elements inside the block also reuse their regular markers, for example `[]`, `[call]`, `()`, `(s)`, `(e)`, `<>`, `<x>`, `<p>`.
- Entries in the block identify the same pools, lanes, stages, and elements as the main process section and attach bounds to them.
- Element ordering inside each pool should follow the order of the main process section.

**Example:**

```dogl
collab OrderProcess
    == MainPool
        -- Ops
            || Default
                (s) Start
                    => Review
                [] Review
                    => Done
                <x> Route
                    => Done
                (e) Done

layout
    == MainPool {0 0 600 320}
        -- Ops {0 40 600 80}
            || Default {120 0 180 320}
                (s) Start {80 140 36 36}
                [] Review {180 132 100 52}
                <x> Route {340 136 50 50}
                (e) Done {460 140 36 36}
```

The `layout` block is a second source form, not a second semantic model. Tools should lower either mode into the same layout representation.

### `{` and `}` (bounds)

| Rule | Description |
|------|-------------|
| Lexical | Single `{` or `}`. |
| Context | Start/end of inline bounds. Only in the form `{ x y w h }` after an identifier. |
| Follows | For `{`: whitespace, then four numbers, then `}`. For `}`: whitespace or newline. |

---

## 8. Comments and annotations

### Line comment `//`

| Rule | Description |
|------|-------------|
| Lexical | Two slashes `//`. Everything from `//` to end of line is comment. |
| Context | Anywhere on a line. |
| Parser | Lexer may drop comment content (no token) or emit a **Comment** token; parser ignores it. |

### Annotation `[[` ... `]]`

| Rule | Description |
|------|-------------|
| Lexical | `[[` starts annotation; `]]` ends it. Content between may span lines. Nested `[[` not defined; treat first `]]` as closing. |
| Context | Documentation above or beside an element. |
| Parser | Lexer may emit **Annotation** with payload or skip; parser may attach to next element or ignore. |

---

## 9. Whitespace and boundaries

Indentation uses **the same rules as Python**: significant whitespace, no mixing tabs and spaces (use one consistently). Same indent level = same block; increase indent = enter a nested block; decrease indent = exit one or more blocks. The lexer **emits Indent / Dedent tokens** (like Python) so the parser sees explicit block structure.

### Newline

| Rule | Description |
|------|-------------|
| Lexical | `\n` or `\r\n`. |
| Context | Line boundary. Indentation of next line is significant (nesting: collab → pool → lane → stage → element → flows). |
| Parser | May be explicit **Newline** token or implied; parser uses indent level (or Indent/Dedent tokens) to determine hierarchy. |

### Indentation (Python-style)

| Rule | Description |
|------|-------------|
| Lexical | Spaces **or** tabs at start of line (same as Python: one kind per file; no mixing). |
| Context | Determines nesting. Same column = same block; deeper = nested block; shallower = dedent (exit blocks). |
| Parser | **Lexer emits Indent when indent increases, Dedent when it decreases** (same semantics as Python). On dedent, emit one Dedent per level exited. Parser uses these tokens for block structure. |

### End of file

| Rule | Description |
|------|-------------|
| Lexical | No more input. |
| Parser | Lexer may emit **Eof** token; parser must handle end of token stream. |

---

## 10. Summary table (token kinds)

| Kind | Examples | Notes |
|------|----------|--------|
| Keyword | `collab`, `dmn` | Word boundary. |
| Structure | `==`, `--`, `\|\|` | Two characters. |
| Event | `()`, `(s)`, `(i)`, `(e)` | Four forms. |
| Task | `[]`, `[m]`, `[u]`, `[st]`, … | Longest match for code. |
| Gateway | `<>`, `<x>`, `<p>`, … | Longest match. |
| Artifact | `{}`, `{d}`, `{db}`, … | Longest match. |
| Flow | `=>`, `=>d`, `->`, `.>` | `=>d` before `=>`. |
| Command | `[do]`, `[dmn]`, `[call]`, `[do.exec]` | See §5. |
| Identifier | PascalCase names | Not keyword. |
| String | `"..."` | Quoted. |
| Number | Decimal, optional fraction | Inline bounds: four numbers inside `{ }`. |
| Bounds | `{ x y w h }` | Optional after entity Identifier; if omitted, auto-placement. |
| Keyword | `layout` | Optional bottom layout block. |
| Comment | `//`, `[[ ]]` | Optional token. |
| Newline / Indent / Dedent / Eof | — | Per §9; indentation is Python-style. |

This document is the **contract** for token names, lexical forms, and usage rules. Lexer and parser implementations must stay consistent with it; extensions (new codes, new keywords) should update this file.
