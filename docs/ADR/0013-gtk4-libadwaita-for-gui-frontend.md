# ADR 0013: GTK4 With Libadwaita For The GUI Frontend

- Status: Accepted
- Date: 2026-08-24

## Context

No GUI crate exists yet (`gui/` is planned but empty). ADR 0005 fixed the
binding model (in-process Rust dependency) and deliberately left the toolkit
open. The product owner has decreed the stack in advance: prefer Libadwaita
components over native GTK whenever an equivalent exists.

## Decision

1. The GUI frontend (`gui/`) builds on GTK4 via `gtk4-rs`.
2. Libadwaita (`libadwaita` / `adw` crate) provides the application shell
   and every widget class it offers. `Adw::Application`,
   `Adw::ApplicationWindow`, `Adw::PreferencesWindow`, `Adw::MessageDialog`,
   `Adw::EntryRow`, `Adw::ToolbarView`, and friends are the default choice.
3. Raw `gtk4` widgets are used only where Libadwaita has no equivalent
   (e.g. plain `gtk::Grid` inside custom compositions), never as a styling
   or behavior substitute for an existing Adw component.
4. The app must initialize Adw styles (`Adw::init()`) and follow GNOME HIG
   patterns: adaptive layouts via `Adw::Breakpoint`, declarative actions,
   header bars through `Adw::ToolbarView`.
5. This rule applies to every future GUI contribution regardless of author;
   agents must treat "native GTK where Adw exists" as a review-blocking
   violation.

## Alternatives Considered

- **Tauri/web frontend.** Ships HTML/CSS instead of native widgets;
  rejected by the owner's explicit direction.
- **iced/egui.** Immediate-mode or Elm-style Rust UIs; neither integrates
  with the GNOME platform (no HIG conformance, weaker accessibility).
- **Raw GTK4 without Adw.** Manual CSS theming, missing adaptive helpers,
  inconsistent with the desktop the app targets.

## Consequences

### Positive

- Native GNOME look and feel for free: dark mode, accent colors,
  high-contrast support follow system settings automatically.
- Accessibility and translations come from the platform stack.
- Clear review criterion: if an Adw widget exists, using GTK is a defect.

### Negative

- Ties the GUI to Unix-like platforms (GTK works on Windows/macOS but Adw
  polish targets GNOME best).
- Contributors must know both crates well enough to pick the Adw option.

## Related Documents

- [ADR 0005](./0005-in-process-engine-binding.md) - binding model
