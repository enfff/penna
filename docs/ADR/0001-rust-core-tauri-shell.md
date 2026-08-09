# ADR 0001: Rust Core with Tauri 2.0 Shell

### Context

Penna must be local-first. Penna must protect user privacy. Penna must support self-hosting. Penna must run on many platforms. These platforms include desktop systems (Linux/GNOME first, then macOS and Windows) and, later, mobile systems. Penna must switch between screens with no delay. Penna must use little disk space and little memory.

### Decision

The team uses a **Rust core** for the business logic. The Rust core performs git operations through `git2`/`libgit2` bindings. A **Tauri 2.0 shell** wraps the Rust core for the UI. The team does not use Electron.

### Alternatives Considered

- **Electron** — The team rejects this option. Electron bundles add 120–250MB to the application. Electron uses 100–300MB of RAM when idle. Electron bundles its own Chromium engine. This conflicts with the privacy, footprint, and performance goals.
- **Flutter** — The team rejects this option. Flutter needs FFI plugins for `git2`. These plugins have less mature tooling. Flutter is not a natural fit for a git-native application.
- **React Native** — The team rejects this option. React Native has similar FFI and native-module concerns as Flutter. React Native has a weaker desktop-first story.

### Consequences

**Positive:**

- The application bundle uses 5–15MB of disk space.
- The application uses about 30–40MB of RAM when idle.
- The application starts in 200–500ms.
- The application reuses the system webview. On Linux, this webview is WebKitGTK. The application does not ship a duplicate browser engine.
- The team uses one codebase for desktop and future mobile platforms, through Tauri 2.0.
- The team can unit-test the Rust core without running any UI.
- The IPC boundary separates the business logic from the presentation layer. This separation stays clean.

**Negative / Tradeoffs:**

- Tauri does not create native window chrome automatically. For GNOME integration, the team needs a GTK header bar. The team must build this header bar deliberately. The team must set `decorations: false`. The team must set `transparent: true`. The team must add a custom Adwaita-style header bar in the frontend.
- Rust has a steeper learning curve than JS-only stacks.
- WebKitGTK has rendering quirks. These quirks vary across Linux distributions. Chromium does not have these same quirks.