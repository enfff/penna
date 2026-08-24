# ADR 0014: Marker-Based Conflict State

- Status: Accepted
- Date: 2026-08-24
- Supersedes: mechanics of [ADR 0006](./0006-explicit-per-entry-conflict-resolution.md)
  (in-memory three-way computation and programmatic reconcile commit).
  Keeps ADR 0006's UX contract, no-auto-merge rule, and tag union rule.

## Context

The first frontend reported its integration requirements for conflict
resolution. They favor standard git semantics over engine-private merge
machinery:

1. Pull reconciles via **merge**, never rebase. Conflicts leave the
   standard git conflicted state with plain `<<<<<<< / ======= / >>>>>>>`
   markers in working files. `merge.conflictStyle = merge` is pinned;
   diff3/zdiff3 base sections would break their parser.
2. Reads must return **working-tree** content during a merge so the
   frontend renders conflicted text instead of stale clean text.
3. Writes must succeed **mid-merge**: write + stage is fine; concluding
   the commit may wait.
4. A finish path must exist: after the last note resolves, sync concludes
   the merge automatically.
5. `journal_status` must expose `merge_in_progress` and
   `conflicted_paths` so detection never scans prose for marker strings.
6. Marker-less conflicts need explicit policy.

Two latent correctness bugs make this urgent regardless of frontend needs:
`save()` created commits with a single parent even when `MERGE_HEAD`
existed (stranding merges), and reads ignored working-tree edits entirely.

## Decision

1. On divergence, `sync_journal` / `pull_journal` start a real merge via
   libgit2. The repository-local config pins `merge.conflictStyle =
   merge`. The working tree receives standard conflict markers.
2. Entry reads prefer the working tree at all times; HEAD remains fallback
   for history-derived timestamps only.
3. While `MERGE_HEAD` exists, `save_entry` / `update_entry` /
   `delete_entry` stage changes and skip committing. Staging a resolved
   file clears its conflict stages, which is how resolution registers.
4. `sync_journal` auto-concludes: when a merge is in progress and no index
   conflicts remain, it creates the merge commit with parents
   `[HEAD, MERGE_HEAD]`, removes `MERGE_HEAD`, and refreshes the working
   tree. `reconcile_journal` maps to this conclude step.
5. `journal_status` grows `merge_in_progress: bool` and
   `conflicted_paths: string[]`, derived from `MERGE_HEAD` presence and
   index conflict stages — never from content scanning.
6. Merge-start applies marker-less policies immediately:
   - `.penna/*.json`: union-merged automatically, never surfaced as
     conflicts (ADR 0006 decision 7 preserved).
   - Modified/Deleted: modified side wins and resurrects the file.
   - Deleted/Deleted: dropped silently.
   Remaining content conflicts stay marked for the user.
7. `get_entry_conflict` remains available mid-merge, now served from index
   stages, for editors that want structured three-way data alongside the
   marked files.

## Consequences

### Positive

- Conflict UX matches every git user's existing mental model.
- No private reconcile machinery to maintain or explain.
- Frontends integrate with plain files and two status fields.
- Fixes real corruption (single-parent commits during merges) and
  invisible external edits as side effects.

### Negative

- Working-tree reads mean externally edited files surface uncommitted;
  frontends must handle dirty content (already required by local-first
  semantics).
- libgit2 checkout behavior during merges writes directly into the
  working tree; tests must cover the two-machine flow end to end.
