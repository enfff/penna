# ADR Drafting Prompt

Draft a new ADR for Penna.

## Process

1. Determine next ADR number from docs/ADR/.
2. Read docs/ARCHITECTURE.md, docs/DATA_MODEL.md, and all existing ADRs.
3. If duplicate/conflict exists, stop and output:

```
WARNING: Possible duplicate or conflict.

This decision overlaps with: docs/ADR/000N-existing-title.md
Reason: <one short sentence>
No new ADR was drafted. Review the existing ADR first.
```

4. If new, draft full ADR in this structure:

```markdown
# ADR 000N: <Short Title>

## Context
...

## Decision
...

## Alternatives Considered
- **A**: ...
- **B**: ...
- **C**: ...

## Consequences
### Positive
- ...
### Negative
- ...
```

5. Use short, direct sentences. Active voice. Use must/can.
6. Suggest filename: docs/ADR/000N-short-kebab-case-title.md

Output only the ADR content and final filename line.
