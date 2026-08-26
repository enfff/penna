# Penna

A local-first, git-native journaling application.

[![License: GPL v3+](https://img.shields.io/badge/License-GPLv3%2B-blue.svg)](LICENSE)
[![CI](https://github.com/enfff/penna/actions/workflows/ci.yml/badge.svg)](https://github.com/enfff/penna/actions/workflows/ci.yml)

## What This Repository Is

This repository provides the **Penna engine**: a pure Rust library that owns
all journaling logic — entries, git-backed storage and sync, merge-based
conflict resolution, tags, attachments, and data contracts. It is a library,
not an application; the Penna apps are built in separate repositories and
talk to the engine exclusively through the `penna-engine` API.

### Workspace Layout

```
core/
  domain/       # Pure Rust entities and business rules (zero external deps)
  application/  # Use cases (entry CRUD, sync, conflicts, tags, attachments)
  ports/        # Trait definitions (EntryRepository, ConflictView, AttachmentStore, ...)
  tests/        # Unit tests for domain and application code

adapters/
  git/          # git2-rs implementation of storage, sync, and conflict ports
  fs/           # Filesystem port implementation
  markdown/     # Markdown importer/exporter implementations

engine/         # Public engine API that re-exports core and adapters
```

## Development

### Prerequisites

- Rust toolchain (1.88+, workspace uses edition 2024)

### Building the Engine

```bash
# Build the Rust core and adapters
cargo build

# Run tests
cargo test

# Check for architecture violations
cargo clippy
```

## Releases

The five published crates share one lockstep version, released as an
immutable git tag (ADR 0002):

```bash
# bump + commit + tag vX.Y.Z (requires a clean tree)
scripts/bump-version.sh 0.2.1

# publish (tag push = publish)
git push origin master && git push origin vX.Y.Z
```

Consumers pin a tag:

```toml
penna-engine = { git = "https://github.com/enfff/penna", tag = "v0.2.1" }
```

See [`docs/PUBLISHING.md`](docs/PUBLISHING.md) for the full release process.

## License

Penna is [GPL-3.0-or-later](LICENSE). Copyright © 2026 the Penna authors.

This program is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the
Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
Public License for more details.

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

The engine is stable and published (v0.2.x): entry lifecycle, git sync with
marker-based conflict resolution, attachments, tags, and provider-neutral
credentials. CI runs the test suite and clippy on every push to master.
