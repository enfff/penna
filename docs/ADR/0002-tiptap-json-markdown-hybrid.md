# ADR 0002: TipTap (ProseMirror) JSON Model with Markdown as Source of Truth

### Context

Penna needs a WYSIWYG editor. Penna must store raw Markdown. This Markdown format supports clean diffs and merges in git. Penna also needs a formatting model that is richer than plain Markdown. At the same time, Penna must stay portable. Penna must import and export arbitrary Markdown files without problems.

### Decision

The team uses **TipTap/ProseMirror** as the WYSIWYG editor. The team stores entries as **plain Markdown files**. This Markdown file is the source of truth. Git diffs and merges operate on this file. The team adds an optional **JSON sidecar**. The system stores this sidecar inside the frontmatter or in a companion `.penna/` file. This sidecar captures structures that Markdown cannot express. The system always treats the JSON sidecar as disposable. The system can regenerate the sidecar, but some data may not survive regeneration. The JSON sidecar never overrides the Markdown content.

### Alternatives Considered

- **Storing all data as ProseMirror JSON only** — The team rejects this option. This method produces poor git diffs. This method is not portable or human-readable. This method breaks the "git-based" and "self-hostable/portable" goals.
- **Storing only plain Markdown with no JSON** — The team rejects this option. This method loses rich formatting and custom blocks from TipTap. This method reduces WYSIWYG fidelity.
- **Using a different editor framework, such as Slate or Lexical** — The team rejects this option. TipTap provides first-class, actively maintained Markdown import and export extensions. TipTap also provides a JSON schema. This schema maps cleanly to the hybrid model.

### Consequences

**Positive:**

- Git produces clean, human-readable diffs and merges on the Markdown files.
- The system never loses content, even when the JSON sidecar is deleted or becomes incompatible after a schema change.
- The system imports arbitrary third-party Markdown files smoothly, because Markdown is the primary format.
- The system exports clean, portable Markdown by default. The export process strips the sidecar.

**Negative / Tradeoffs:**

- The application must keep two representations of each entry in sync: the Markdown file and the JSON sidecar. This synchronization adds complexity to the application layer.
- Some Penna-specific rich formatting, such as custom widgets, has no equivalent in Markdown. The export and round-trip process must degrade this formatting gracefully.
- The team must manage schema versioning for the JSON sidecar carefully over time.