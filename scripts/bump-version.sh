#!/usr/bin/env bash
#
# bump-version.sh — one version for every Penna crate, in one place.
#
# All crates inherit their version (and edition + license) from
# [workspace.package] in the root Cargo.toml, so a bump is a one-line edit
# there, plus a Cargo.lock refresh, plus a git tag. This script does all
# three and commits the result.
#
# Usage:
#   scripts/bump-version.sh 0.1.1            # patch
#   scripts/bump-version.sh 0.2.0            # minor
#   scripts/bump-version.sh 1.0.0            # major
#   scripts/bump-version.sh --no-commit 0.2.0
#
# Tag created:  v0.2.0
# Consumers pin to that tag:  penna-engine = { git = "...", tag = "v0.2.0" }
#
# Requires a clean git working tree.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

DO_COMMIT=1
VERSION="${1:-}"
if [[ "$VERSION" == "--no-commit" ]]; then
    DO_COMMIT=0
    VERSION="${2:-}"
fi

# --- Validate SemVer ---------------------------------------------------------
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: expected MAJOR.MINOR.PATCH (e.g. 0.1.1), got: '${VERSION:-<empty>}'" >&2
    exit 1
fi

# --- Guard: clean tree so the tag always matches the bumped files ------------
if [[ -d .git ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "error: working tree is dirty. Commit or stash first, then re-run." >&2
        exit 1
    fi
fi

# --- Current version from the single source of truth -------------------------
# The root manifest has exactly one literal version line, under
# [workspace.package]. Everything else in the workspace inherits it.
CURRENT="$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)"
case "$CURRENT" in
    "") echo "error: no 'version = ' line found in Cargo.toml" >&2; exit 1 ;;
esac

# Refuse no-ops and downgrades.
if [[ "$CURRENT" == "$VERSION" ]]; then
    echo "error: version is already $CURRENT; nothing to bump." >&2
    exit 1
fi
IFS='.' read -r c1 c2 c3 <<< "$CURRENT"
IFS='.' read -r n1 n2 n3 <<< "$VERSION"
if (( n1 < c1 || (n1 == c1 && n2 < c2) || (n1 == c1 && n2 == c2 && n3 < c3) )); then
    echo "error: $VERSION is older than current $CURRENT (downgrade not allowed)." >&2
    exit 1
fi

# --- Write the new version into the root manifest ----------------------------
sed -i 's/^version = "[0-9][0-9.]*"$/version = "'"$VERSION"'"/' Cargo.toml

# --- Refresh Cargo.lock so the workspace packages carry the new version ------
# Workspace members' locked versions mirror their manifests, so after the
# manifest bump a regenerate is required to get the five penna-* entries
# onto the new version. Side effect to note: external dependencies are also
# re-resolved to their latest semver-compatible release — acceptable, and a
# full `cargo update` would do the same; review the regenerated diff before
# pushing if that matters.
if command -v cargo >/dev/null 2>&1; then
    cargo generate-lockfile --quiet
else
    # Fallback without cargo: patch just the penna-* [[package]] blocks.
    # Block shape in Cargo.lock (TOML v1 lockfile):
    #   [[package]]
    #   name = "penna-core"
    #   version = "0.1.0"          <- only line we may change
    #   dependencies = [ ... ]
    awk -v old="$CURRENT" -v new="$VERSION" '
        /^name = "penna/ { inp = 1 }
        /^\[\[/          { inp = 0 }
        inp && index($0, "version = \"" old "\"") == 1 {
            $0 = "version = \"" new "\""
        }
        { print }
    ' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock
fi

# --- Commit + tag -------------------------------------------------------------
# Release tags are IMMUTABLE: once vX.Y.Z is pushed, consumers may have
# pinned it, so it must never be force-moved. If the tag already exists
# locally (stale from a failed prior run), delete it first — but refuse to
# proceed if it exists remotely.
if [[ "$DO_COMMIT" -eq 1 && -d .git ]]; then
    git add Cargo.toml Cargo.lock
    git commit -q -m "chore(release): bump to $VERSION"
    if git rev-parse -q --verify "refs/tags/v$VERSION" >/dev/null 2>&1; then
        if git ls-remote --exit-code --tags origin "refs/tags/v$VERSION" >/dev/null 2>&1; then
            echo "error: v$VERSION is already pushed to origin — releases are immutable." >&2
            echo "       Pick a different version (or delete the tag on GitHub first if safe)." >&2
            exit 1
        fi
        # Tag exists only locally (stale) — safe to recreate.
        git tag -d "v$VERSION"
    fi
    git tag "v$VERSION"
    echo "bumped: $CURRENT -> $VERSION"
    echo "committed + tagged v$VERSION"
    echo ""
    echo "publish:  git push origin main && git push origin v$VERSION"
else
    echo "bumped: $CURRENT -> $VERSION (files only; --no-commit)"
fi
