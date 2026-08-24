# ADR 0010: OS-Native Git Credentials

- Status: Accepted
- Date: 2026-08-24

## Context

Sync (`pull_journal`, `push_journal`, `sync_journal`, `clone_journal`) goes
through `adapters/git` on top of git2. All documented flows assume a remote,
and journals are personal, so remotes will be private repositories requiring
authentication. The git adapter currently has no credential handling at all.

Secrets policy must respect the project's nature: local-first, no server,
no accounts, and nothing sensitive written where the user did not put it.

## Decision

1. Penna never stores credentials itself and never embeds them in remote
   URLs (which would leak into `.git/config`).
2. Resolution order per remote, tried in sequence:
   - SSH remotes (`ssh://`, `git@host:path`): the user's ssh-agent, falling
     back to default key paths. No secrets touch Penna storage.
   - HTTPS remotes: the provider-neutral `PENNA_GIT_TOKEN` environment
     variable first, then the OS keychain via the `keyring` crate, scoped
     to service `penna` with the remote URL as account. Penna deliberately
     does not probe provider-specific variables (`GITHUB_TOKEN`,
     `GITLAB_TOKEN`, `AZURE_*`): any git server must work, self-hosted
     included. The token is sent as the basic-auth password with a neutral
     username.
3. On successful HTTPS authentication, the engine offers (never forces)
   saving the token to the OS keychain for future sessions.
4. When authentication is required but unavailable, sync fails with a typed
   `EngineError::CredentialsRequired { remote_url }`. Per ADR 0009 its
   public code is `REPO` (the five-code set stays closed); the structured
   payload tells the frontend to prompt. The frontend collects the secret,
   passes it to the engine for the current session, and retries. Session-held
   secrets live in memory only and die with the session (ADR 0008).
5. Credentials logic lives exclusively in `adapters/git`. Core/application
   sees only success or the typed error.

## Alternatives Considered

- **Token embedded in the remote URL.** Zero plumbing, but persists secrets
  into `.git/config` and process listings. Rejected.
- **Penna-managed encrypted store.** Reinvents the OS keychain badly and
  adds a master-password problem to a single-user app.
- **Shell out to the `git` CLI for transport.** Gets host credential
  helpers for free but drags a second git implementation into the release
  matrix and splits I/O across two stacks.

## Consequences

### Positive

- SSH users (the primary persona for private git remotes) need zero setup.
- Headless environments (CI, servers) work via env vars alone.
- No Penna-owned secret surface to audit, migrate, or lose.

### Negative

- Keychain behavior varies per platform; needs integration tests on all
  three OS targets.
- Users without ssh-agent or writable keychain must supply env vars or a
  per-session prompt every time.
