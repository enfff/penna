# Copilot Instructions For Penna

Read these documents before coding:
1. docs/ARCHITECTURE.md
2. docs/DATA_MODEL.md
3. docs/ADR/

## Core principle

The engine is frontend-agnostic.
- Engine logic lives in Rust core/application/domain/ports and adapters.
- Frontends (CLI, TUI, GUI) are replaceable clients of the engine API.

## Hard architecture rules

1. Do not import outer layers into inner layers.
- core/domain imports nothing outside std/serde.
- core/application imports only core/domain and core/ports.

2. Do not make application depend on concrete adapters.
- Define/use a port trait in core/ports first.

3. Do not let frontends call git/filesystem/adapters directly.
- Route through engine API -> application use cases.

4. Domain code must not do I/O.
- No filesystem, network, process calls.
- No git2, tauri, or frontend crates.

5. New use cases require tests.
- Each new core/application use case must have a unit test in core/tests.

6. Adapter behavior for Markdown must degrade gracefully.
- Importer/exporter must not panic on malformed input.
- Preserve frontmatter.
- Exporter strips sidecar by default.

7. If architecture changes, update docs/ARCHITECTURE.md in the same change.

## Implementation placement

- Use cases: core/application
- Port traits: core/ports
- Adapter implementations: adapters/
- Engine API: engine/
- Frontends: cli/, tui/, gui/
