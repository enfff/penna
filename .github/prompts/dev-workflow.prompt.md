# Development Workflow Prompt

Drive work in this order:

1. Confirm ADR coverage.
- If no ADR covers the change, draft one first.

2. Create branch.
- Format: feat/NNNN-short-title

3. Implement in small vertical slices.
- Domain/application first, then ports, then adapters, then engine API, then frontend.

4. Run architecture check.
- Resolve all boundary violations before continuing.

5. Add tests.
- Every new application use case needs a unit test.

6. Commit.
- Stage only related files.
- Use the commit prompt for message generation.

7. Resolve merge conflicts safely if needed.
- List conflicts first.
- Resolve file by file.
- Re-run architecture check before merge commit.

8. Merge via squash back to main.
- One ADR/feature -> one main commit.
