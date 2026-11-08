---
name: Desktop Light Precision
colors:
  surface: '#f8f9ff'
  surface-dim: '#cbdbf5'
  surface-bright: '#f8f9ff'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#eff4ff'
  surface-container: '#e5eeff'
  surface-container-high: '#dce9ff'
  surface-container-highest: '#d3e4fe'
  on-surface: '#0b1c30'
  on-surface-variant: '#434655'
  inverse-surface: '#213145'
  inverse-on-surface: '#eaf1ff'
  outline: '#737686'
  outline-variant: '#c3c6d7'
  surface-tint: '#0053db'
  primary: '#004ac6'
  on-primary: '#ffffff'
  primary-container: '#2563eb'
  on-primary-container: '#eeefff'
  inverse-primary: '#b4c5ff'
  secondary: '#0058be'
  on-secondary: '#ffffff'
  secondary-container: '#2170e4'
  on-secondary-container: '#fefcff'
  tertiary: '#943700'
  on-tertiary: '#ffffff'
  tertiary-container: '#bc4800'
  on-tertiary-container: '#ffede6'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#dbe1ff'
  primary-fixed-dim: '#b4c5ff'
  on-primary-fixed: '#00174b'
  on-primary-fixed-variant: '#003ea8'
  secondary-fixed: '#d8e2ff'
  secondary-fixed-dim: '#adc6ff'
  on-secondary-fixed: '#001a42'
  on-secondary-fixed-variant: '#004395'
  tertiary-fixed: '#ffdbcd'
  tertiary-fixed-dim: '#ffb596'
  on-tertiary-fixed: '#360f00'
  on-tertiary-fixed-variant: '#7d2d00'
  background: '#f8f9ff'
  on-background: '#0b1c30'
  surface-variant: '#d3e4fe'
typography:
  headline-lg:
    fontFamily: Hanken Grotesk
    fontSize: 1.5rem
    fontWeight: '600'
    lineHeight: 2rem
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Hanken Grotesk
    fontSize: 1.25rem
    fontWeight: '600'
    lineHeight: 1.75rem
    letterSpacing: -0.015em
  headline-sm:
    fontFamily: Hanken Grotesk
    fontSize: 1.125rem
    fontWeight: '600'
    lineHeight: 1.5rem
    letterSpacing: -0.01em
  body-lg:
    fontFamily: Hanken Grotesk
    fontSize: 0.9375rem
    fontWeight: '400'
    lineHeight: 1.375rem
    letterSpacing: -0.005em
  body-md:
    fontFamily: Hanken Grotesk
    fontSize: 0.8125rem
    fontWeight: '400'
    lineHeight: 1.125rem
    letterSpacing: 0em
  body-sm:
    fontFamily: Hanken Grotesk
    fontSize: 0.75rem
    fontWeight: '400'
    lineHeight: 1rem
    letterSpacing: 0em
  label-lg:
    fontFamily: Hanken Grotesk
    fontSize: 0.8125rem
    fontWeight: '500'
    lineHeight: 1rem
    letterSpacing: 0.01em
  label-md:
    fontFamily: Hanken Grotesk
    fontSize: 0.75rem
    fontWeight: '500'
    lineHeight: 0.875rem
    letterSpacing: 0.015em
  label-code:
    fontFamily: JetBrains Mono
    fontSize: 0.75rem
    fontWeight: '400'
    lineHeight: 1rem
    letterSpacing: -0.01em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  gutter: 0.5rem
  margin: 0.75rem
  space-xs: 0.125rem
  space-sm: 0.25rem
  space-md: 0.5rem
  space-lg: 0.75rem
  space-xl: 1rem
---

## Brand & Style

This design system establishes a high-performance, native-grade desktop environment modeled after refined macOS and Windows 11 Fluent interface paradigms. The visual language focuses on structural clarity, deterministic interactions, and absolute minimization of visual noise to maximize readability and workspace efficiency.

### Target Audience & Emotional Intent
- **Audience:** Power users, developers, and knowledge professionals who demand an uncluttered, responsive desktop browsing environment.
- **Perception:** Fluid, structurally solid, surgically precise, and quiet. The UI recedes into the background, prioritizing web content and navigation velocity.

