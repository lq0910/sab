---
name: SAB Lightweight Desktop Browser
colors:
  surface: '#0f131c'
  surface-dim: '#0f131c'
  surface-bright: '#353943'
  surface-container-lowest: '#0a0e17'
  surface-container-low: '#181b25'
  surface-container: '#1c1f29'
  surface-container-high: '#262a34'
  surface-container-highest: '#31353f'
  on-surface: '#dfe2ef'
  on-surface-variant: '#c2c6d6'
  inverse-surface: '#dfe2ef'
  inverse-on-surface: '#2c303a'
  outline: '#8c909f'
  outline-variant: '#424754'
  surface-tint: '#adc6ff'
  primary: '#adc6ff'
  on-primary: '#002e6a'
  primary-container: '#4d8eff'
  on-primary-container: '#00285d'
  inverse-primary: '#005ac2'
  secondary: '#c0c1ff'
  on-secondary: '#1000a9'
  secondary-container: '#3131c0'
  on-secondary-container: '#b0b2ff'
  tertiary: '#89ceff'
  on-tertiary: '#00344d'
  tertiary-container: '#009ada'
  on-tertiary-container: '#002d43'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#d8e2ff'
  primary-fixed-dim: '#adc6ff'
  on-primary-fixed: '#001a42'
  on-primary-fixed-variant: '#004395'
  secondary-fixed: '#e1e0ff'
  secondary-fixed-dim: '#c0c1ff'
  on-secondary-fixed: '#07006c'
  on-secondary-fixed-variant: '#2f2ebe'
  tertiary-fixed: '#c9e6ff'
  tertiary-fixed-dim: '#89ceff'
  on-tertiary-fixed: '#001e2f'
  on-tertiary-fixed-variant: '#004c6e'
  background: '#0f131c'
  on-background: '#dfe2ef'
  surface-variant: '#31353f'
typography:
  headline-xl:
    fontFamily: Geist
    fontSize: 28px
    fontWeight: '600'
    lineHeight: 36px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Geist
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: -0.015em
  headline-md:
    fontFamily: Geist
    fontSize: 18px
    fontWeight: '500'
    lineHeight: 24px
    letterSpacing: -0.01em
  headline-sm:
    fontFamily: Geist
    fontSize: 15px
    fontWeight: '500'
    lineHeight: 20px
    letterSpacing: -0.005em
  body-lg:
    fontFamily: Geist
    fontSize: 15px
    fontWeight: '400'
    lineHeight: 22px
  body-md:
    fontFamily: Geist
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  body-sm:
    fontFamily: Geist
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  label-lg:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 18px
    letterSpacing: -0.01em
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 14px
    letterSpacing: 0.02em
  label-sm:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: '500'
    lineHeight: 12px
    letterSpacing: 0.04em
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  gutter: 0.75rem
  gutter-loose: 1rem
  margin: 0.75rem
  margin-window: 0.5rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style

This design system targets power users, minimalist developers, researchers, and mindful readers seeking absolute cognitive calm. The experience centers around the core ethos of "Small And Beautiful" (SAB): high runtime velocity, deterministic desktop integration, zero promotional feed noise, and distraction-free workspace purity.

Drawing inspiration from high-precision desktop engineering (Tauri, Rust-backed utilities, and classic macOS/NeXT utility conventions), the visual identity balances technical restraint with tactile polish:
- **Calm Minimalism:** Interfaces yield precedence entirely to user content. Browser chrome remains whisper-quiet until explicitly engaged.
- **Tauri Native Resonance:** Compact geometries, snappy zero-latency micro-interactions, hardware-accelerated opacity transitions, and structured structural lines that feel woven into the host OS.
- **Instrument-grade Precision:** Pixel-accurate 1px micro-borders, purposeful monochrome hierarchies, and fine-tuned geometric typography that echo precision mechanical instruments.

## Colors

