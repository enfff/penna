# ADR 0003: git2 (libgit2) for Version Control and In-App Merge Resolution

### Context

Penna's main differentiator is its git-based design. Penna must resolve merge conflicts inside the application itself. This function is similar to the merge editor in VS Code. Penna must remain self-hostable. Penna must not depend on an external git binary. Penna must not depend on a remote service.

### Decision

The team uses **`git2-rs`**. This library provides Rust bindings to `libgit2`. The Rust core uses this library directly for all version control operations. These operations include commit, diff, branch, and 3-way merge and conflict detection. The application layer accesses these operations through a `GitRepository` port/trait.

### Alternatives Considered

- **Shelling out to the system `git` CLI** — The team rejects this option. This method depends fragilely on git being installed and present in `PATH`. This method makes output parsing harder and less reliable. This method also creates a worse cross-platform experience, especially on Windows.
- **Using a pure-Rust git reimplementation, such as `gitoxide`** — The team rejects this option for now. This library has a less mature merge and conflict API than the well-tested `libgit2`. The team notes that this option is worth revisiting later. A future ADR can supersede this ADR if `gitoxide` matures.
- **Using no git integration, only file timestamps or snapshots** — The team rejects this option. This method defeats the core "git-based" product pillar. This method also defeats the in-app conflict resolution feature.

### Consequences

**Positive:**

- `libgit2` provides mature, well-tested merge algorithms. Many git tools use this same engine.
- The team gets full programmatic access to diffs and conflicts. This access supports a custom in-app merge resolution UI.
- The application has no external process dependency. The application works identically on Linux, macOS, and Windows.
- The `GitRepository` port/trait isolates all git logic. This isolation keeps the git logic swappable. The team can test this logic fully with a mock or in-memory implementation in `core/tests`.

**Negative / Tradeoffs:**

- `libgit2` is a C library. Therefore, `git2-rs` adds an FFI and build dependency. Each platform needs a native compilation toolchain.
- Some advanced or newer git features can lag behind the official git CLI implementation.
- Building a correct, user-friendly 3-way merge UI on top of the raw conflict data requires significant engineering work.