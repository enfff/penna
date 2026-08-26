# Publishing the Penna Engine

## The private registry: git tags

GitHub Packages does **not** host cargo crates (it supports npm, Maven,
Gradle, NuGet, RubyGems, and containers — cargo is not among them). The
canonical *private* way to distribute a Rust workspace is therefore:

> **The registry is a private GitHub repository; the "version" is a git tag.**

No hosting account, no credentials file, no registry server. Anyone with
read access to the private repo can build against any tag; everyone else
cannot even see the source.

### Consumers

```toml
# consumer's Cargo.toml
[dependencies]
penna-engine = { git = "https://github.com/<owner>/penna", tag = "v0.1.0" }
```

The workspace member directories are not the package paths — the git
dependency resolves to the repo root, cargo finds the workspace, and picks
the crate by name. All crates in the workspace share the same tag, and are
all at the same version (ADR 0002), so a single `tag =` is the entire pin.

Offline or air-gapped builds can `cargo vendor` the dependency graph as
usual; nothing about the git-dependency model prevents vendoring.

## Releasing: one script

Releases are **immutable tags**, created only by the versioning script
(ADR 0002). A release = "the tree at tag `vX.Y.Z`, where all five
workspace crates are at `X.Y.Z`".

```bash
# 1. tree must be clean (script enforces)
git status --porcelain   # expect: empty

# 2. bump (patch example); script commits + tags vX.Y.Z
scripts/bump-version.sh 0.1.1

# 3. push to GitHub (this IS the publish)
git push origin main
git push origin v0.1.1
```

SemVer for pre-1.0: `0.x.y -> 0.x.(y+1)` additive/fix,
`0.x.y -> 0.(x+1).0` breaking.

### Release checklist (every time)

- [ ] `cargo test --workspace` passes on the tagged tree
- [ ] `cargo clippy --workspace` clean (or known-clean)
- [ ] `git diff <prev-tag>..v<new> -- Cargo.lock | grep '^+name = "penna'`
      shows the version moved exactly as intended
- [ ] `git push origin main && git push origin v<new>`
- [ ] one smoke build of a consumer against the new tag

## First publish (0.1.0) — one-time setup

There is currently **no `origin` remote** (this repo is local-only).
First-time sequence:

```bash
# 1. create the PRIVATE repo on GitHub (web UI):
#    github.com -> New repository -> name: penna -> Private

# 2. wire it up
git remote add origin git@github.com:<owner>/penna.git
git push -u origin main

# 3. tag the current version and push it
git tag v0.1.0
git push origin v0.1.0

# 4. verify consumer resolution works (in a scratch project):
#    penna-engine = { git = "https://github.com/<owner>/penna", tag = "v0.1.0" }
```

The workspace carries `license = "GPL-3.0-or-later"` (inherited by every
crate from `[workspace.package]`), with the full license text in
[`LICENSE`](../LICENSE). The repository is public; the git-tag registry
model above is unchanged.

## crates.io (public option)

crates.io has **no private tier**. Publishing there is a public release:
the full sources of `penna-core`, `penna-adapters-git`, and
`penna-engine` (everything in the engine's dependency chain) become
downloadable by anyone, including git2, serde, etc. That is the price of
being discoverable in `cargo search`.

Prepared state:

- The three publishable crates carry full metadata (description, license,
  repository, homepage, keywords, categories, rust-version), inherited
  from `[workspace.package]`.
- `penna-adapters-fs` and `penna-adapters-markdown` have
  `publish = false` — they are not in the engine's dependency graph.
  Set them to publishable when they join it.
- The versions stay 1:1 with the git tag numbers (ADR 0002): v0.1.0 of
  git = 0.1.0 on crates.io.

Publishing (token from crates.io -> "Account" -> "API Tokens"):

```bash
cargo login <TOKEN>          # or: export CARGO_TOKEN=<token>

# order matters: dependencies first
cargo publish -p penna-core
cargo publish -p penna-adapters-git
cargo publish -p penna-engine
```

Pre-publish validation (catches missing metadata / wrong file inclusion
before a version is burned — crate versions can never be republished):

```bash
cargo publish --dry-run -p penna-core
cargo publish --dry-run -p penna-adapters-git
cargo publish --dry-run -p penna-engine
```

Publish only from a tagged, tested commit; tag first
(`scripts/bump-version.sh X.Y.Z`), then publish, so every consumed version
resolves the same way from both registries.

## A note on path dependencies

The in-repo path dependencies
(`penna-engine -> penna-core -> ...`) never leave the workspace:
cargo resolves them relative to the root Cargo.toml it started from, so a
git-tag consumer builds the same five crates from the same tag with no
extra configuration. External dependencies (git2, serde, chrono, ...)
still come from crates.io.
