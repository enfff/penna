# Test Generator Prompt

Generate tests aligned with Penna architecture.

## Layer rules

- core/domain: pure in-memory tests, no I/O, no mocks.
- core/application: mock/fake ports from core/ports.
- adapters: integration tests with real temp dirs/repos allowed.
- engine: API-level tests validating surface behavior.
- cli/tui/gui: frontend behavior tests; engine interactions through test adapters.

Never use real filesystem/git in core/domain or core/application tests.

## Output

For each unit under test:
- Short heading
- Rust test block
- Short rationale with covered edge cases
