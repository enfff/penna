# Legacy Alias: adr

This legacy command is preserved.
Use GitHub Copilot prompt:
- .github/prompts/adr.prompt.md

Behavior:
- Draft next ADR number.
- Check ARCHITECTURE, DATA_MODEL, and existing ADRs for conflicts.
- If conflict exists, stop and emit warning.
- Otherwise output complete ADR and suggested filename.
