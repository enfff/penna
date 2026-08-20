# ADR 0002: Lockstep Workspace Versioning

- Status: Accepted
- Date: 2026-08-14

## Context

The engine will be published to a private registry (git-tag based; see
`docs/PUBLISHING.md`). The workspace currently has five crates
(`penna-core`, `penna-adapters-git`, `penna-adapters-fs`,
`penna-adapters-markdown`, `penna-engine`), each carrying its own literal
`version = "0.1.0"`, and none of them has ever diverged in version.

The engine is a single product: `penna-engine` re-exports
`penna-core` and `penna-adapters-git`, and a consumer that builds against
two different versions of the same crate pair is only a footgun. Keeping
five independent SemVer counters is maintenance tax with no benefit.

## Decision

Adopt **lockstep versioning**: the entire workspace ships at one version.

1. The single source of truth is `[workspace.package]` in the root
   `Cargo.toml`:

   ```toml
   [workspace.package]
   version = "0.1.0"
   edition = "2024"
   license = "UNLICENSED"
   ```

2. Every member crate inherits `version`, `edition`, and `license` via
   `version.workspace = true` etc. No member manifest holds a literal
   version.

3. All version bumps go through `scripts/bump-version.sh <MAJOR.MINOR.PATCH>`
   (SemVer rules for pre-1.0 Rust releases: `0.1.0 -> 0.2.0` is breaking,
   `0.1.0 -> 0.1.1` is additive/fix). The script:
   - refuses to run on a dirty working tree,
   - refuses downgrades,
   - writes the new version to the one root manifest line,
   - refreshes `Cargo.lock`,
   - commits and creates an **immutable** tag `v<VERSION>` (refuses to
     overwrite a tag that exists on a remote).

4. Releases are the git tags. Consumers pin a tag in a `git` dependency
   (see `docs/PUBLISHING.md`). A release is valid only when its tag points
   at a tree in which all five `[[package]]` entries in `Cargo.lock` carry
   the same version.

## Consequences

- One edit, one script invocation, one tag: a release is trivially
  consistent.
- `penna-adapters-fs` and `penna-adapters-markdown` bump in lockstep even
  though they are not in the `penna-engine` dependency graph; they remain
  part of the product (importer/exporter adapters) and are not published
  separately from the rest.
- Because the registry is a private git tag (not crates.io), there is no
  registry-side version uniqueness to protect — but immutability of tags is
  still enforced by the script for the same reason: consumers pin.
- A future public crates.io publication, if it ever happens, reuses the
  same versions: tags and crates.io versions stay in 1:1 correspondence.

## Alternatives Considered

- **Independent per-crate SemVer versions.** The standard for
  multi-product workspaces; wrong here because these are not separate
  products — `penna-engine` is the only product, the rest are its guts.
- **`cargo-workspaces` (external tooling).** Does exactly what
  `scripts/bump-version.sh` does, plus CI hooks, but adds a third-party
  dependency to the release process for a one-line change. Revisit if the
  workspace grows beyond what a bash script can sanely hold.
- **No formal versioning, publish ad hoc.** The version number is the
  contract consumers pin against. Publishing without bump discipline is
  how "which version is in prod" becomes folklore.