The color palette centers on a disciplined Slate/Zinc continuum engineered specifically for dark desktop environments with seamless parity in light mode.

- **Primary (`#3b82f6` - Deep Tech Blue):** Reserved strictly for active focal indicators, address bar active focus rings, download throughput pulses, and primary transactional actions. Never deployed across expansive surface fills.
- **Secondary (`#6366f1` - Deep Indigo):** Designates systemic states, developer inspect utilities, workspace tags, and privacy isolation badges.
- **Tertiary (`#0ea5e9` - Pure Cyan-Sky):** Used exclusively for telemetry indicators, secure protocol shields (TLS/HTTPS validation), and background sync status dots.
- **Neutral Core (`#090d16` - Deep Zinc Night):** Sets the ground canvas. Paired with layered neutral values:
  - Surface base: `#090d16`
  - Surface canvas/window background: `#0f141f`
  - Surface container/floating tabs: `#161d2b`
  - Subtle structural lines: `#1e293b` (Dark) and `#e2e8f0` (Light)
  - Ghost borders: `rgba(255, 255, 255, 0.07)` on dark; `rgba(15, 23, 42, 0.08)` on light.

## Typography

Typography prioritizes legibility, typographic density, and mechanical clarity across high-DPI desktop viewports.

- **Primary Typeface (`Geist`):** Delivers clean geometry with engineered aperture shapes, ensuring legibility across omnibar inputs, tab titles, settings modals, and navigation tree structures.
- **Monospace Auxiliary (`JetBrains Mono`):** Dedicated to URLs, query strings, latency monitors, memory footprint displays, keyboard shortcuts (`⌘K`, `⌃T`), and terminal-level settings.
- **System Fallbacks:** System UI stacks (`-apple-system`, `BlinkMacSystemFont`, `Segoe UI Variable`, `Segoe UI`, `Noto Sans SC`) ensure immediate zero-FOUT native boot performance in lightweight Tauri runtimes.
- **Optical Rules:** Tab labels, omnibar URLs, and status indicators strictly leverage tabular numbers (`font-variant-numeric: tabular-nums`) to prevent layout shift during progressive page loads.

## Layout & Spacing

The layout is built as a fluid application shell governed by strict structural panels:

- **Browser Window Architecture:**
  - Header / Tab Strip: Fixed 40px baseline height with inline window controls (`margin-window: 0.5rem`).
  - Omnibar Navigation Deck: 44px height spanning between global shortcuts and utility actions.
  - Sidebar Tree (collapsible): Fixed 240px width with fluid collapse to 48px icon-dock mode.
  - Webview Viewport: Strictly isolated 100% fluid container with 0px leakage into chrome gutters.
- **Spacing Rhythm:** Built on a tight 4px base increment (`0.25rem`). Desktop software demands high spatial density; internal toolbars and segmented controls rely primarily on `space-xs` (4px) and `space-sm` (8px) for compact alignment.
- **Responsive Adaptability:**
  - **Compact Desktop (<960px):** Vertical tabs collapse automatically into an overlay drawer; omnibar integrates essential navigation buttons into an inline single-row layout.
  - **Standard Desktop (960px - 1440px):** Unified top-bar or dual-pane tab layout with 12px outer canvas margins.
  - **Ultrawide (>1440px):** Content-view centered with maximum readability clamping for internal reader views; sidebar switches to persistent docked state.

## Elevation & Depth

This design system avoids heavy blurred skeuomorphic drop shadows, adopting instead a calibrated system of **Tonal Layers**, **1px Micro-borders**, and **Subtle Ambient Vignettes**:

- **Ground Level (Base Canvas / Host Window):** `rgba(9, 13, 22, 0.98)` with platform native acrylic or vibrant blur (macOS Vibrancy / Windows Mica through Tauri window flags).
- **Surface Elevation 1 (Tab Bar / Toolbars):** Background `#0f141f` framed by an absolute `1px solid rgba(255, 255, 255, 0.06)` border.
- **Surface Elevation 2 (Active Tab / Input Fields):** Background `#161d2b` with a distinct top highlight: `inset 0 1px 0 0 rgba(255, 255, 255, 0.08)`.
- **Surface Elevation 3 (Overlays, Omnibar Search HUD, Context Menus):** Background `rgba(18, 24, 38, 0.92)` paired with `backdrop-filter: blur(16px)` and a subtle ambient drop shadow: `0 12px 32px -4px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(255, 255, 255, 0.08)`.

## Shapes

The geometric silhouette reflects the "Small And Beautiful" identity: soft, humanized inner shapes encased in crisp, engineered bounds.

- **Micro Controls & Buttons (`8px - 10px`):** Window controls, toolbar icons, tag pills, and segmented chips.
- **Interactive Inputs (`12px`):** Address bar (omnibar), quick search triggers, form inputs, and inline filter inputs.
- **Floating Modals & Cards (`16px - 18px`):** Context sheets, Quick Command Palette (`⌘K`), tab overview switcher tiles, and settings panels.
- **Continuous Curvature:** All rounded geometry applies smooth continuous curves (`corner-smoothing: 60%` / iOS-style squircle where rendering permits) to prevent visual pinch points on high-density displays.

## Components

### 1. The Omnibar (Address & Action Bar)
- **Form:** 12px roundedness, 36px height, seated within the main toolbar.
- **Visuals:** Background `#121826`, 1px border `rgba(255, 255, 255, 0.07)`.
- **Focus State:** Border shifts to `#3b82f6` with a soft glow: `box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25)`.
- **Components:** Security lock / protocol chip (10px rounded, mono font, muted slate), central URL text with faded parameter string, right-aligned clean reader-mode & bookmark toggles.

### 2. Tab Elements
- **Inactive Tab:** Transparent background, muted text `#94a3b8`, seamless 10px rounded corners, 32px height. Hover brings background to `rgba(255, 255, 255, 0.04)`.
- **Active Tab:** Solid `#161d2b` fill, 1px border `rgba(255, 255, 255, 0.08)`, border-bottom transparent, white text `#f8fafc`. Subtle bottom blue accent indicator line (2px height, `#3b82f6`, centered, 16px width).

### 3. Command Palette HUD (`⌘K` Spotlight)
- **Geometry:** 600px max-width, 16px roundedness, positioned at top 15% viewport.
- **Backdrop:** Semi-transparent dark blur `rgba(4, 7, 13, 0.65)` with CSS `backdrop-filter: blur(20px)`.
- **Panel:** 1px border `rgba(255, 255, 255, 0.12)`, inner list items styled with 8px rounded rows, highlighted state in `#1e293b`, selection accompanied by monospace shortcut markers.

### 4. Buttons & Icon Actions
- **Ghost Action Icons:** 28x28px, 8px roundedness. No border, muted `#94a3b8` icon fill. On hover: background `rgba(255, 255, 255, 0.08)`, icon color shifts to `#ffffff`.
- **Primary Action (e.g., "Install Extension", "Save Session"):** Solid `#3b82f6` fill, text `#ffffff`, 10px roundedness, subtle inset highlight `inset 0 1px 0 rgba(255, 255, 255, 0.2)`.

### 5. Selection Controls (Checkboxes, Toggles, Radio)
- **Switches:** Ultra-compact 36x20px capsule, `#1e293b` track with 16px white thumb. Active state switches track to `#3b82f6`.
- **Checkboxes:** 16x16px, 4px roundedness, 1px border `#334155`. Checked state filled with `#3b82f6` featuring crisp 1.5px checkmark vector.

### 6. Minimal Reader View & Cards
- **Distraction-Free Card:** 16px roundedness, 1px border `rgba(255, 255, 255, 0.06)`, background `#0f141f`, padding `1.25rem`. No promotional widgets, badges, or third-party card clutter. Zero-noise content framing.