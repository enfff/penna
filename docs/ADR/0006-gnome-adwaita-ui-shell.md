# ADR 0006: GNOME Adwaita UI Design and App Shell Layout

## Context

Penna must integrate into the GNOME desktop. ADR 0005 defines Svelte+TypeScript as the frontend framework. No ADR covers visual design, component architecture, or theming. The app must follow GNOME design patterns. The app must use disabled window decorations. The app must use a custom Adwaita-style header bar. The app must follow GTK/libadwaita light-dark and accent-color settings via gsettings/D-Bus. The app must have a clear layout: header bar, sidebar for journal navigation, main editor area.

## Decision

The team uses **GNOME Adwaita** as the default theme and the only theme at this stage. The app must not offer other theme presets now. Design tokens (colors, spacing, typography) must be defined in CSS custom properties. This design allows future theme support without code changes.

The app shell has three areas:

1. **Header bar** — Custom Svelte component. Contains app title, entry actions, and settings. Follows Adwaita header style.
2. **Sidebar** — Custom Svelte component. Displays journal entries list. Supports entry selection and basic navigation.
3. **Main content** — Custom Svelte component. Placeholder for the TipTap/ProseMirror editor. No editor logic now.

Svelte components must be organized by area:

```
src/components/
├── header/
│   └── HeaderBar.svelte
├── sidebar/
│   └── Sidebar.svelte
└── content/
    └── EditorPlaceholder.svelte
```

Each component must be independently themeable and testable. The app must read GNOME theme settings (light/dark mode, accent color) at startup and on change.

## Alternatives Considered

- **Multiple themes at launch** — The team rejects this option. Multiple themes add unnecessary complexity. The team must focus on GNOME integration first. Theme support can expand later.
- **Use a UI component library (DaisyUI, Skeleton)** — The team rejects this option. These libraries do not match GNOME design. The team must build custom components for Adwaita integration.
- **No custom header bar, use Tauri default** — The team rejects this option. Tauri default does not match GNOME style. This conflicts with ADR 0001's GNOME integration goal.

## Consequences

### Positive

- The app integrates seamlessly into GNOME desktop.
- The app follows native GNOME design patterns.
- The component structure supports future growth.
- The app reads system theme settings automatically.

### Negative / Tradeoffs

- The team must build all UI components from scratch.
- The team must maintain Adwaita styling manually.
- Only one theme ships at launch. Users cannot customize appearance.

---

## GNOME Integration Guidelines

The app must follow these rules to "feel like" GNOME/Adwaita. These rules apply to all future UI work.

### 1. Typography

The app uses **Adwaita Sans** (modified Inter) as the default UI font. The app uses **Adwaita Mono** (based on Iosevka) for code contexts.

If neither font is available, the app falls back to system sans-serif or Cantarell. The app never uses generic web font stacks (Roboto, Arial).

### 2. Accent Color

The app reads the system accent color from GNOME settings. The app supports the user's choice: blue, teal, green, yellow, orange, red, pink, purple, or slate.

The app does not hardcode a single brand color. The app applies the system accent color to interactive elements (primary buttons, active states, selection highlights).

Technical implementation: Read `org.gnome.desktop.interface accent-color` via gsettings or D-Bus.

### 3. Light/Dark Mode

The app follows the OS-level light/dark preference automatically. The app uses `prefers-color-scheme` CSS media query in the WebView.

The app does not expose a separate light/dark toggle as a first-class setting.

Technical implementation: Read `org.gnome.desktop.interface color-scheme` via gsettings or D-Bus.

### 4. Window Chrome

The app uses a single merged header bar. The header bar contains window controls, title, and primary actions together.

The app has rounded top corners matching the window's outer radius. The header has no drop-shadow. The header is draggable (click-drag anywhere empty moves the window).

Technical implementation: `decorations: false` in Tauri config. Custom header bar with `-webkit-app-region: drag`.

### 5. Rounded Corners

The app uses consistent, moderate corner radii across the entire hierarchy. Window corners, cards, buttons, and input fields all share the same rounding.

The app does not mix flat and rounded elements.

### 6. Widget Compliance

Buttons, switches, and dialogs follow GNOME HIG shapes and states (hover, active, disabled).

The app does not reinvent widgets. If a custom widget is necessary, it matches Adwaita's exact proportions and states.

### 7. Adaptive Layout

The app structure supports adaptive layouts. The sidebar collapses into a stack at narrow widths. Content reflows gracefully.

Even as a desktop-only app, the layout must collapse at narrow widths. This follows GNOME adaptive design principles.