### Design Movement
- **Hybrid Native Modernism:** Combines the typographic discipline and spatial restraint of macOS with the subtle border structures, acrylic transparency hints, and micro-deliberation of Windows 11 Fluent design.
- **Density:** Compact, high-information density tailored for keyboard-first navigation, mouse precision, and dense toolbars.

## Colors

The palette relies on calibrated cool slates and pure white reflective surfaces to achieve high structural contrast without harshness.

### Palette Architecture
- **Surfaces:**
  - `Canvas / Window Chrome`: `#f1f5f9` (Slate 100) — Used for inactive tab strips, window framing, and status bars.
  - `Toolbar & Active Tab`: `#f8fafc` (Slate 50) — Elevated surface directly connecting the active tab to the unified address bar.
  - `Content Panels & Flyouts`: `#ffffff` (Pure White) — High-contrast container for web content, dropdown menus, and popovers.
- **Borders & Separators:**
  - `Hairline Border (Subtle)`: `#e2e8f0` (Slate 200) — Defines tabs, omnibox boundary, and docked panels.
  - `Structural Border (Strong)`: `#cbd5e1` (Slate 300) — Used for active focus rings, splitters, and panel resizers.
- **Typography:**
  - `Primary Text`: `#0f172a` (Slate 900) — Crisp legibility for URL input, page titles, and primary labels.
  - `Secondary Text`: `#334155` (Slate 700) — Supporting labels, keyboard shortcuts, and active state hints.
  - `Muted / Metadata`: `#64748b` (Slate 500) — Protocol prefixes (`https://`), inactive tabs, and timestamps.
- **Accents:**
  - `Primary Accent`: `#2563eb` (Blue 600) — Secure state indicators, primary actions, focused text selection, and active tab indicator pip.
  - `Interactive Hover`: `#3b82f6` (Blue 500) — Hover states for primary controls and download progress fills.

## Typography

The type scale prioritizes scanability across dense interface panels and tabular information. 

- **Primary Typeface (`Hanken Grotesk`):** Delivers clean geometry with neo-grotesque neutrality. Its narrow tracking and tall x-height allow dense tab text and omnibox URLs to remain sharp at small render sizes.
- **Monospace Auxiliary (`JetBrains Mono`):** Dedicated to technical strings, shortcut hints (e.g., `⌘T`, `Ctrl+L`), network status codes, and security protocol details.
- **Rendering & Weight Rules:** 
  - Subpixel anti-aliasing (`-webkit-font-smoothing: antialiased`) must be enforced across all light-mode window surfaces.
  - Weights are restricted to `400` (regular), `500` (medium), and `600` (semibold) to avoid heavy ink bleed on standard DPI monitors.

## Layout & Spacing

The layout model adheres to a strict, compact desktop rhythm optimized for mouse tracking and screen conservation.

### Window Geometry & Spacing Rhythm
- **Tab Strip Height:** Fixed at `36px` (`2.25rem`), providing low vertical footprint while maintaining distinct click targets.
- **Unified Navigation Bar:** Fixed at `40px` (`2.5rem`), housing navigation controls, extension icons, and the address bar.
- **Base Grid:** Built on an incremental 4px spatial baseline. Internal component gaps leverage `space-xs` (2px) and `space-sm` (4px) for tight utility groupings.
- **Desktop Window Adaptation:**
  - Full-width fluid layout anchored to system title-bar constraints.
  - Sidebars (bookmarks, tree-style tabs, developer tools) default to `260px` with fluid collapsible flex mechanics and `1px` vertical splitters.
  - Breakpoints respond to browser window viewport widths rather than device displays (`compact`: `< 768px`, `regular`: `768px - 1280px`, `wide`: `> 1280px`). Under `compact`, extension trays collapse into an overflow chevron menu.

## Elevation & Depth

Visual hierarchy is maintained through subtle tonal contrast and razor-thin borders rather than heavy drop shadows, reinforcing a light, responsive desktop application feel.

