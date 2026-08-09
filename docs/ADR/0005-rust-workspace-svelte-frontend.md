# ADR 0005: Rust Workspace Layout with Svelte+TypeScript Frontend

## Context

The team must establish the basic project structure. The team must define the Rust workspace layout. The team must choose the frontend framework. The project uses Tauri 2.0. The project uses a Rust core for business logic. The team needs a clear boundary between layers. The team needs a clear boundary between the core and the presentation layer. The team needs a frontend that integrates well with Tauri. The team needs a frontend that supports TipTap/ProseMirror.

## Decision

The team uses a **Rust workspace** with three main crates:

1. **`core`** — Contains the domain and application logic. The `core` crate has three sub-crates:
   - `core/domain` — Contains entities and business rules. This code must use pure Rust. This code must have no I/O. This code must have no external dependencies beyond `serde`.
   - `core/application` — Contains use cases. This code must depend only on `core/domain` and `core/ports`.
   - `core/ports` — Contains trait definitions. This code must contain no implementations.

2. **`adapters`** — Contains concrete implementations of ports. This directory has three sub-crates:
   - `adapters/git` — Implements the git repository port using `git2-rs`.
   - `adapters/fs` — Implements the filesystem port.
   - `adapters/markdown` — Implements the Markdown importer and exporter.

3. **`src-tauri`** — Contains the Tauri application. This crate contains:
   - Tauri command handlers.
   - The dependency injection root.
   - The application lifecycle code.

The team uses **Svelte with TypeScript** for the frontend. The frontend lives in the `src/` directory. The frontend communicates with the Rust backend only through Tauri commands. The frontend must not access the filesystem directly. The frontend must not access git directly.

The repository root contains a `Cargo.toml` workspace definition. The workspace lists `core`, `adapters`, and `src-tauri` as members.

## Alternatives Considered

- **Monolithic Rust crate** — The team rejects this option. A single crate makes layer boundaries unclear. A single crate makes testing harder. A single crate violates the hexagonal architecture pattern.
- **React or Vue for the frontend** — The team rejects these options. Svelte has a smaller bundle size. Svelte has simpler reactivity. Svelte integrates cleanly with TipTap. Svelte matches the performance goals in ADR 0001.
- **JavaScript/TypeScript-only backend** — The team rejects this option. This option defeats the Rust core decision in ADR 0001. This option loses the performance and memory benefits.
- **Separate frontend repository** — The team rejects this option. A separate repository adds unnecessary overhead. A single repository simplifies development and testing.

## Consequences

### Positive

- Clear separation of concerns between domain, application, and presentation layers.
- Each layer has a well-defined boundary. Each layer can be tested independently.
- The Svelte+TypeScript frontend integrates well with Tauri's TypeScript bindings.
- The workspace structure scales as the project grows.
- The team can unit-test the Rust core without running the UI.
- The frontend can leverage Svelte's reactivity for a responsive UI.

### Negative / Tradeoffs

- The workspace structure adds complexity for new contributors. Contributors must understand the layer boundaries.
- Building the project requires compiling multiple Rust crates. This increases build time compared to a monolithic crate.
- The team must maintain TypeScript type definitions for Tauri commands.
- Svelte has a smaller community than React or Vue. Some third-party components may be less available.
