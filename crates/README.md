# Crates

This directory is the target home for the DOGL library-platform crates.

Current active crate:

- `dogl-language`

Reserved crate locations for later milestones:

- `dogl-render`
- `dogl-editor`
- `dogl-adapters`
- `dogl-cli`
- `dogl-wasm`

The repository still contains legacy root-level `src/` code during the transition. The workspace root now points the active language crate at that code so the repository can move toward the target multi-crate shape incrementally.
