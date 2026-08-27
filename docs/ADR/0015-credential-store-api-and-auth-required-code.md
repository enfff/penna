# ADR 0015: Credential Store API and AUTH_REQUIRED Code

- Status: Accepted
- Date: 2026-08-27

## Context

ADR 0010 established credential resolution (SSH agent; HTTPS via
`PENNA_GIT_TOKEN` env var, then the OS keychain under service `penna`)
inside `adapters/git`, and the adapter already gained
`store_keychain_token` / `lookup_keychain_token`. Two gaps blocked the
frontend from implementing the documented "prompt the user for a token and
persist it" flow:

1. The `PennaEngine` facade exposed no write path to the credential store.
   The only way a token could enter was the `PENNA_GIT_TOKEN` environment
   variable, which is not viable for a GUI application (and not viable
   inside a Flatpak sandbox without manual user setup).
2. `EngineError::CredentialsRequired { remote_url }` was mapped to the
   generic `REPO` code (ADR 0010 item 4), so a frontend following the
   contractual `code` (ADR 0009: `message` text is never contractual)
   could not distinguish "a token is needed for this remote" from any
   other repository failure — string-matching the message would break on
   rewording and i18n.

## Decision

1. The engine exposes per-remote credential management on `PennaEngine`:
   - `store_credential(remote_url, secret)` — persist a token in the
     platform secret store; the existing resolution path picks it up on
     the next fetch/push/clone.
   - `delete_credential(remote_url)` — remove the stored credential
     (account rotation).
   - `has_credential(remote_url)` — report whether one is stored.
   These are deliberately **not** session-scoped: a credential belongs to
   the remote, not to an open journal.
2. Blank secrets are rejected with `VALIDATION` before the secret store
   is touched (ADR 0009 item 5).
3. A sixth public code, `AUTH_REQUIRED`, is added — the first extension of
   the ADR 0009 closed set, made under its own rule (new ADR + minor
   version bump, 0.2.1 → 0.3.0). `EngineError::CredentialsRequired` maps
   to it. This supersedes ADR 0010 item 4's mapping to `REPO`.
4. `EngineErrorDto` gains an additive optional field `auth_remote:
   Option<String>` (serialized only when present), set iff the code is
   `AUTH_REQUIRED`. It carries the remote URL to authenticate to, so
   frontends never parse `message`.
5. The engine contains no prompt or callback: it reports
   `AUTH_REQUIRED` + URL; the consumer collects the secret, calls
   `store_credential`, and retries. A UI callback reaching into the
   library is the wrong layer and threading-hostile.
6. Credential resolution order (env → keychain) and its exclusive
   ownership by `adapters/git` are unchanged (ADR 0010 items 2 and 5).
   SSH remains an out-of-band ssh-agent path; no SSH-key storage API is
   added.

## Alternatives Considered

- **Keep the code as `REPO`, document the message text.** Forces
  string-matching that ADR 0009 item 4 explicitly forbids. Rejected.
- **Session-scoped in-memory secrets** (ADR 0010 item 4's original
  wording). No persistence: every app relaunch re-prompts. The env var
  remains the headless option, but the product path needs durability.
  Rejected as the primary mechanism.
- **Blocking authentication callback in the engine.** Couples the library
  to a UI thread model and is hostile to the threading contract
  (network calls run on worker threads). Rejected.

## Consequences

### Positive

- Frontends can implement prompt → store → retry using only stable,
  machine-readable contract: `AUTH_REQUIRED` code + `auth_remote` URL.
- GUI and Flatpak deployments no longer depend on environment variables.
- The additive DTO field is non-breaking for existing consumers.

### Negative

- Frontends that exhaustively matched the old five codes now see a sixth;
  per ADR 0009 they must keep a default branch for future codes.
- `delete_credential` idempotency varies by secret-store backend (some
  backends error when deleting a missing entry); consumers should check
  `has_credential` first when idempotency matters.
- ADR 0010 item 4 is partially superseded; both ADRs must be read
  together.