### Surface Tiers
- **Tier 0 (Window Base / Inactive Chrome):** `#f1f5f9` with zero shadow. Sits flush with system window decorations.
- **Tier 1 (Active Controls / Navigation Rail):** `#f8fafc`, separated from Tier 0 by a 1px bottom border of `#e2e8f0`.
- **Tier 2 (Floating Omnibox Results / Dropdowns):** `#ffffff`, bounded by a 1px border of `#cbd5e1`, paired with a high-diffusion, near-transparent shadow: `0 4px 16px -2px rgba(15, 23, 42, 0.06), 0 2px 4px -1px rgba(15, 23, 42, 0.04)`.
- **Tier 3 (Modal Dialogs / Critical Alerts):** `#ffffff`, bounded by `1px solid #cbd5e1` with depth: `0 12px 32px -4px rgba(15, 23, 42, 0.12)`.

### Border Discipline
- Borders are never thicker than `1px`.
- Inactive tabs use transparent borders; the active tab uses a top, left, and right border of `#e2e8f0`, merging seamlessly into the toolbar below with no bottom border.

## Shapes

The design system employs a soft, native-grade corner curvature (Level 1: `0.25rem` / `4px`) tailored for precision desktop ergonomics.

### Curvature Guidelines
- **Controls & Micro-elements:** Buttons, address bar inputs, and tab containers use `4px` (`0.25rem`) rounding.
- **Flyouts & Menus:** Dropdown context menus and settings sheets use `6px` (`rounded-lg`) to soften panel edges without encroaching on content.
- **Tab Top Edges:** Tabs feature `4px` top-left and top-right radii; bottom corners remain square (`0px`) to fuse with the main toolbar surface.

## Components

### Buttons
- **Toolbar Icon Buttons:** Height `28px`, width `28px`, border-radius `4px`. Background transparent. Hover state: `#e2e8f0` background, `#0f172a` icon fill. Active/Pressed: `#cbd5e1`.
- **Primary Action:** Solid `#2563eb` fill, `#ffffff` text, font weight `500`. Hover state `#1d4ed8`. Padding `0.375rem 0.75rem`.
- **Secondary Action:** `#ffffff` fill, `1px solid #e2e8f0`, `#334155` text. Hover state `#f8fafc` background with `#cbd5e1` border.

### Omnibox (Address & Search Input)
- **Container:** Height `32px`, full-width flex child, background `#ffffff`, border `1px solid #e2e8f0`, border-radius `4px`.
- **Focus State:** Border color `#2563eb`, paired with an outer focus ring `0 0 0 2px rgba(37, 99, 235, 0.15)`. No layout shift.
- **Text Rendering:** Host domain in `#0f172a` (`font-weight: 500`); scheme and sub-paths in `#64748b` (`font-weight: 400`).

### Tabs
- **Inactive Tab:** Background transparent, text `#64748b`. Hover state brings `#e2e8f0` at 50% opacity.
- **Active Tab:** Background `#f8fafc`, text `#0f172a` (`font-weight: 500`), borders `1px solid #e2e8f0` (top, left, right), bottom border none. Top accent pip (optional): 2px line in `#2563eb`.

### Context Menus & Dropdowns
- **Container:** Solid `#ffffff` background, `1px solid #cbd5e1`, border-radius `6px`, padding `4px`.
- **Items:** Height `26px`, padding `0 8px`, border-radius `3px`, label in `body-md`. Hover background `#f1f5f9`. Shortcut text aligned right in `label-code`, color `#64748b`.

### Checkboxes & Radios
- **Checkbox:** `14px x 14px`, `1px solid #cbd5e1`, background `#ffffff`, border-radius `3px`. Checked state: background `#2563eb`, border `#2563eb`, white checkmark glyph.
- **Radio:** `14px x 14px`, circular, checked state features a `4px` solid `#2563eb` center dot.

### Cards & Settings Containers
- **Surface:** `#ffffff` background with `1px solid #e2e8f0` perimeter border. Padding `16px`. Header separated by an internal `1px` horizontal rule `#f1f5f9`.