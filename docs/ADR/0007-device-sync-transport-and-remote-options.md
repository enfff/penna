# ADR 0007: Device Sync Transport and Remote Options

## Context

Penna is local-first. Penna is multiplatform. Devices must sync journal data. ADR 0004 establishes a bare git repository as the sync rendezvous point. Devices push and pull commits through this remote. No decision covers how devices connect to this remote. No decision covers authentication. No decision covers which remote hosting options users can choose. The team must define the transport mechanism. The team must define authentication. The team must define supported remote types.

## Decision

The team uses **SSH key pairs** for transport and authentication. Each device generates its own SSH key pair. The private key never leaves the device. Only the public key is shared with the remote. Losing a device only requires revoking that device's public key. This approach does not require rotating a shared secret.

The team supports **two remote types** as equal first-class options. The user chooses one during setup. Both options use the exact same underlying mechanism: a git remote plus an SSH key.

1. **Self-hosted bare git repository** — The user hosts a bare git repository on their own infrastructure. This can be a home server, a NAS, or a small VPS. The repository is reachable over SSH. This option provides maximum privacy with zero third parties involved.

2. **Private repository on a third-party git host** — The user creates a private repository on GitHub, GitLab, Gitea, or a similar service. The user adds an SSH deploy key scoped to that single repository. A deploy key is preferred over a full account key. This option provides zero-infrastructure onboarding. The user must explicitly acknowledge that user data leaves the user's own infrastructure.

The following items remain **open questions** for future ADRs:

- **Sync trigger** — Whether sync happens manually via a button, automatically on app launch and close, or on a background interval. This requires its own future ADR.
- **Device pairing** — How a new device obtains the remote URL and registers its public key. Options include manual entry versus a pairing flow from an already configured device. This requires its own future ADR.

## Alternatives Considered

- **Password-based SSH authentication** — The team rejects this option. Passwords require user input on every device. Passwords are less secure than key-based authentication. Passwords do not support easy device revocation.
- **Shared secret or token-based authentication** — The team rejects this option. A shared secret must be stored on every device. Losing a device requires rotating the secret on all devices. This creates operational overhead. SSH keys avoid this problem.
- **HTTPS with OAuth or API tokens** — The team rejects this option. OAuth requires third-party service integration. API tokens have similar rotation problems as shared secrets. HTTPS does not provide the same device-revocation granularity as SSH keys.
- **Only self-hosted remote option** — The team rejects this option. Requiring self-hosting raises the barrier to entry. Users cannot try the app without setting up server infrastructure. Supporting third-party hosts lowers the onboarding barrier.
- **Only third-party remote option** — The team rejects this option. This would violate the self-hostable principle in ADR 0001. This would alienate privacy-focused users. Both options must be first-class.

## Consequences

### Positive

- SSH keys provide strong, industry-standard authentication.
- Device revocation is simple: remove the public key from the remote.
- Users can choose between full self-hosting or zero-infrastructure onboarding.
- The same code path handles both remote types. The implementation is identical.
- The design aligns with ADR 0004's bare git repository model.

### Negative

- SSH key management adds complexity to the user experience. Users must understand public and private keys.
- Third-party hosting means user data leaves the user's infrastructure. Privacy-conscious users must explicitly opt in.
- Deploy keys on third-party hosts are often read-only or require special configuration for write access.
- Device pairing (how to share the remote URL and public key) is not solved. This requires a future ADR.
