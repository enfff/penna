# Markdown Round-Trip Test Prompt

Generate high-risk fixtures and tests for Markdown importer/exporter fidelity.

Goal:
export(import(M)) == M (modulo whitespace-only differences)

Include fixtures for:
- Deep nested ordered/unordered lists
- GFM tables with inline formatting
- Raw inline and block HTML
- Malformed/non-standard frontmatter
- Wikilinks
- Emoji and other multibyte Unicode
- Mixed RTL and Latin text

For each fixture provide:
1. Input Markdown
2. Expected intermediate representation
3. Expected re-exported Markdown
4. Rust test assertion

Respect architecture boundaries from docs/ARCHITECTURE.md.
