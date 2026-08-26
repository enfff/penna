# DEV.md — Development Guide

Practical day-to-day guide for working on Penna. For rules, see `AGENTS.md`,
`docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, and `docs/ENGINE_SCOPE.md`.

## Prerequisites

- Rust toolchain 1.88+ (workspace uses edition 2024)
- Linux builds need the dbus client library headers for the OS keychain
  (`sudo apt install libdbus-1-dev pkg-config`, `dbus-devel` on Fedora);
  macOS and Windows use their native keychains with no extra packages
- Git (no direct git usage outside `adapters/git`)
- Optional for agents: [opencode](https://opencode.ai) (`AGENTS.md` at the repo
  root is loaded automatically)

## Common Commands

```bash
cargo build                 # build all workspace crates
cargo test --workspace      # run all tests (baseline is green)
cargo clippy --workspace    # architecture/style lint (exit code must stay 0)
cargo run -p penna-engine   # engine is a library; see examples instead:
cargo run -p penna-adapters-git --example test_repo
```

Engine API examples live in `engine/examples/` and `adapters/git/examples/`.

## Verification Gate

Every change must pass before commit:

```bash
cargo test --workspace && cargo clippy --workspace
```

Known state: clippy currently reports style warnings (e.g. `&PathBuf` vs
`&Path`) in `penna-core` / `penna-adapters-fs`; exit code is still 0. Do not
introduce new failures.

## Where Code Goes

| Task | Location |
|------|----------|
| New use case | `core/application` (+ unit test in `core/tests`) |
| New port trait | `core/ports` |
| New I/O adapter | `adapters/*` implementing a port |
| Engine API surface | `engine/` |
| Frontend feature | separate repositories (consume the published `penna-engine`) |

## Releases

Releases are immutable git tags created only by the versioning script
(ADR 0002). Requires a clean tree:

```bash
git status --porcelain        # must be empty
scripts/bump-version.sh 0.1.2 # bump + commit + tag vX.Y.Z
git push origin main && git push origin v0.1.2
```

Never re-tag or force-push release tags. Full process: `docs/PUBLISHING.md`.

