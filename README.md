# Penna

A local-first, git-native journaling application.

## Architecture

Penna is built with a clean separation between the **engine** and **frontends**:

- **Engine** (`core/`, `adapters/`, `engine/`): Pure Rust library implementing all journaling logic, version control, and data management. Completely frontend-agnostic.
- **Frontends** (`cli/`, `tui/`, `gui/`): Separate binaries that interact with the engine. Currently focusing on building the engine first.

### Layer Structure

```
core/
  domain/       # Pure Rust entities and business rules (zero external deps)
  application/  # Use cases (create entry, commit, resolve conflict, etc.)
  ports/        # Trait definitions (GitProvider, FileSystem, MarkdownImporter, etc.)
  tests/        # Unit tests for domain and application code

adapters/
  git/          # git2-rs implementation of GitProvider port
  fs/           # Filesystem implementation of FileSystem port
  markdown/     # Markdown importer/exporter implementations

engine/         # Public engine API that re-exports core and adapters

cli/            # Command-line interface (future)
tui/            # Terminal UI (future)
gui/            # GUI application (future)
```

## Development

### Prerequisites

- Rust toolchain (1.70+)

### Building the Engine

```bash
# Build the Rust core and adapters
cargo build

# Run tests
cargo test

# Check for architecture violations
cargo clippy
```

### Running Frontends

Currently the focus is on building the engine. Frontends will be developed separately:

```bash
# CLI (when implemented)
cargo run -p penna-cli

# TUI (when implemented)
cargo run -p penna-tui
```

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - Layer rules and dependency boundaries
- [`docs/DATA_MODEL.md`](docs/DATA_MODEL.md) - Entry structure, sidecar format, import/export contracts
- [`docs/ADR/`](docs/ADR/) - Architecture Decision Records

## Agent Instructions

GitHub Copilot should use these project files before making changes:

- `.github/agents/Penna Developer.agent.md` as the primary custom implementation agent.
- `.github/copilot-instructions.md` for repository-wide coding rules.
- `AGENTS.md` for architecture guardrails and placement rules.
- Prompt files in `.github/prompts/` for repeatable workflows.

Available prompts include:

- `adr.prompt.md` - Draft new Architecture Decision Records
- `arch-check.prompt.md` - Check for layer violations
- `test-gen.prompt.md` - Generate unit tests
- `review.prompt.md` - Code review
- `commit.prompt.md` - Generate commit messages

## Status

This is a work in progress. The engine architecture is being established. Frontends will be developed after the engine is stable.
