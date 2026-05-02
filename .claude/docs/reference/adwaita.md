# GNOME / Adwaita Design System — Web Implementation Reference

> Target stack: React functional components + Tailwind CSS (extended config) + CSS Custom Properties.  
> Source of truth: GNOME Human Interface Guidelines, GTK4 source, libadwaita source (v1.x).  
> All color values verified against `_adwaita.scss` and `adwaita-dark.css` from the libadwaita repository.

---

## Section 1 — Design Tokens

### 1.1 Color Palette (Adwaita)

#### Window / Surface

| Token Name (CSS var) | Light Mode (hex) | Dark Mode (hex) | Semantic Use                       |
|----------------------|------------------|-----------------|------------------------------------|
| `--window-bg-color`  | `#fafafa`        | `#242424`       | Main application window background |
| `--view-bg-color`    | `#ffffff`        | `#1e1e1e`       | Content areas, lists, text views   |
| `--card-bg-color`    | `#ffffff`        | `#2d2d2d`       | Card surfaces, preference rows     |
| `--popover-bg-color` | `#ffffff`        | `#2d2d2d`       | Popovers, context menus            |
| `--dialog-bg-color`  | `#fafafa`        | `#2d2d2d`       | Dialog window backgrounds          |

#### Headerbar

| Token Name (CSS var)       | Light Mode (hex)   | Dark Mode (hex)    | Semantic Use                         |
|----------------------------|--------------------|--------------------|--------------------------------------|
| `--headerbar-bg-color`     | `#ebebeb`          | `#303030`          | Headerbar default background         |
| `--headerbar-fg-color`     | `#2d2d2d`          | `#ffffff`          | Text and icon color inside headerbar |
| `--headerbar-border-color` | `#d0d0d0`          | `#1a1a1a`          | Bottom border of headerbar           |
| `--headerbar-shade-color`  | `rgba(0,0,0,0.07)` | `rgba(0,0,0,0.36)` | Gradient shadow below headerbar      |

#### Sidebar

| Token Name (CSS var) | Light Mode (hex) | Dark Mode (hex) | Semantic Use                              |
|----------------------|------------------|-----------------|-------------------------------------------|
| `--sidebar-bg-color` | `#ebebeb`        | `#2d2d2d`       | Sidebar background (split-view left pane) |
| `--sidebar-fg-color` | `#2d2d2d`        | `#ffffff`       | Text and icon color inside sidebar        |

#### Text / Foreground

| Token Name (CSS var) | Light Mode (hex) | Dark Mode (hex) | Semantic Use                          |
|----------------------|------------------|-----------------|---------------------------------------|
| `--window-fg-color`  | `#2d2d2d`        | `#ffffff`       | Primary text on window background     |
| `--view-fg-color`    | `#2d2d2d`        | `#ffffff`       | Primary text on view/list backgrounds |
| `--card-fg-color`    | `#2d2d2d`        | `#ffffff`       | Primary text on card surfaces         |

#### Accent

| Token Name (CSS var) | Light Mode (hex) | Dark Mode (hex) | Semantic Use                                             |
|----------------------|------------------|-----------------|----------------------------------------------------------|
| `--accent-bg-color`  | `#3584e4`        | `#3584e4`       | Filled accent button background, selected row bg         |
| `--accent-fg-color`  | `#ffffff`        | `#ffffff`       | Text/icons on top of accent background                   |
| `--accent-color`     | `#2373c7`        | `#78aeed`       | Accent-colored text, links, focus rings (meets contrast) |

#### Destructive

| Token Name (CSS var)     | Light Mode (hex) | Dark Mode (hex) | Semantic Use                              |
|--------------------------|------------------|-----------------|-------------------------------------------|
| `--destructive-bg-color` | `#e01b24`        | `#c01c28`       | Filled destructive button background      |
| `--destructive-fg-color` | `#ffffff`        | `#ffffff`       | Text on destructive background            |
| `--destructive-color`    | `#c01c28`        | `#ff7b7f`       | Destructive-colored text, inline warnings |

#### Success / Warning / Error

| Token Name (CSS var) | Light Mode (hex) | Dark Mode (hex) | Semantic Use                                        |
|----------------------|------------------|-----------------|-----------------------------------------------------|
| `--success-bg-color` | `#26a269`        | `#2ec27e`       | Success badge/banner background                     |
| `--success-fg-color` | `#ffffff`        | `#ffffff`       | Text on success background                          |
| `--success-color`    | `#1c8c57`        | `#57e389`       | Inline success-colored text                         |
| `--warning-bg-color` | `#e5a50a`        | `#cd9309`       | Warning badge/banner background                     |
| `--warning-fg-color` | `#ffffff`        | `#ffffff`       | Text on warning background                          |
| `--warning-color`    | `#9c6e03`        | `#f8e45c`       | Inline warning-colored text                         |
| `--error-bg-color`   | `#e01b24`        | `#c01c28`       | Error badge/banner background (same as destructive) |
| `--error-fg-color`   | `#ffffff`        | `#ffffff`       | Text on error background                            |
| `--error-color`      | `#c01c28`        | `#ff7b7f`       | Inline error text                                   |

#### Borders and Separators

| Token Name (CSS var)        | Light Mode (hex)        | Dark Mode (hex)          | Semantic Use                  |
|-----------------------------|-------------------------|--------------------------|-------------------------------|
| `--border-color`            | `rgba(0,0,0,0.12)`      | `rgba(255,255,255,0.12)` | General-purpose border        |
| `--shade-color`             | `rgba(0,0,0,0.07)`      | `rgba(0,0,0,0.36)`       | Inset shadows, pressed states |
| `--scrollbar-outline-color` | `rgba(255,255,255,0.5)` | `rgba(255,255,255,0.1)`  | Scrollbar thumb outline       |

```css
/* ============================================================
   GNOME / Adwaita — CSS Custom Properties
   ============================================================ */

:root {
  /* Window / Surface */
  --window-bg-color:     #fafafa;
  --view-bg-color:       #ffffff;
  --card-bg-color:       #ffffff;
  --popover-bg-color:    #ffffff;
  --dialog-bg-color:     #fafafa;

  /* Headerbar */
  --headerbar-bg-color:      #ebebeb;
  --headerbar-fg-color:      #2d2d2d;
  --headerbar-border-color:  #d0d0d0;
  --headerbar-shade-color:   rgba(0, 0, 0, 0.07);

  /* Sidebar */
  --sidebar-bg-color:  #ebebeb;
  --sidebar-fg-color:  #2d2d2d;

  /* Text / Foreground */
  --window-fg-color:  #2d2d2d;
  --view-fg-color:    #2d2d2d;
  --card-fg-color:    #2d2d2d;

  /* Accent */
  --accent-bg-color:  #3584e4;
  --accent-fg-color:  #ffffff;
  --accent-color:     #2373c7;

  /* Destructive */
  --destructive-bg-color:  #e01b24;
  --destructive-fg-color:  #ffffff;
  --destructive-color:     #c01c28;

  /* Success */
  --success-bg-color:  #26a269;
  --success-fg-color:  #ffffff;
  --success-color:     #1c8c57;

  /* Warning */
  --warning-bg-color:  #e5a50a;
  --warning-fg-color:  #ffffff;
  --warning-color:     #9c6e03;

  /* Error */
  --error-bg-color:  #e01b24;
  --error-fg-color:  #ffffff;
  --error-color:     #c01c28;

  /* Borders */
  --border-color:              rgba(0, 0, 0, 0.12);
  --shade-color:               rgba(0, 0, 0, 0.07);
  --scrollbar-outline-color:   rgba(255, 255, 255, 0.5);

  /* Dim / Disabled */
  --dim-label-alpha: 0.55;
}

[data-theme="dark"] {
  /* Window / Surface */
  --window-bg-color:     #242424;
  --view-bg-color:       #1e1e1e;
  --card-bg-color:       #2d2d2d;
  --popover-bg-color:    #2d2d2d;
  --dialog-bg-color:     #2d2d2d;

  /* Headerbar */
  --headerbar-bg-color:      #303030;
  --headerbar-fg-color:      #ffffff;
  --headerbar-border-color:  #1a1a1a;
  --headerbar-shade-color:   rgba(0, 0, 0, 0.36);

  /* Sidebar */
  --sidebar-bg-color:  #2d2d2d;
  --sidebar-fg-color:  #ffffff;

  /* Text / Foreground */
  --window-fg-color:  #ffffff;
  --view-fg-color:    #ffffff;
  --card-fg-color:    #ffffff;

  /* Accent */
  --accent-bg-color:  #3584e4;
  --accent-fg-color:  #ffffff;
  --accent-color:     #78aeed;

  /* Destructive */
  --destructive-bg-color:  #c01c28;
  --destructive-fg-color:  #ffffff;
  --destructive-color:     #ff7b7f;

  /* Success */
  --success-bg-color:  #2ec27e;
  --success-fg-color:  #ffffff;
  --success-color:     #57e389;

  /* Warning */
  --warning-bg-color:  #cd9309;
  --warning-fg-color:  #ffffff;
  --warning-color:     #f8e45c;

  /* Error */
  --error-bg-color:  #c01c28;
  --error-fg-color:  #ffffff;
  --error-color:     #ff7b7f;

  /* Borders */
  --border-color:              rgba(255, 255, 255, 0.12);
  --shade-color:               rgba(0, 0, 0, 0.36);
  --scrollbar-outline-color:   rgba(255, 255, 255, 0.1);

  --dim-label-alpha: 0.55;
}
```

---

### 1.2 Typography

**GNOME 47+ (current):** Primary font is **Inter** — a variable font chosen for its high legibility across display
densities and fine-grained weight/contrast control. **Cantarell** is the legacy font used in GNOME 46 and earlier;
retain it only as a fallback for older environments. For monospaced contexts (terminals, code editors, GNOME Text
Editor), use **Source Code Pro**.

```css
/* Google Fonts import — Inter + Source Code Pro */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&family=Source+Code+Pro:wght@400;500;700&display=swap');

/* Global rendering — matches GNOME's light antialiasing + hinting */
body {
  font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

code, pre, kbd, samp, .monospace {
  font-family: "Source Code Pro", "Cascadia Code", "Fira Code", ui-monospace, monospace;
}
```

**Weight usage:**

- `400` Regular — body text, descriptions, row subtitles
- `500` Medium — `AdwHeaderBar` titles, button labels, row titles
- `700` Bold — section headings, dialog titles, display text

> **Legacy note:** If targeting GNOME 46 or earlier, swap `"Inter"` for `"Cantarell"` in the font stack. Cantarell only
> ships weights `400` and `700` — `500` Medium is not available.

| Scale             | GNOME Element                          | `font-size` (rem) | `font-weight` | `line-height` | `letter-spacing` |
|-------------------|----------------------------------------|-------------------|---------------|---------------|------------------|
| `display`         | Large hero text, welcome screens       | `2.5rem` (40px)   | `700`         | `1.2`         | `-0.02em`        |
| `title-1`         | Page/dialog primary title              | `2rem` (32px)     | `700`         | `1.25`        | `-0.015em`       |
| `title-2`         | Section header, large labels           | `1.5rem` (24px)   | `700`         | `1.3`         | `-0.01em`        |
| `title-3`         | Card titles, pane headers              | `1.25rem` (20px)  | `700`         | `1.35`        | `-0.005em`       |
| `title-4`         | Sub-section titles                     | `1.125rem` (18px) | `700`         | `1.4`         | `0`              |
| `heading`         | Group headings, sidebar section labels | `0.875rem` (14px) | `700`         | `1.4`         | `0.04em`         |
| `body`            | Default paragraph and UI text          | `1rem` (16px)     | `400`         | `1.5`         | `0`              |
| `caption`         | Helper text, timestamps, metadata      | `0.75rem` (12px)  | `400`         | `1.4`         | `0`              |
| `caption-heading` | Uppercase label overlines              | `0.75rem` (12px)  | `700`         | `1.4`         | `0.06em`         |

```css
/* Typography utility classes */
.text-gnome-display        { font-size: 2.5rem;    font-weight: 700; line-height: 1.2;  letter-spacing: -0.02em; }
.text-gnome-title-1        { font-size: 2rem;      font-weight: 700; line-height: 1.25; letter-spacing: -0.015em; }
.text-gnome-title-2        { font-size: 1.5rem;    font-weight: 700; line-height: 1.3;  letter-spacing: -0.01em; }
.text-gnome-title-3        { font-size: 1.25rem;   font-weight: 700; line-height: 1.35; letter-spacing: -0.005em; }
.text-gnome-title-4        { font-size: 1.125rem;  font-weight: 700; line-height: 1.4;  letter-spacing: 0; }
.text-gnome-heading        { font-size: 0.875rem;  font-weight: 700; line-height: 1.4;  letter-spacing: 0.04em; }
.text-gnome-body           { font-size: 1rem;      font-weight: 400; line-height: 1.5;  letter-spacing: 0; }
.text-gnome-caption        { font-size: 0.75rem;   font-weight: 400; line-height: 1.4;  letter-spacing: 0; }
.text-gnome-caption-heading{ font-size: 0.75rem;   font-weight: 700; line-height: 1.4;  letter-spacing: 0.06em; text-transform: uppercase; }
```

---

### 1.3 Spacing (8px Grid)

GNOME spacing is built on a 4px base unit with structural preference for 8px multiples.

| Token      | Value (px) | Value (rem) | Typical Use                                         |
|------------|------------|-------------|-----------------------------------------------------|
| `space-1`  | 4px        | 0.25rem     | Icon-to-label gap, tight row padding                |
| `space-2`  | 8px        | 0.5rem      | Button internal padding (vertical), list item gap   |
| `space-3`  | 12px       | 0.75rem     | Card/row horizontal padding (narrow), input padding |
| `space-4`  | 16px       | 1rem        | Standard component padding, section gap             |
| `space-5`  | 20px       | 1.25rem     | Larger section internal padding                     |
| `space-6`  | 24px       | 1.5rem      | Dialog padding, group gap                           |
| `space-7`  | 28px       | 1.75rem     | Status page element spacing                         |
| `space-8`  | 32px       | 2rem        | Between major sections                              |
| `space-9`  | 36px       | 2.25rem     | Large structural gaps                               |
| `space-10` | 40px       | 2.5rem      | Welcome/status icon margin                          |
| `space-11` | 44px       | 2.75rem     | Minimum touch target height                         |
| `space-12` | 48px       | 3rem        | Headerbar height, large structural padding          |

```js
// tailwind.config.js — extend.spacing
spacing: {
  'gnome-1':  '0.25rem',  //  4px
  'gnome-2':  '0.5rem',   //  8px
  'gnome-3':  '0.75rem',  // 12px
  'gnome-4':  '1rem',     // 16px
  'gnome-5':  '1.25rem',  // 20px
  'gnome-6':  '1.5rem',   // 24px
  'gnome-7':  '1.75rem',  // 28px
  'gnome-8':  '2rem',     // 32px
  'gnome-9':  '2.25rem',  // 36px
  'gnome-10': '2.5rem',   // 40px
  'gnome-11': '2.75rem',  // 44px
  'gnome-12': '3rem',     // 48px
},
```

---

### 1.4 Border Radius

| Token                  | Value    | Usage Context                               |
|------------------------|----------|---------------------------------------------|
| `--border-radius-xs`   | `4px`    | Badges, small chips, progress bars          |
| `--border-radius-sm`   | `6px`    | Buttons, text inputs, switches, spinners    |
| `--border-radius-md`   | `12px`   | Cards, dialogs, popovers, banners           |
| `--border-radius-lg`   | `16px`   | Main application window corners             |
| `--border-radius-pill` | `9999px` | Pill badges, circular avatars, pill buttons |

```css
:root {
  --border-radius-xs:   4px;
  --border-radius-sm:   6px;
  --border-radius-md:   12px;
  --border-radius-lg:   16px;
  --border-radius-pill: 9999px;
}
```

---

### 1.5 Elevation and Shadows

GNOME shadows are subtle separation markers, not deep-space elevation cues. Dark mode shadows rely on surface color
contrast rather than shadow intensity.

| Level        | `box-shadow` (light)                                      | `box-shadow` (dark)                                      | Context                             |
|--------------|-----------------------------------------------------------|----------------------------------------------------------|-------------------------------------|
| `elevated-0` | `none`                                                    | `none`                                                   | Base surface, no visual lift needed |
| `elevated-1` | `0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.08)`  | `0 1px 3px rgba(0,0,0,0.4), 0 1px 2px rgba(0,0,0,0.3)`   | Cards, list boxes, rows with hover  |
| `elevated-2` | `0 4px 12px rgba(0,0,0,0.15), 0 2px 4px rgba(0,0,0,0.08)` | `0 4px 12px rgba(0,0,0,0.5), 0 2px 4px rgba(0,0,0,0.35)` | Dialogs, sheets, dropdown menus     |

```css
:root {
  --shadow-elevated-0: none;
  --shadow-elevated-1:
    0 1px 3px rgba(0, 0, 0, 0.12),
    0 1px 2px rgba(0, 0, 0, 0.08);
  --shadow-elevated-2:
    0 4px 12px rgba(0, 0, 0, 0.15),
    0 2px  4px rgba(0, 0, 0, 0.08);
}

[data-theme="dark"] {
  --shadow-elevated-1:
    0 1px 3px rgba(0, 0, 0, 0.4),
    0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-elevated-2:
    0 4px 12px rgba(0, 0, 0, 0.5),
    0 2px  4px rgba(0, 0, 0, 0.35);
}
```

---

### 1.6 Animations and Transitions

GNOME transitions are intentionally fast. They confirm state changes without drawing attention away from content.

| Property                         | Value                                  | Use                                        |
|----------------------------------|----------------------------------------|--------------------------------------------|
| `--transition-duration-fast`     | `100ms`                                | Hover color changes, focus ring appearance |
| `--transition-duration-normal`   | `200ms`                                | Component expansion, popover open/close    |
| `--transition-duration-slow`     | `400ms`                                | Page/view transitions, full-sheet slides   |
| `--transition-easing-default`    | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | Standard UI movement (ease-out-like)       |
| `--transition-easing-spring`     | `cubic-bezier(0.34, 1.56, 0.64, 1)`    | Modal/popover entrance (slight overshoot)  |
| `--transition-easing-decelerate` | `cubic-bezier(0.0, 0.0, 0.2, 1)`       | Elements entering the screen               |
| `--transition-easing-accelerate` | `cubic-bezier(0.4, 0.0, 1, 1)`         | Elements leaving the screen                |

```css
:root {
  --transition-duration-fast:    100ms;
  --transition-duration-normal:  200ms;
  --transition-duration-slow:    400ms;

  --transition-easing-default:     cubic-bezier(0.25, 0.46, 0.45, 0.94);
  --transition-easing-spring:      cubic-bezier(0.34, 1.56, 0.64, 1);
  --transition-easing-decelerate:  cubic-bezier(0.0,  0.0,  0.2,  1);
  --transition-easing-accelerate:  cubic-bezier(0.4,  0.0,  1,    1);
}
```

---

## Section 2 — libadwaita Components

---

#### AdwHeaderBar

**GTK4 equivalent**: `AdwHeaderBar` (extends `GtkWidget`, typically inside `AdwToolbarView`)

**Anatomy**:

- `start` slot — left-aligned actions (back button, hamburger menu)
- `title` slot — center-aligned `AdwWindowTitle` (title + optional subtitle), or `AdwViewSwitcher`
- `end` slot — right-aligned actions (primary action button, overflow menu)
- Bottom border line — `--headerbar-border-color`
- Bottom gradient — `--headerbar-shade-color`

**Behavioral States**:

- `default` — standard background
- `raised` (when content scrolls under it) — subtle shadow intensifies
- `flat` — no bottom border (used inside dialogs or preference pages)

**CSS (web implementation)**:

```css
.adw-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 48px;
  padding: 0 6px;
  background-color: var(--headerbar-bg-color);
  color: var(--headerbar-fg-color);
  border-bottom: 1px solid var(--headerbar-border-color);
  box-shadow: 0 1px 0 var(--headerbar-shade-color);
  position: relative;
  z-index: 10;
  transition:
    background-color var(--transition-duration-normal) var(--transition-easing-default),
    box-shadow var(--transition-duration-normal) var(--transition-easing-default);
}

.adw-header-bar--raised {
  box-shadow:
    0 1px 0 var(--headerbar-border-color),
    0 2px 8px rgba(0, 0, 0, 0.12);
}

.adw-header-bar--flat {
  border-bottom: none;
  box-shadow: none;
  background-color: transparent;
}

.adw-header-bar__start,
.adw-header-bar__end {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
}

.adw-header-bar__end {
  justify-content: flex-end;
}

.adw-header-bar__title {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* AdwWindowTitle */
.adw-window-title {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1.2;
}

.adw-window-title__title {
  font-size: 0.9375rem; /* 15px */
  font-weight: 700;
  color: var(--headerbar-fg-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
}

.adw-window-title__subtitle {
  font-size: 0.75rem;
  font-weight: 400;
  color: var(--headerbar-fg-color);
  opacity: var(--dim-label-alpha);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
}

/* Header bar buttons */
.adw-header-bar .adw-button {
  min-width: 36px;
  min-height: 36px;
  padding: 6px;
  border-radius: var(--border-radius-sm);
  background: transparent;
  border: none;
  color: var(--headerbar-fg-color);
  cursor: pointer;
  transition:
    background-color var(--transition-duration-fast) var(--transition-easing-default);
}

.adw-header-bar .adw-button:hover {
  background-color: rgba(128, 128, 128, 0.15);
}

.adw-header-bar .adw-button:active {
  background-color: rgba(128, 128, 128, 0.25);
}

.adw-header-bar .adw-button:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: 1px;
}
```

**React (minimum JSX structure)**:

```jsx
function AdwHeaderBar({
  start,
  end,
  title,
  subtitle,
  raised = false,
  flat = false,
  titleWidget,
}) {
  const classes = [
    'adw-header-bar',
    raised && 'adw-header-bar--raised',
    flat  && 'adw-header-bar--flat',
  ].filter(Boolean).join(' ');

  return (
    <header className={classes} role="banner">
      <div className="adw-header-bar__start">
        {start}
      </div>
      <div className="adw-header-bar__title">
        {titleWidget ?? (
          <AdwWindowTitle title={title} subtitle={subtitle} />
        )}
      </div>
      <div className="adw-header-bar__end">
        {end}
      </div>
    </header>
  );
}

function AdwWindowTitle({ title, subtitle }) {
  return (
    <div className="adw-window-title" aria-label={subtitle ? `${title}, ${subtitle}` : title}>
      <span className="adw-window-title__title">{title}</span>
      {subtitle && (
        <span className="adw-window-title__subtitle" aria-hidden="true">
          {subtitle}
        </span>
      )}
    </div>
  );
}
```

**Accessibility**: `role="banner"` on the outer `<header>`. Back button must have `aria-label="Back"`. Overflow menu
button: `aria-label="Main menu"`, `aria-haspopup="menu"`.

**Fidelity notes**: Native GTK uses CSS nodes `headerbar`, `windowtitle`, `.title`, `.subtitle`. The `raised` state in
GTK is driven automatically by scroll position — on web, wire it to a scroll listener on the content area. The centering
of the title in GTK uses absolute positioning relative to the bar, not flexbox — for strict centering when start/end
slots have different widths, use CSS Grid with `grid-template-columns: 1fr auto 1fr` instead of flexbox.

---

#### AdwViewSwitcher

**GTK4 equivalent**: `AdwViewSwitcher` (used inside `AdwHeaderBar` or as a bottom navigation bar)

**Anatomy**:

- Container bar — horizontal flex row of tab buttons
- Each tab — icon (24px symbolic) + label text, stacked vertically or side-by-side depending on available width
- Active indicator — underline or background fill on selected tab
- Badge overlay — optional numeric badge on icon

**Behavioral States**:

- `default` — inactive tab, dimmed
- `active` — selected tab, full opacity, accent indicator
- `hover` — subtle background tint
- `narrow` — icon-only mode when bar width < 360px (label hidden, tooltip shown)

**CSS (web implementation)**:

```css
.adw-view-switcher {
  display: flex;
  align-items: stretch;
  background-color: var(--headerbar-bg-color);
  height: 48px;
}

.adw-view-switcher--bottom {
  background-color: var(--window-bg-color);
  border-top: 1px solid var(--border-color);
  height: 56px;
}

.adw-view-switcher__button {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 2px;
  padding: 4px 8px;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--headerbar-fg-color);
  opacity: 0.55;
  position: relative;
  border-radius: 0;
  min-width: 56px;
  transition:
    opacity var(--transition-duration-fast) var(--transition-easing-default),
    background-color var(--transition-duration-fast) var(--transition-easing-default);
}

.adw-view-switcher__button:hover {
  opacity: 0.8;
  background-color: rgba(128, 128, 128, 0.1);
}

.adw-view-switcher__button[aria-selected="true"] {
  opacity: 1;
}

/* Active indicator line */
.adw-view-switcher__button[aria-selected="true"]::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 20%;
  right: 20%;
  height: 2px;
  background-color: var(--accent-bg-color);
  border-radius: 2px 2px 0 0;
}

.adw-view-switcher__button:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: -2px;
}

.adw-view-switcher__icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.adw-view-switcher__label {
  font-size: 0.75rem;
  font-weight: 500;
  white-space: nowrap;
}

/* Narrow: hide labels */
.adw-view-switcher--narrow .adw-view-switcher__label {
  display: none;
}
```

**React (minimum JSX structure)**:

```jsx
function AdwViewSwitcher({ pages, activePage, onPageChange, position = 'top', narrow = false }) {
  const classes = [
    'adw-view-switcher',
    position === 'bottom' && 'adw-view-switcher--bottom',
    narrow && 'adw-view-switcher--narrow',
  ].filter(Boolean).join(' ');

  return (
    <nav className={classes} role="tablist" aria-label="Main navigation">
      {pages.map((page) => (
        <button
          key={page.id}
          className="adw-view-switcher__button"
          role="tab"
          aria-selected={activePage === page.id}
          aria-controls={`panel-${page.id}`}
          onClick={() => onPageChange(page.id)}
          title={narrow ? page.label : undefined}
        >
          <svg className="adw-view-switcher__icon" viewBox="0 0 24 24" aria-hidden="true">
            {page.icon}
          </svg>
          <span className="adw-view-switcher__label">{page.label}</span>
        </button>
      ))}
    </nav>
  );
}

/* Usage */
const pages = [
  { id: 'home',     label: 'Home',     icon: <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/> },
  { id: 'library',  label: 'Library',  icon: <path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z"/> },
  { id: 'settings', label: 'Settings', icon: <path d="M19.43 12.98c.04-.32.07-.64.07-.98s-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65C14.46 2.18 14.25 2 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.23-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98s.03.66.07.98l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.23.09.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zM12 15.5c-1.93 0-3.5-1.57-3.5-3.5s1.57-3.5 3.5-3.5 3.5 1.57 3.5 3.5-1.57 3.5-3.5 3.5z"/> },
];
```

**Accessibility**: `role="tablist"` on container. Each button `role="tab"`, `aria-selected`, `aria-controls` pointing to
its panel. Panels use `role="tabpanel"` with `aria-labelledby`.

**Fidelity notes**: In narrow mode, the GTK widget automatically transitions to bottom bar placement — on web, manage
this via `ResizeObserver` or a breakpoint CSS class. Badge support requires absolute-positioned `<span>` over the icon.

---

#### AdwNavigationView

**GTK4 equivalent**: `AdwNavigationView` + `AdwNavigationPage`

**Anatomy**:

- View container — clips children, full height/width
- Navigation stack — array of page records `{id, title, component}`
- Slide transition — new pages slide in from the right; back slides out to the right
- Header integration — each page owns its `AdwHeaderBar` with a back button when depth > 1

**Behavioral States**:

- `idle` — stable, one page visible
- `pushing` — new page animating in from right
- `popping` — top page animating out to right

**CSS (web implementation)**:

```css
.adw-navigation-view {
  position: relative;
  overflow: hidden;
  width: 100%;
  height: 100%;
}

.adw-navigation-page {
  position: absolute;
  inset: 0;
  background-color: var(--window-bg-color);
  will-change: transform;
  display: flex;
  flex-direction: column;
}

/* Transition classes — apply via JS */
.adw-navigation-page--enter {
  transform: translateX(100%);
}

.adw-navigation-page--enter-active {
  transform: translateX(0);
  transition: transform var(--transition-duration-slow) var(--transition-easing-decelerate);
}

.adw-navigation-page--exit {
  transform: translateX(0);
}

.adw-navigation-page--exit-active {
  transform: translateX(100%);
  transition: transform var(--transition-duration-slow) var(--transition-easing-accelerate);
}

/* Previous page dims while new page pushes in */
.adw-navigation-page--behind {
  transform: translateX(-30%);
  opacity: 0.7;
  transition:
    transform var(--transition-duration-slow) var(--transition-easing-default),
    opacity   var(--transition-duration-slow) var(--transition-easing-default);
  pointer-events: none;
}
```

**React (minimum JSX structure)**:

```jsx
import { useState, useCallback } from 'react';

function AdwNavigationView({ initialPage }) {
  const [stack, setStack] = useState([initialPage]);
  const [transitioning, setTransitioning] = useState(false);

  const push = useCallback((page) => {
    if (transitioning) return;
    setStack(prev => [...prev, page]);
  }, [transitioning]);

  const pop = useCallback(() => {
    if (transitioning || stack.length <= 1) return;
    setStack(prev => prev.slice(0, -1));
  }, [transitioning, stack.length]);

  const currentPage = stack[stack.length - 1];
  const canGoBack = stack.length > 1;

  return (
    <div className="adw-navigation-view">
      <div className="adw-navigation-page adw-navigation-page--enter-active">
        <AdwHeaderBar
          title={currentPage.title}
          start={canGoBack && (
            <button
              className="adw-button"
              onClick={pop}
              aria-label="Back"
            >
              {/* go-previous-symbolic */}
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
          )}
        />
        <div className="adw-navigation-page__content" style={{ flex: 1, overflowY: 'auto' }}>
          <currentPage.component push={push} pop={pop} />
        </div>
      </div>
    </div>
  );
}

/* Usage */
const rootPage = {
  id: 'root',
  title: 'Settings',
  component: ({ push }) => (
    <button onClick={() => push({ id: 'wifi', title: 'Wi-Fi', component: WiFiPage })}>
      Wi-Fi
    </button>
  ),
};
```

**Accessibility**: Each page should be `role="region"` with `aria-label` matching the page title. When pushing, move
focus to the new page's first interactive element. When popping, return focus to the triggering element.

**Fidelity notes**: Production use should integrate `react-transition-group` or a similar library for robust enter/exit
class management. The "behind" page dim is a libadwaita visual detail — acceptable to omit in constrained environments.
Deep-link navigation (back to specific stack depth) is not part of the base GTK widget but is common in web
implementations.

---

#### AdwActionRow

**GTK4 equivalent**: `AdwActionRow` (extends `AdwPreferencesRow`)

**Anatomy**:

- Row container — full-width, min 44px height
- `prefix` slot — optional icon or widget at left (24px symbolic icon in a 32px container)
- `title` — primary label, medium weight
- `subtitle` — secondary label below title, dimmed
- `suffix` slot — right-aligned widget (Switch, Arrow, Badge, Button)
- Separator line — bottom border between rows

**Behavioral States**:

- `default` — resting state
- `hover` — subtle background tint (only when `activatable`)
- `active` / `pressed` — darker tint
- `focus` — accent outline
- `disabled` — full row dimmed to 50% opacity, pointer-events none

**CSS (web implementation)**:

```css
.adw-action-row {
  display: flex;
  align-items: center;
  min-height: 56px;
  padding: 8px 12px;
  gap: 12px;
  background-color: var(--card-bg-color);
  color: var(--card-fg-color);
  border-bottom: 1px solid var(--border-color);
  position: relative;
  transition: background-color var(--transition-duration-fast) var(--transition-easing-default);
}

.adw-action-row:last-child {
  border-bottom: none;
}

.adw-action-row--activatable {
  cursor: pointer;
}

.adw-action-row--activatable:hover {
  background-color: rgba(128, 128, 128, 0.07);
}

.adw-action-row--activatable:active {
  background-color: rgba(128, 128, 128, 0.14);
}

.adw-action-row--activatable:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: -2px;
}

.adw-action-row--disabled {
  opacity: 0.5;
  pointer-events: none;
}

.adw-action-row__prefix {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  color: var(--card-fg-color);
  opacity: 0.75;
}

.adw-action-row__prefix svg {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.adw-action-row__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.adw-action-row__title {
  font-size: 1rem;
  font-weight: 400;
  color: var(--card-fg-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.adw-action-row__subtitle {
  font-size: 0.8125rem;
  font-weight: 400;
  color: var(--card-fg-color);
  opacity: var(--dim-label-alpha);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.adw-action-row__suffix {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

/* Chevron for navigable rows */
.adw-action-row__chevron {
  width: 16px;
  height: 16px;
  fill: currentColor;
  opacity: 0.4;
}

/* ---- AdwExpanderRow ---- */
.adw-expander-row__children {
  display: none;
  flex-direction: column;
  background-color: rgba(0, 0, 0, 0.03);
  border-top: 1px solid var(--border-color);
}

[data-theme="dark"] .adw-expander-row__children {
  background-color: rgba(255, 255, 255, 0.03);
}

.adw-expander-row--expanded .adw-expander-row__children {
  display: flex;
}

.adw-expander-row__arrow {
  width: 16px;
  height: 16px;
  fill: currentColor;
  opacity: 0.6;
  transition: transform var(--transition-duration-normal) var(--transition-easing-default);
}

.adw-expander-row--expanded .adw-expander-row__arrow {
  transform: rotate(90deg);
}
```

**React (minimum JSX structure)**:

```jsx
function AdwActionRow({
  title,
  subtitle,
  prefix,
  suffix,
  activatable = false,
  disabled = false,
  onClick,
  children, // for expander variant
}) {
  const classes = [
    'adw-action-row',
    activatable && 'adw-action-row--activatable',
    disabled    && 'adw-action-row--disabled',
  ].filter(Boolean).join(' ');

  const Tag = activatable ? 'button' : 'div';

  return (
    <Tag
      className={classes}
      onClick={activatable ? onClick : undefined}
      disabled={disabled}
      aria-disabled={disabled}
    >
      {prefix && (
        <div className="adw-action-row__prefix" aria-hidden="true">
          {prefix}
        </div>
      )}
      <div className="adw-action-row__body">
        <span className="adw-action-row__title">{title}</span>
        {subtitle && (
          <span className="adw-action-row__subtitle">{subtitle}</span>
        )}
      </div>
      {suffix && (
        <div className="adw-action-row__suffix">{suffix}</div>
      )}
    </Tag>
  );
}

/* AdwExpanderRow variant */
function AdwExpanderRow({ title, subtitle, prefix, children }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className={`adw-action-row adw-expander-row ${expanded ? 'adw-expander-row--expanded' : ''}`}
    >
      <button
        className="adw-action-row adw-action-row--activatable"
        style={{ flex: 1, background: 'none', border: 'none', padding: 0, cursor: 'pointer', display: 'contents' }}
        onClick={() => setExpanded(e => !e)}
        aria-expanded={expanded}
      >
        {prefix && <div className="adw-action-row__prefix">{prefix}</div>}
        <div className="adw-action-row__body">
          <span className="adw-action-row__title">{title}</span>
          {subtitle && <span className="adw-action-row__subtitle">{subtitle}</span>}
        </div>
        <div className="adw-action-row__suffix">
          <svg className="adw-expander-row__arrow" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
          </svg>
        </div>
      </button>
      <div className="adw-expander-row__children" role="region">
        {children}
      </div>
    </div>
  );
}
```

**Accessibility**: Use `<button>` when `activatable`. Title is the accessible name. Suffix Switch: `aria-label` must
include the row title (e.g., `aria-label="Enable notifications"`). `AdwExpanderRow`: `aria-expanded` on trigger,
`role="region"` on children container.

**Fidelity notes**: GTK natively handles the `activatable-widget` property which forwards row clicks to a child widget (
e.g., a Switch). On web, stop propagation on suffix widget click events to avoid double-firing the row's onClick.

---

#### AdwPreferencesGroup

**GTK4 equivalent**: `AdwPreferencesGroup`

**Anatomy**:

- Group container — vertical flex column
- `header` — optional title (bold, small caps style) + optional description text
- `listbox` — bordered box containing `AdwActionRow` children, with rounded corners
- Spacing — 24px gap between groups

**Behavioral States**: Stateless container. Row children carry their own states.

**CSS (web implementation)**:

```css
.adw-preferences-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0 12px;
}

.adw-preferences-group + .adw-preferences-group {
  margin-top: 24px;
}

.adw-preferences-group__header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 6px;
}

.adw-preferences-group__title {
  font-size: 0.875rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--window-fg-color);
  opacity: 0.55;
  text-transform: uppercase;
}

.adw-preferences-group__description {
  font-size: 0.875rem;
  font-weight: 400;
  color: var(--window-fg-color);
  opacity: var(--dim-label-alpha);
}

.adw-preferences-group__listbox {
  display: flex;
  flex-direction: column;
  background-color: var(--card-bg-color);
  border-radius: var(--border-radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-elevated-1);
  border: 1px solid var(--border-color);
}
```

**React (minimum JSX structure)**:

```jsx
function AdwPreferencesGroup({ title, description, children }) {
  return (
    <section className="adw-preferences-group" aria-labelledby={title ? `group-${title}` : undefined}>
      {(title || description) && (
        <div className="adw-preferences-group__header">
          {title && (
            <h3
              id={`group-${title}`}
              className="adw-preferences-group__title"
            >
              {title}
            </h3>
          )}
          {description && (
            <p className="adw-preferences-group__description">{description}</p>
          )}
        </div>
      )}
      <div className="adw-preferences-group__listbox" role="list">
        {children}
      </div>
    </section>
  );
}

/* Usage */
<AdwPreferencesGroup title="Notifications" description="Control how the app notifies you.">
  <AdwActionRow
    title="Allow notifications"
    activatable
    suffix={<AdwSwitch checked={notifEnabled} onChange={setNotifEnabled} />}
  />
  <AdwActionRow
    title="Sound alerts"
    activatable
    suffix={<AdwSwitch checked={soundEnabled} onChange={setSoundEnabled} />}
  />
</AdwPreferencesGroup>
```

**Accessibility**: `<section>` with `aria-labelledby` pointing to the group title. The listbox uses `role="list"`; each
`AdwActionRow` should use `role="listitem"` when not activatable.

**Fidelity notes**: GTK renders the title with `title-case`, not uppercase — the uppercase style here is a common web
adaptation for visual hierarchy. The original GTK uses small-caps-like treatment via Pango; the CSS
`text-transform: uppercase` + `letter-spacing` approximates it faithfully on web.

---

#### AdwStatusPage

**GTK4 equivalent**: `AdwStatusPage`

**Anatomy**:

- Full-height centered container
- `icon` — large symbolic SVG, 64–96px, color uses `--accent-color` or neutral dimmed
- `title` — bold, large, centered
- `description` — body text, centered, max 400px width, dimmed
- `child` — optional widget below description (typically a button or small action group)

**Behavioral States**: Stateless display component.

**CSS (web implementation)**:

```css
.adw-status-page {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  min-height: 300px;
  gap: 0;
}

.adw-status-page__icon {
  width: 80px;
  height: 80px;
  fill: currentColor;
  color: var(--window-fg-color);
  opacity: 0.35;
  margin-bottom: 24px;
}

.adw-status-page__icon--accent {
  color: var(--accent-color);
  opacity: 1;
}

.adw-status-page__icon--error {
  color: var(--error-color);
  opacity: 1;
}

.adw-status-page__title {
  font-size: 1.5rem;
  font-weight: 800;
  color: var(--window-fg-color);
  margin-bottom: 8px;
  max-width: 400px;
}

.adw-status-page__description {
  font-size: 1rem;
  color: var(--window-fg-color);
  opacity: var(--dim-label-alpha);
  max-width: 360px;
  line-height: 1.5;
  text-wrap: pretty;
  margin-bottom: 24px;
}

.adw-status-page__child {
  margin-top: 4px;
}
```

**React (minimum JSX structure)**:

```jsx
function AdwStatusPage({ icon, title, description, iconVariant = 'default', children }) {
  const iconClass = [
    'adw-status-page__icon',
    iconVariant === 'accent' && 'adw-status-page__icon--accent',
    iconVariant === 'error'  && 'adw-status-page__icon--error',
  ].filter(Boolean).join(' ');

  return (
    <div className="adw-status-page" role="status" aria-live="polite">
      {icon && (
        <div className={iconClass} aria-hidden="true">
          {icon}
        </div>
      )}
      <h2 className="adw-status-page__title">{title}</h2>
      {description && (
        <p className="adw-status-page__description">{description}</p>
      )}
      {children && (
        <div className="adw-status-page__child">{children}</div>
      )}
    </div>
  );
}

/* Usage — empty state */
<AdwStatusPage
  icon={<svg viewBox="0 0 24 24"><path d="M20 6h-2.18c.07-.44.18-.88.18-1.35C18 2.53 15.48 0 12.35 0c-1.7 0-3.23.72-4.35 1.86C6.98.72 5.45 0 3.65 0 .52 0-2 2.53-2 5.65c0 .47.07.91.18 1.35H-2v14h22V6zm-8.35-4c1.57 0 2.85 1.28 2.85 2.85S13.22 7.7 11.65 7.7c-1.57 0-2.85-1.28-2.85-2.85S10.08 2 11.65 2z"/></svg>}
  title="No Items Found"
  description="There are no items to display. Try adjusting your search or adding new content."
  iconVariant="default"
>
  <button className="adw-button adw-button--suggested">Add Item</button>
</AdwStatusPage>
```

**Accessibility**: `role="status"` with `aria-live="polite"` for dynamic empty states. Icon is `aria-hidden`. Title uses
`<h2>` (assuming it appears inside a page with an `<h1>` headerbar title).

**Fidelity notes**: GTK uses a 128px icon for welcome screens and 64px for in-page empty states. The web implementation
uses 80px as a balanced default — adjust per context with the `iconVariant` and size props.

---

#### AdwToast

**GTK4 equivalent**: `AdwToast` + `AdwToastOverlay`

**Anatomy**:

- Overlay container — fixed, bottom-center, full-width z-index layer
- Toast element — pill-shaped or rounded-rect bar, dark background
- `title` — toast message text
- `button` — optional action button (accent color text, no background)
- Stack — up to 3 toasts visible; new ones push upward, oldest auto-dismiss

**Behavioral States**:

- `entering` — slides up from bottom
- `visible` — steady display
- `exiting` — fades/slides down on timeout or dismiss
- `dismissed` — removed from DOM

**CSS (web implementation)**:

```css
.adw-toast-overlay {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column-reverse;
  align-items: center;
  gap: 8px;
  z-index: 9000;
  pointer-events: none;
}

.adw-toast {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 8px 10px 16px;
  background-color: #363636;
  color: #ffffff;
  border-radius: var(--border-radius-pill);
  box-shadow: var(--shadow-elevated-2);
  pointer-events: all;
  max-width: min(480px, 90vw);
  animation: toast-in var(--transition-duration-normal) var(--transition-easing-decelerate) forwards;
}

[data-theme="dark"] .adw-toast {
  background-color: #e0e0e0;
  color: #2d2d2d;
}

.adw-toast--exiting {
  animation: toast-out var(--transition-duration-normal) var(--transition-easing-accelerate) forwards;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes toast-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(16px) scale(0.96);
  }
}

.adw-toast__title {
  font-size: 0.9375rem;
  font-weight: 400;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.adw-toast__action {
  background: transparent;
  border: none;
  color: #78aeed; /* accent in dark context */
  font-size: 0.9375rem;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: var(--border-radius-sm);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background-color var(--transition-duration-fast) var(--transition-easing-default);
}

[data-theme="dark"] .adw-toast__action {
  color: #2373c7;
}

.adw-toast__action:hover {
  background-color: rgba(255, 255, 255, 0.12);
}

[data-theme="dark"] .adw-toast__action:hover {
  background-color: rgba(0, 0, 0, 0.08);
}

.adw-toast__action:focus-visible {
  outline: 2px solid var(--accent-color);
}
```

**React (minimum JSX structure)**:

```jsx
import { useState, useCallback, useEffect, useRef } from 'react';

const TOAST_DURATION = 4000; // ms before auto-dismiss

function useToast() {
  const [toasts, setToasts] = useState([]);
  const idRef = useRef(0);

  const addToast = useCallback(({ title, actionLabel, onAction }) => {
    const id = ++idRef.current;
    setToasts(prev => [...prev.slice(-2), { id, title, actionLabel, onAction, exiting: false }]);
    return id;
  }, []);

  const removeToast = useCallback((id) => {
    setToasts(prev => prev.map(t => t.id === id ? { ...t, exiting: true } : t));
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 200);
  }, []);

  return { toasts, addToast, removeToast };
}

function AdwToastOverlay({ toasts, removeToast }) {
  return (
    <div
      className="adw-toast-overlay"
      role="region"
      aria-label="Notifications"
      aria-live="polite"
    >
      {toasts.map(toast => (
        <AdwToast key={toast.id} toast={toast} onRemove={removeToast} />
      ))}
    </div>
  );
}

function AdwToast({ toast, onRemove }) {
  useEffect(() => {
    const timer = setTimeout(() => onRemove(toast.id), TOAST_DURATION);
    return () => clearTimeout(timer);
  }, [toast.id, onRemove]);

  return (
    <div
      className={`adw-toast ${toast.exiting ? 'adw-toast--exiting' : ''}`}
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
    >
      <span className="adw-toast__title">{toast.title}</span>
      {toast.actionLabel && (
        <button
          className="adw-toast__action"
          onClick={() => { toast.onAction?.(); onRemove(toast.id); }}
        >
          {toast.actionLabel}
        </button>
      )}
    </div>
  );
}
```

**Accessibility**: Each toast uses `role="alert"` with `aria-live="assertive"`. The overlay region uses
`aria-live="polite"`. Action button must be keyboard-reachable; toasts should not auto-dismiss while keyboard focus is
inside them.

**Fidelity notes**: GTK toasts are always pill-shaped and center-docked at the bottom. The dark-on-light / light-on-dark
inversion of the toast background (always near-black in light mode, near-white in dark mode) creates a separation from
the page regardless of theme — do not use the page's surface tokens for toast backgrounds.

---

#### AdwDialog / AdwAlertDialog

**GTK4 equivalent**: `AdwDialog` (v1.5+), `AdwAlertDialog`

**Anatomy**:

- Scrim — full-viewport dark overlay
- Dialog container — centered, max-width 480px (alert) or 560px (dialog), rounded `--border-radius-lg`
- Header — title (center-aligned for alerts, left for dialogs) + optional close button
- Body — scrollable content area
- Footer (response area) — horizontal row of buttons, right-aligned for dialogs; vertically stacked for alerts on narrow
  screens

**Behavioral States**:

- `opening` — scale-in + fade animation
- `visible` — stable
- `closing` — scale-out + fade

**CSS (web implementation)**:

```css
.adw-dialog-scrim {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  z-index: 8000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  animation: scrim-in var(--transition-duration-normal) var(--transition-easing-default) forwards;
}

.adw-dialog-scrim--closing {
  animation: scrim-out var(--transition-duration-normal) var(--transition-easing-default) forwards;
}

@keyframes scrim-in  { from { opacity: 0; } to { opacity: 1; } }
@keyframes scrim-out { from { opacity: 1; } to { opacity: 0; } }

.adw-dialog {
  background-color: var(--dialog-bg-color);
  color: var(--window-fg-color);
  border-radius: var(--border-radius-lg);
  box-shadow: var(--shadow-elevated-2);
  width: 100%;
  max-width: 560px;
  max-height: calc(100vh - 48px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: dialog-in var(--transition-duration-normal) var(--transition-easing-spring) forwards;
}

.adw-dialog--alert {
  max-width: 480px;
  text-align: center;
}

.adw-dialog--closing {
  animation: dialog-out var(--transition-duration-normal) var(--transition-easing-accelerate) forwards;
}

@keyframes dialog-in {
  from { opacity: 0; transform: scale(0.96); }
  to   { opacity: 1; transform: scale(1); }
}

@keyframes dialog-out {
  from { opacity: 1; transform: scale(1); }
  to   { opacity: 0; transform: scale(0.96); }
}

.adw-dialog__header {
  display: flex;
  align-items: center;
  padding: 18px 24px 12px;
  min-height: 56px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.adw-dialog--alert .adw-dialog__header {
  justify-content: center;
  border-bottom: none;
  padding-bottom: 4px;
}

.adw-dialog__title {
  font-size: 1rem;
  font-weight: 700;
  flex: 1;
  color: var(--window-fg-color);
}

.adw-dialog--alert .adw-dialog__title {
  font-size: 1.125rem;
  text-align: center;
}

.adw-dialog__close {
  background: transparent;
  border: none;
  padding: 6px;
  border-radius: var(--border-radius-sm);
  cursor: pointer;
  color: var(--window-fg-color);
  opacity: 0.6;
  transition: opacity var(--transition-duration-fast) var(--transition-easing-default);
}

.adw-dialog__close:hover { opacity: 1; }

.adw-dialog__body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
  color: var(--window-fg-color);
  font-size: 1rem;
  line-height: 1.5;
}

.adw-dialog--alert .adw-dialog__body {
  padding: 8px 24px 16px;
  text-align: center;
  color: var(--window-fg-color);
  opacity: var(--dim-label-alpha);
}

.adw-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px 16px;
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
  flex-wrap: wrap;
}

.adw-dialog--alert .adw-dialog__footer {
  justify-content: stretch;
  flex-direction: column-reverse;
  border-top: none;
  padding-top: 8px;
}

.adw-dialog--alert .adw-dialog__footer .adw-button {
  width: 100%;
  justify-content: center;
}
```

**React (minimum JSX structure)**:

```jsx
import { useEffect, useRef } from 'react';

function AdwDialog({
  open,
  onClose,
  title,
  children,
  footer,
  alert = false,
  closeButton = true,
}) {
  const dialogRef = useRef(null);

  // Trap focus and handle Escape
  useEffect(() => {
    if (!open) return;
    const el = dialogRef.current;
    el?.focus();
    const onKeyDown = (e) => {
      if (e.key === 'Escape') onClose?.();
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  const dialogClass = [
    'adw-dialog',
    alert && 'adw-dialog--alert',
  ].filter(Boolean).join(' ');

  return (
    <div
      className="adw-dialog-scrim"
      onClick={(e) => { if (e.target === e.currentTarget) onClose?.(); }}
      role="presentation"
    >
      <div
        ref={dialogRef}
        className={dialogClass}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        tabIndex={-1}
      >
        <div className="adw-dialog__header">
          <h2 id="dialog-title" className="adw-dialog__title">{title}</h2>
          {closeButton && !alert && (
            <button
              className="adw-dialog__close"
              onClick={onClose}
              aria-label="Close dialog"
            >
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          )}
        </div>
        <div className="adw-dialog__body">{children}</div>
        {footer && <div className="adw-dialog__footer">{footer}</div>}
      </div>
    </div>
  );
}

/* AdwAlertDialog usage */
<AdwDialog
  open={showDeleteAlert}
  onClose={() => setShowDeleteAlert(false)}
  title="Delete File?"
  alert
  closeButton={false}
  footer={<>
    <button className="adw-button" onClick={() => setShowDeleteAlert(false)}>Cancel</button>
    <button className="adw-button adw-button--destructive" onClick={handleDelete}>Delete</button>
  </>}
>
  This file will be permanently deleted and cannot be recovered.
</AdwDialog>
```

**Accessibility**: `role="dialog"`, `aria-modal="true"`, `aria-labelledby` matching title `id`. Focus must be trapped
within the dialog while open. On close, focus returns to the element that triggered it. Destructive actions must be the
last in DOM order and not the default-focused element.

**Fidelity notes**: `AdwAlertDialog` in libadwaita v1.5+ replaced `AdwMessageDialog`. The footer button order is
reversed in GTK (cancel on left, destructive on right) — the web `column-reverse` flexbox achieves the same visual order
while keeping the destructive action last in DOM (better for accessibility).
`[to be verified in the official documentation]` — the exact spring curve for dialog entrance animation may vary between
libadwaita versions.

---

#### Buttons

**GTK4 equivalent**: `GtkButton` with CSS classes `.suggested-action`, `.destructive-action`, `.flat`, `.pill`,
`.opaque`

**Anatomy**:

- Container — bordered/filled box with rounded corners
- Icon — optional leading 16px symbolic icon
- Label — button text, medium weight
- Variants: `default`, `suggested`, `destructive`, `flat`, `pill`

**Behavioral States** (all variants):

- `default` — resting
- `hover` — lighter/darker tint overlay
- `active` / `pressed` — deeper tint, slight downward shift
- `focus` — accent outline
- `disabled` — 50% opacity, pointer-events none

**CSS (web implementation)**:

```css
/* ---- Base button ---- */
.adw-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  padding: 0 16px;
  font-size: 0.9375rem;
  font-weight: 500;
  font-family: inherit;
  line-height: 1;
  border-radius: var(--border-radius-sm);
  border: 1px solid var(--border-color);
  background-color: var(--card-bg-color);
  color: var(--window-fg-color);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  text-decoration: none;
  transition:
    background-color var(--transition-duration-fast) var(--transition-easing-default),
    box-shadow       var(--transition-duration-fast) var(--transition-easing-default),
    transform        var(--transition-duration-fast) var(--transition-easing-default);
  box-shadow: var(--shadow-elevated-1);
}

.adw-button:hover {
  background-color: color-mix(in srgb, var(--card-bg-color) 85%, var(--window-fg-color) 15%);
}

.adw-button:active {
  background-color: color-mix(in srgb, var(--card-bg-color) 75%, var(--window-fg-color) 25%);
  box-shadow: none;
  transform: translateY(1px);
}

.adw-button:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: 2px;
}

.adw-button:disabled {
  opacity: 0.5;
  pointer-events: none;
}

/* ---- Suggested action (primary) ---- */
.adw-button--suggested {
  background-color: var(--accent-bg-color);
  border-color: transparent;
  color: var(--accent-fg-color);
  box-shadow: none;
}

.adw-button--suggested:hover {
  background-color: color-mix(in srgb, var(--accent-bg-color) 88%, #000 12%);
}

.adw-button--suggested:active {
  background-color: color-mix(in srgb, var(--accent-bg-color) 75%, #000 25%);
}

/* ---- Destructive action ---- */
.adw-button--destructive {
  background-color: var(--destructive-bg-color);
  border-color: transparent;
  color: var(--destructive-fg-color);
  box-shadow: none;
}

.adw-button--destructive:hover {
  background-color: color-mix(in srgb, var(--destructive-bg-color) 88%, #000 12%);
}

.adw-button--destructive:active {
  background-color: color-mix(in srgb, var(--destructive-bg-color) 75%, #000 25%);
}

/* ---- Flat (toolbar-style) ---- */
.adw-button--flat {
  background-color: transparent;
  border-color: transparent;
  box-shadow: none;
}

.adw-button--flat:hover {
  background-color: rgba(128, 128, 128, 0.12);
}

.adw-button--flat:active {
  background-color: rgba(128, 128, 128, 0.22);
}

/* ---- Pill ---- */
.adw-button--pill {
  border-radius: var(--border-radius-pill);
  padding: 0 20px;
}

/* ---- Icon-only button ---- */
.adw-button--icon-only {
  width: 36px;
  padding: 0;
  border-radius: var(--border-radius-sm);
}

.adw-button--icon-only.adw-button--pill {
  width: 36px;
  border-radius: var(--border-radius-pill);
}

/* ---- Button icon ---- */
.adw-button__icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
  flex-shrink: 0;
}
```

**React (minimum JSX structure)**:

```jsx
function AdwButton({
  children,
  variant = 'default', // 'default' | 'suggested' | 'destructive' | 'flat'
  pill = false,
  iconOnly = false,
  icon,
  disabled = false,
  onClick,
  type = 'button',
  ariaLabel,
}) {
  const classes = [
    'adw-button',
    variant !== 'default' && `adw-button--${variant}`,
    pill && 'adw-button--pill',
    iconOnly && 'adw-button--icon-only',
  ].filter(Boolean).join(' ');

  return (
    <button
      className={classes}
      disabled={disabled}
      onClick={onClick}
      type={type}
      aria-label={iconOnly ? ariaLabel : undefined}
    >
      {icon && (
        <svg className="adw-button__icon" viewBox="0 0 24 24" aria-hidden="true">
          {icon}
        </svg>
      )}
      {!iconOnly && children}
    </button>
  );
}

/* Usage examples */
<AdwButton variant="suggested" onClick={handleSave}>Save</AdwButton>
<AdwButton variant="destructive" onClick={handleDelete}>Delete Account</AdwButton>
<AdwButton variant="flat" icon={<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/>} iconOnly ariaLabel="Add item" />
<AdwButton variant="suggested" pill>Get Started</AdwButton>
```

**Accessibility**: Icon-only buttons MUST have `aria-label`. Destructive buttons in dialogs should not be auto-focused.
`type="button"` prevents accidental form submission.

**Fidelity notes**: GTK distinguishes headerbar buttons (flat by default) from dialog/content buttons (raised). Web
implementations should always use `--flat` variant inside `AdwHeaderBar`. The `color-mix()` hover/active approach avoids
hardcoded shades and works correctly in both light and dark themes.

---

#### AdwSpinner

**GTK4 equivalent**: `GtkSpinner` (wrapped as `AdwSpinner` in libadwaita context)

**Anatomy**:

- Single circular arc element
- Animates with a continuous rotation + arc-length oscillation
- Inherits color from `currentColor`

**Behavioral States**:

- `spinning` — active (default)
- `stopped` — animation paused (rarely needed)

**CSS (web implementation)**:

```css
.adw-spinner {
  display: inline-block;
  width: 24px;
  height: 24px;
  border: 2.5px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: adw-spin 700ms linear infinite;
  opacity: 0.7;
  flex-shrink: 0;
}

.adw-spinner--sm {
  width: 16px;
  height: 16px;
  border-width: 2px;
}

.adw-spinner--lg {
  width: 32px;
  height: 32px;
  border-width: 3px;
}

@keyframes adw-spin {
  to { transform: rotate(360deg); }
}
```

**React (minimum JSX structure)**:

```jsx
function AdwSpinner({ size = 'md', label = 'Loading…' }) {
  const sizeClass = size !== 'md' ? `adw-spinner--${size}` : '';
  return (
    <span
      className={`adw-spinner ${sizeClass}`.trim()}
      role="status"
      aria-label={label}
    />
  );
}

/* Inline usage */
<AdwButton variant="suggested" disabled>
  <AdwSpinner size="sm" label="Saving" />
  Saving…
</AdwButton>
```

**Accessibility**: `role="status"` with `aria-label` describing the operation. For inline spinners inside buttons, the
button's label already provides context — `aria-hidden="true"` on the spinner is acceptable in that case.

**Fidelity notes**: The real GTK spinner uses a more complex animation with arc-length oscillation (the Adwaita CSS uses
a multi-keyframe stroke-dashoffset animation on an SVG circle). The CSS `border` approach above is a well-understood web
approximation. For strict fidelity, use an SVG spinner:

```jsx
function AdwSpinnerSVG({ size = 24, label = 'Loading…' }) {
  return (
    <svg
      width={size} height={size} viewBox="0 0 24 24"
      role="status" aria-label={label}
      style={{ animation: 'adw-spin 700ms linear infinite', opacity: 0.7 }}
    >
      <circle
        cx="12" cy="12" r="9"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeDasharray="28 56"
      />
    </svg>
  );
}
```

---

#### AdwAvatar

**GTK4 equivalent**: `AdwAvatar`

**Anatomy**:

- Circular container with `border-radius: 50%`
- Content layers (in priority order): custom image → user-supplied text initials → icon fallback
- Background — generated or default accent tint when showing initials/icon
- Size variants: 24px, 32px, 40px, 48px, 64px, 96px

**Behavioral States**: Display-only; no interactive states unless used as a button.

**CSS (web implementation)**:

```css
.adw-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  overflow: hidden;
  background-color: var(--accent-bg-color);
  color: var(--accent-fg-color);
  flex-shrink: 0;
  user-select: none;
}

/* Size variants */
.adw-avatar--24 { width: 24px; height: 24px; font-size: 0.625rem; font-weight: 700; }
.adw-avatar--32 { width: 32px; height: 32px; font-size: 0.75rem;  font-weight: 700; }
.adw-avatar--40 { width: 40px; height: 40px; font-size: 1rem;     font-weight: 700; }
.adw-avatar--48 { width: 48px; height: 48px; font-size: 1.125rem; font-weight: 700; }
.adw-avatar--64 { width: 64px; height: 64px; font-size: 1.375rem; font-weight: 700; }
.adw-avatar--96 { width: 96px; height: 96px; font-size: 2rem;     font-weight: 700; }

.adw-avatar__image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.adw-avatar__initials {
  line-height: 1;
  font-family: inherit;
}

.adw-avatar__icon {
  width: 55%;
  height: 55%;
  fill: currentColor;
  opacity: 0.85;
}

/* Color variants — hashed from name for consistency */
.adw-avatar--color-0 { background-color: #3584e4; }
.adw-avatar--color-1 { background-color: #2ec27e; }
.adw-avatar--color-2 { background-color: #e5a50a; }
.adw-avatar--color-3 { background-color: #e66100; }
.adw-avatar--color-4 { background-color: #9141ac; }
.adw-avatar--color-5 { background-color: #c64600; }
.adw-avatar--color-6 { background-color: #ed333b; }
.adw-avatar--color-7 { background-color: #813d9c; }
```

**React (minimum JSX structure)**:

```jsx
function getInitials(name) {
  return name
    .split(' ')
    .filter(Boolean)
    .slice(0, 2)
    .map(w => w[0].toUpperCase())
    .join('');
}

function getColorIndex(name) {
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return Math.abs(hash) % 8;
}

function AdwAvatar({ name, src, size = 40, icon, showInitials = true }) {
  const sizeClass = `adw-avatar--${size}`;
  const colorClass = !src ? `adw-avatar--color-${getColorIndex(name ?? '')}` : '';

  return (
    <div
      className={`adw-avatar ${sizeClass} ${colorClass}`.trim()}
      role="img"
      aria-label={name ?? 'Avatar'}
    >
      {src ? (
        <img className="adw-avatar__image" src={src} alt="" />
      ) : showInitials && name ? (
        <span className="adw-avatar__initials" aria-hidden="true">
          {getInitials(name)}
        </span>
      ) : (
        <svg className="adw-avatar__icon" viewBox="0 0 24 24" aria-hidden="true">
          {/* person-symbolic */}
          <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
        </svg>
      )}
    </div>
  );
}
```

**Accessibility**: `role="img"` with `aria-label` set to the person's name when known. Image `alt=""` since the outer
label covers it.

**Fidelity notes**: GTK generates avatar background colors deterministically using the contact name — the
`getColorIndex` hash above approximates this. The exact GTK color palette for avatars uses a curated set of 8 colors
defined in `adwaita-avatar-colors.scss`.

---

#### AdwBanner

**GTK4 equivalent**: `AdwBanner`

**Anatomy**:

- Full-width horizontal bar, positioned at the top of the content area (below the headerbar)
- `title` — informational text, single line, truncated with ellipsis
- `button` — optional action button on the right
- Background — uses `--accent-bg-color` by default; can be `--warning-bg-color` or `--error-bg-color`

**Behavioral States**:

- `visible` — slides down from headerbar
- `hidden` — collapsed, height 0

**CSS (web implementation)**:

```css
.adw-banner {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 12px;
  background-color: var(--accent-bg-color);
  color: var(--accent-fg-color);
  min-height: 44px;
  overflow: hidden;
  max-height: 80px;
  transition:
    max-height var(--transition-duration-normal) var(--transition-easing-default),
    padding    var(--transition-duration-normal) var(--transition-easing-default);
}

.adw-banner--hidden {
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.adw-banner--warning {
  background-color: var(--warning-bg-color);
  color: var(--warning-fg-color);
}

.adw-banner--error {
  background-color: var(--error-bg-color);
  color: var(--error-fg-color);
}

.adw-banner__title {
  font-size: 0.9375rem;
  font-weight: 400;
  flex: 1;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.adw-banner__button {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: var(--border-radius-pill);
  color: inherit;
  font-size: 0.875rem;
  font-weight: 700;
  padding: 4px 12px;
  height: 28px;
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color var(--transition-duration-fast) var(--transition-easing-default);
}

.adw-banner__button:hover {
  background: rgba(255, 255, 255, 0.3);
}

.adw-banner__button:focus-visible {
  outline: 2px solid rgba(255, 255, 255, 0.8);
  outline-offset: 2px;
}
```

**React (minimum JSX structure)**:

```jsx
function AdwBanner({
  visible = true,
  title,
  buttonLabel,
  onButtonClick,
  variant = 'accent', // 'accent' | 'warning' | 'error'
}) {
  const classes = [
    'adw-banner',
    !visible && 'adw-banner--hidden',
    variant !== 'accent' && `adw-banner--${variant}`,
  ].filter(Boolean).join(' ');

  return (
    <div
      className={classes}
      role="status"
      aria-live="polite"
      aria-hidden={!visible}
    >
      <span className="adw-banner__title">{title}</span>
      {buttonLabel && (
        <button className="adw-banner__button" onClick={onButtonClick}>
          {buttonLabel}
        </button>
      )}
    </div>
  );
}

/* Usage */
<AdwBanner
  visible={updateAvailable}
  title="A new version is available."
  buttonLabel="Update"
  onButtonClick={handleUpdate}
/>
```

**Accessibility**: `role="status"` with `aria-live="polite"` so screen readers announce it when it appears.
`aria-hidden={!visible}` prevents screen reader traversal when collapsed. The button's accessible name is its label
text.

**Fidelity notes**: GTK's `AdwBanner` slides in with a reveal animation (height expansion). The `max-height` CSS
transition above approximates this cleanly. GTK does not support multi-line banner text — enforce `white-space: nowrap`
strictly.

---

## Section 3 — Layout and Responsiveness

### 3.1 GNOME Window Model

A typical GNOME application window follows this hierarchy:

```
AdwApplicationWindow
└── AdwToolbarView
    ├── AdwHeaderBar (top toolbar)
    ├── content area (flex: 1, overflow-y: auto)
    │   └── AdwPreferencesPage / AdwClamp / custom widget
    └── AdwViewSwitcher (bottom, narrow only — optional)
```

The key structural rule: **the content area is the scroll container**, not the window. The `AdwHeaderBar` and bottom bar
remain fixed. Content scrolls beneath them.

```jsx
function AdwApplicationWindow({ title, children, pages, currentPage, onPageChange }) {
  const [scrolled, setScrolled] = useState(false);
  const contentRef = useRef(null);

  const handleScroll = () => {
    setScrolled(contentRef.current?.scrollTop > 4);
  };

  return (
    <div
      className="adw-application-window"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: 'var(--window-bg-color)',
        color: 'var(--window-fg-color)',
        overflow: 'hidden',
      }}
    >
      {/* AdwToolbarView — top */}
      <AdwHeaderBar title={title} raised={scrolled} />

      {/* AdwBanner (if needed) */}

      {/* Content area — the ONLY scroll container */}
      <main
        ref={contentRef}
        onScroll={handleScroll}
        style={{
          flex: 1,
          overflowY: 'auto',
          backgroundColor: 'var(--window-bg-color)',
        }}
      >
        {children}
      </main>

      {/* AdwViewSwitcher — bottom (narrow only) */}
      {pages && (
        <AdwViewSwitcher
          pages={pages}
          activePage={currentPage}
          onPageChange={onPageChange}
          position="bottom"
        />
      )}
    </div>
  );
}
```

---

### 3.2 AdwBreakpoint — Adaptive Behavior

GNOME breakpoints are **functional**, not decorative. They drive structural layout changes.

| Breakpoint | Width         | Behavior                                                                                                          |
|------------|---------------|-------------------------------------------------------------------------------------------------------------------|
| `narrow`   | < 360px       | Single column, bottom navigation bar (ViewSwitcher), no sidebar, full-width cards                                 |
| `default`  | 360px – 720px | Normal single-column content, header ViewSwitcher, sidebar hidden or overlaying                                   |
| `wide`     | > 720px       | Sidebar visible in split-view, content/sidebar side by side, header ViewSwitcher (or none if sidebar carries nav) |

**Changes per breakpoint:**

| Element       | narrow                   | default                      | wide                          |
|---------------|--------------------------|------------------------------|-------------------------------|
| Navigation    | Bottom `AdwViewSwitcher` | Header `AdwViewSwitcher`     | Sidebar list                  |
| Sidebar       | Hidden                   | Hidden or slide-over overlay | Persistent, 260px wide        |
| Header        | Compact (no subtitle)    | Full title + subtitle        | Full, may include search      |
| Content width | 100%                     | 100% (max 720px clamped)     | Remaining width after sidebar |
| Card margins  | 0 (edge-to-edge)         | 12px horizontal              | 12px horizontal               |

```css
/* Breakpoint variables */
:root {
  --bp-narrow:  360px;
  --bp-wide:    720px;
}

/* Narrow: show bottom bar, hide sidebar */
@media (max-width: 359px) {
  .adw-view-switcher--header { display: none; }
  .adw-view-switcher--bottom { display: flex; }
  .adw-sidebar { display: none; }
  .adw-preferences-group { margin: 0; }
  .adw-preferences-group__listbox { border-radius: 0; border-left: none; border-right: none; }
}

/* Wide: show sidebar */
@media (min-width: 720px) {
  .adw-split-view {
    display: grid;
    grid-template-columns: 260px 1fr;
  }
  .adw-sidebar { display: flex; }
  .adw-view-switcher--bottom { display: none; }
}
```

---

### 3.3 Content Margins and Padding

| Context                    | Horizontal margin | Internal padding       |
|----------------------------|-------------------|------------------------|
| Window narrow (< 360px)    | 0 (edge-to-edge)  | 12px                   |
| Window default (360–720px) | 12px              | 18px                   |
| Dialog                     | 24px              | 24px                   |
| Card / ActionRow           | —                 | 12px                   |
| PreferencesGroup           | 12px              | — (listbox handles it) |
| ViewSwitcher button        | —                 | 8px horizontal         |

---

### 3.4 AdwClamp — Maximum Content Width

`AdwClamp` constrains legible content width to prevent excessively wide line lengths and overstretched UI elements.

| View Type             | `max-width` | Notes                              |
|-----------------------|-------------|------------------------------------|
| Preferences / Forms   | `400px`     | `AdwPreferencesPage` default clamp |
| Editorial / Documents | `720px`     | Long-form content, articles        |
| Wide lists / tables   | none        | Data grids fill available space    |
| Dialogs               | `480–560px` | Handled by dialog component        |

```css
.adw-clamp {
  width: 100%;
  margin-left: auto;
  margin-right: auto;
}

.adw-clamp--sm  { max-width: 400px; }
.adw-clamp--md  { max-width: 560px; }
.adw-clamp--lg  { max-width: 720px; }
.adw-clamp--xl  { max-width: 960px; }
.adw-clamp--none { max-width: none; }
```

```jsx
function AdwClamp({ children, size = 'sm', className = '', style = {} }) {
  return (
    <div
      className={`adw-clamp adw-clamp--${size} ${className}`.trim()}
      style={{ padding: '0 12px', ...style }}
    >
      {children}
    </div>
  );
}
```

---

## Section 4 — Icon System

GNOME uses **symbolic icons** — monochromatic, single-path SVGs that inherit color through `currentColor`. They look
like outlined glyphs, not full-color illustrations.

### Standard Sizes

| Size   | Use context                                         |
|--------|-----------------------------------------------------|
| `16px` | Inline text decorations, small badges, menu items   |
| `24px` | Buttons, list rows (`AdwActionRow` prefix), toolbar |
| `32px` | Section headers, prominent list icons               |
| `64px` | `AdwStatusPage` empty states, error screens         |
| `96px` | Welcome screens, onboarding illustrations           |

### SVG Implementation

Every symbolic icon must use `fill: currentColor` (never hardcoded colors) and a `24×24` viewBox:

```jsx
function SymbolicIcon({ path, size = 24, className = '', ariaLabel }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="currentColor"
      aria-hidden={!ariaLabel}
      aria-label={ariaLabel}
      className={className}
      style={{ flexShrink: 0 }}
    >
      <path d={path} />
    </svg>
  );
}
```

```css
/* Icon color inherits from parent; override contextually */
.adw-action-row__prefix svg  { opacity: 0.75; }   /* dimmed prefix icons */
.adw-status-page__icon       { opacity: 0.35; }   /* muted empty-state icons */
.adw-button--suggested svg   { color: var(--accent-fg-color); } /* white on accent */
```

### Recommended Library

For web implementations, use the **GNOME Icon Library** canonical set:

- npm: `@gnome-icon-library/symbolic` `[to be verified in the official documentation]`
- CDN: The icons are available as individual SVGs from the GNOME GitLab repository (
  `gitlab.gnome.org/World/design/icon-library`)
- Alternative: Bundle only the icons your app uses as inline SVG constants

### Canonical Icon Names by Context

| Context                  | Icon name                     | Notes                     |
|--------------------------|-------------------------------|---------------------------|
| Navigate forward / next  | `go-next-symbolic`            | Chevron right             |
| Navigate back / previous | `go-previous-symbolic`        | Chevron left              |
| Delete / remove          | `edit-delete-symbolic`        | Trash can                 |
| Edit / rename            | `document-edit-symbolic`      | Pencil                    |
| Add / new                | `list-add-symbolic`           | Plus sign                 |
| Settings / preferences   | `preferences-system-symbolic` | Gear                      |
| Search                   | `edit-find-symbolic`          | Magnifier                 |
| Close / dismiss          | `window-close-symbolic`       | X                         |
| Overflow / more          | `view-more-symbolic`          | Three dots                |
| Hamburger menu           | `open-menu-symbolic`          | Three lines               |
| Refresh                  | `view-refresh-symbolic`       | Circular arrow            |
| Download                 | `folder-download-symbolic`    | Down arrow into folder    |
| Upload / send            | `document-send-symbolic`      | Document with arrow       |
| Warning                  | `dialog-warning-symbolic`     | Triangle with exclamation |
| Error                    | `dialog-error-symbolic`       | Circle with X             |
| Information              | `dialog-information-symbolic` | Circle with i             |
| Checkmark / done         | `object-select-symbolic`      | Check mark                |
| Starred / favorite       | `starred-symbolic`            | Star                      |
| Person / contact         | `person-symbolic`             | Human silhouette          |

---

## Section 5 — Best Practices and UX

### 5.1 GNOME HIG Simplification Principles

#### 1. Direct Controls Instead of Nested Menus

**Principle**: Place the most common actions directly on-screen. Reserve menus for secondary and contextual actions.

**Implementation decisions**:

- Primary action goes in the headerbar's `end` slot as a visible button, not in a menu
- Destructive actions (delete, reset) go in a menu or alert, never in a primary CTA position
- Maximum 3–4 items in a headerbar; use `view-more-symbolic` overflow for the rest
- `AdwActionRow` with a suffix widget (Switch, chevron) replaces toggle menus and sub-menus
- Never put more than 2 levels of navigation depth in a `AdwNavigationView` stack

#### 2. Destructive Actions with Explicit Confirmation

**Principle**: The user must consciously choose to destroy data. Make it hard to do accidentally.

**Implementation decisions**:

- All irreversible destructive actions (delete, clear, reset) must trigger `AdwAlertDialog`
- The confirmation dialog's primary button must use `.adw-button--destructive`
- The destructive button must NOT be the default-focused element on dialog open
- Cancel must always be available and keyboard-accessible (Escape key)
- Undo (via `AdwToast` with action button) is preferred over confirmation dialogs for reversible deletions
- Do not use a single-step "hold to delete" pattern — GNOME HIG requires explicit confirmation UI

#### 3. Settings Accessible Without Restarting State

**Principle**: Preference changes apply immediately and non-destructively.

**Implementation decisions**:

- Settings changes in `AdwPreferencesGroup` apply on toggle/change, not on a Save button press
- If a change requires a restart, show `AdwBanner` ("Restart required to apply changes. [Restart Now]") — never block
  the UI
- Use `localStorage` or app state to persist preferences immediately
- Validation errors appear inline below the input field (not in a blocking dialog)
- Never navigate away from a settings page to apply changes

#### 4. Immediate Feedback for Every User Action

**Principle**: Every tap, click, or keyboard action has a visible response within 100ms.

**Implementation decisions**:

- Buttons get `:active` CSS state for tactile press feedback (translateY(1px), darker background)
- Loading states use `AdwSpinner` inline or overlay; never leave the UI frozen silently
- Success/failure of async operations always trigger `AdwToast`
- Form validation: show inline errors on blur, not only on submit
- Switches animate their state change (100ms transition on the thumb position)
- Row activations show a brief press highlight before navigation

#### 5. Adaptation to Context (Narrow vs. Wide) Without Breaking Flow

**Principle**: The app must be fully functional at all supported widths. Layout changes, but no features are hidden.

**Implementation decisions**:

- Use `ResizeObserver` (not `window.resize`) to detect container width changes
- Navigation switches between header `AdwViewSwitcher` (default) and bottom bar (narrow) based on window width — not
  user preference
- `AdwPreferencesGroup` listboxes go edge-to-edge on narrow (remove border-radius and horizontal margins)
- The sidebar collapses to a slide-over overlay on default widths; auto-dismisses when a row is tapped
- `AdwClamp` prevents content from stretching beyond readable width on wide screens — always wrap preference content in
  a clamp
- Never hide essential controls at any breakpoint — reduce prominence instead

---

### 5.2 Accessibility

| Requirement                      | Value / Standard                                              | Implementation                                  |
|----------------------------------|---------------------------------------------------------------|-------------------------------------------------|
| Text/background contrast (body)  | 4.5:1 minimum (WCAG AA)                                       | All foreground/background token pairs meet this |
| Text/background contrast (large) | 3:1 minimum                                                   | Headings ≥ 24px or bold ≥ 18.66px               |
| Interactive element contrast     | 3:1 vs adjacent                                               | Button borders, focus rings                     |
| Visible focus state              | `outline: 2px solid var(--accent-color); outline-offset: 2px` | All focusable elements                          |
| Minimum touch target             | 44×44px                                                       | Buttons, switches, action rows, nav items       |
| ARIA landmark roles              | `banner`, `navigation`, `main`, `region`, `dialog`            | Per component, see below                        |
| Screen reader text               | `aria-label` or visible label                                 | Icon-only buttons must have `aria-label`        |
| Motion safety                    | `prefers-reduced-motion`                                      | All transition/animation durations → 0ms or 1ms |

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 1ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 1ms !important;
  }
}
```

**ARIA attributes by component:**

| Component                    | Required ARIA                                                                            |
|------------------------------|------------------------------------------------------------------------------------------|
| `AdwHeaderBar`               | `role="banner"` on `<header>`                                                            |
| `AdwViewSwitcher`            | `role="tablist"` on container; `role="tab"`, `aria-selected`, `aria-controls` on buttons |
| `AdwNavigationView`          | `role="region"` on each page; `aria-label` = page title                                  |
| `AdwActionRow` (activatable) | `<button>` element; `aria-disabled` when disabled                                        |
| `AdwExpanderRow`             | `aria-expanded` on trigger; `role="region"` on children                                  |
| `AdwPreferencesGroup`        | `<section>` with `aria-labelledby` group title                                           |
| `AdwStatusPage`              | `role="status"` + `aria-live="polite"`                                                   |
| `AdwToast`                   | `role="alert"` + `aria-live="assertive"` + `aria-atomic="true"`                          |
| `AdwDialog`                  | `role="dialog"` + `aria-modal="true"` + `aria-labelledby`                                |
| `AdwBanner`                  | `role="status"` + `aria-live="polite"` + `aria-hidden` when invisible                    |
| `AdwSpinner`                 | `role="status"` + `aria-label`                                                           |
| `AdwAvatar`                  | `role="img"` + `aria-label`                                                              |

---

### 5.3 Dark Mode

#### Applying the Theme

**Method 1 — Data attribute (explicit)**:

```js
document.documentElement.setAttribute('data-theme', 'dark');
document.documentElement.removeAttribute('data-theme'); // revert to light
```

**Method 2 — CSS media query (system-follow)**:

```css
@media (prefers-color-scheme: dark) {
  :root {
    /* Paste all [data-theme="dark"] values here for system-follow */
    --window-bg-color: #242424;
    /* ... */
  }
}
```

**Method 3 — Combined (user override + system default)**:

```css
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --window-bg-color: #242424;
    /* ... */
  }
}
[data-theme="dark"] {
  --window-bg-color: #242424;
  /* ... */
}
```

#### Which Tokens Change

All tokens defined in the `[data-theme="dark"]` block in Section 1.1 change between themes. High-impact changes:

- All `--*-bg-color` surface tokens shift to dark values
- `--accent-color` shifts from `#2373c7` (dark text, high contrast on light bg) to `#78aeed` (lighter, maintains
  contrast on dark bg)
- `--destructive-color`, `--error-color` shift to lighter values for dark background contrast
- `--border-color` inverts from `rgba(0,0,0,0.12)` to `rgba(255,255,255,0.12)`
- `--shade-color` intensifies in dark mode (covers more alpha range)

#### Theme Transition

Apply smooth transitions on theme toggle:

```css
html {
  transition:
    background-color 200ms ease,
    color 200ms ease;
}

/* Extend to key surfaces */
.adw-application-window,
.adw-header-bar,
.adw-preferences-group__listbox,
.adw-action-row,
.adw-dialog,
.adw-card {
  transition:
    background-color 200ms ease,
    border-color 200ms ease,
    color 200ms ease;
}
```

#### Common Dark Mode Pitfalls

| Pitfall                                     | Problem                                     | Fix                                                                                                          |
|---------------------------------------------|---------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| Hardcoded hex values in component CSS       | Color doesn't change on theme switch        | Replace every `#hex` with the appropriate CSS token                                                          |
| Images without dark variant                 | Bright images jarring on dark background    | Use `<picture>` with `prefers-color-scheme` media or apply `filter: brightness(0.85)` on images in dark mode |
| `box-shadow` with fixed rgba black          | Shadows invisible on dark surface           | Use `--shadow-elevated-N` tokens which have separate light/dark values                                       |
| White `background-color` in forms           | Inputs look like glowing boxes in dark mode | Use `var(--view-bg-color)` for input backgrounds                                                             |
| SVG icons with fill colors                  | Icon wrong color in dark mode               | Use `fill: currentColor` on all SVG paths; never inline `fill="#hex"`                                        |
| Z-index stacking without opacity adjustment | Overlays too light/dark                     | Scrim `rgba(0,0,0,0.5)` works for both themes but verify contrast                                            |

---

## Section 6 — Tailwind Configuration

```js
// tailwind.config.js
module.exports = {
  content: ['./src/**/*.{js,jsx,ts,tsx}', './index.html'],
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {

      // ── Font families ──────────────────────────────────────────────
      // GNOME 47+: Inter is the system font. Cantarell is legacy (GNOME ≤46).
      fontFamily: {
        gnome: ['Inter', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'Helvetica', 'Arial', 'sans-serif'],
        sans:  ['Inter', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono:  ['"Source Code Pro"', '"Cascadia Code"', '"Fira Code"', 'ui-monospace', 'monospace'],
        // Legacy — use only when targeting GNOME 46 or earlier:
        'gnome-legacy': ['Cantarell', 'Inter', 'sans-serif'],
      },

      // ── Font sizes (maps to .text-gnome-* utility classes) ─────────
      fontSize: {
        'gnome-display':         ['2.5rem',    { lineHeight: '1.2',  letterSpacing: '-0.02em',  fontWeight: '700' }],
        'gnome-title-1':         ['2rem',      { lineHeight: '1.25', letterSpacing: '-0.015em', fontWeight: '700' }],
        'gnome-title-2':         ['1.5rem',    { lineHeight: '1.3',  letterSpacing: '-0.01em',  fontWeight: '700' }],
        'gnome-title-3':         ['1.25rem',   { lineHeight: '1.35', letterSpacing: '-0.005em', fontWeight: '700' }],
        'gnome-title-4':         ['1.125rem',  { lineHeight: '1.4',  letterSpacing: '0',        fontWeight: '700' }],
        'gnome-heading':         ['0.875rem',  { lineHeight: '1.4',  letterSpacing: '0.04em',   fontWeight: '700' }],
        'gnome-body':            ['1rem',      { lineHeight: '1.5',  letterSpacing: '0',        fontWeight: '400' }],
        'gnome-caption':         ['0.75rem',   { lineHeight: '1.4',  letterSpacing: '0',        fontWeight: '400' }],
        'gnome-caption-heading': ['0.75rem',   { lineHeight: '1.4',  letterSpacing: '0.06em',   fontWeight: '700' }],
      },

      // ── Spacing (8px grid, gnome-* prefix) ─────────────────────────
      spacing: {
        'gnome-1':  '0.25rem',   //  4px
        'gnome-2':  '0.5rem',    //  8px
        'gnome-3':  '0.75rem',   // 12px
        'gnome-4':  '1rem',      // 16px
        'gnome-5':  '1.25rem',   // 20px
        'gnome-6':  '1.5rem',    // 24px
        'gnome-7':  '1.75rem',   // 28px
        'gnome-8':  '2rem',      // 32px
        'gnome-9':  '2.25rem',   // 36px
        'gnome-10': '2.5rem',    // 40px
        'gnome-11': '2.75rem',   // 44px
        'gnome-12': '3rem',      // 48px
      },

      // ── Colors (CSS var-backed, work with bg-gnome-*, text-gnome-*) ─
      colors: {
        gnome: {
          // Window / Surface
          'window-bg':     'var(--window-bg-color)',
          'view-bg':       'var(--view-bg-color)',
          'card-bg':       'var(--card-bg-color)',
          'popover-bg':    'var(--popover-bg-color)',
          'dialog-bg':     'var(--dialog-bg-color)',

          // Headerbar
          'headerbar-bg':       'var(--headerbar-bg-color)',
          'headerbar-fg':       'var(--headerbar-fg-color)',
          'headerbar-border':   'var(--headerbar-border-color)',

          // Sidebar
          'sidebar-bg':  'var(--sidebar-bg-color)',
          'sidebar-fg':  'var(--sidebar-fg-color)',

          // Foreground
          'window-fg':  'var(--window-fg-color)',
          'view-fg':    'var(--view-fg-color)',
          'card-fg':    'var(--card-fg-color)',

          // Accent
          'accent-bg':  'var(--accent-bg-color)',
          'accent-fg':  'var(--accent-fg-color)',
          'accent':     'var(--accent-color)',

          // Destructive
          'destructive-bg':  'var(--destructive-bg-color)',
          'destructive-fg':  'var(--destructive-fg-color)',
          'destructive':     'var(--destructive-color)',

          // Status
          'success-bg':  'var(--success-bg-color)',
          'success-fg':  'var(--success-fg-color)',
          'success':     'var(--success-color)',
          'warning-bg':  'var(--warning-bg-color)',
          'warning-fg':  'var(--warning-fg-color)',
          'warning':     'var(--warning-color)',
          'error-bg':    'var(--error-bg-color)',
          'error-fg':    'var(--error-fg-color)',
          'error':       'var(--error-color)',

          // Borders
          'border':  'var(--border-color)',
          'shade':   'var(--shade-color)',
        },
      },

      // ── Border radius ───────────────────────────────────────────────
      borderRadius: {
        'gnome-xs':   '4px',
        'gnome-sm':   '6px',
        'gnome-md':   '12px',
        'gnome-lg':   '16px',
        'gnome-pill': '9999px',
      },

      // ── Box shadows (elevation levels) ──────────────────────────────
      boxShadow: {
        'gnome-0': 'none',
        'gnome-1': 'var(--shadow-elevated-1)',
        'gnome-2': 'var(--shadow-elevated-2)',
      },

      // ── Transition durations ────────────────────────────────────────
      transitionDuration: {
        'gnome-fast':   '100ms',
        'gnome-normal': '200ms',
        'gnome-slow':   '400ms',
      },

      // ── Transition timing functions ─────────────────────────────────
      transitionTimingFunction: {
        'gnome-default':    'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
        'gnome-spring':     'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'gnome-decelerate': 'cubic-bezier(0.0, 0.0, 0.2, 1)',
        'gnome-accelerate': 'cubic-bezier(0.4, 0.0, 1, 1)',
      },

      // ── Max widths (AdwClamp equivalents) ───────────────────────────
      maxWidth: {
        'gnome-form':      '400px',
        'gnome-dialog':    '480px',
        'gnome-dialog-lg': '560px',
        'gnome-content':   '720px',
        'gnome-wide':      '960px',
      },

      // ── Min heights (touch targets) ─────────────────────────────────
      minHeight: {
        'gnome-target': '44px',
        'gnome-row':    '56px',
        'gnome-header': '48px',
      },
    },
  },

  // ── Plugins: add .text-gnome-* utilities if using @layer ──────────
  plugins: [
    function ({ addUtilities }) {
      addUtilities({
        '.text-gnome-display':         { fontSize: '2.5rem',   fontWeight: '700', lineHeight: '1.2',  letterSpacing: '-0.02em' },
        '.text-gnome-title-1':         { fontSize: '2rem',     fontWeight: '700', lineHeight: '1.25', letterSpacing: '-0.015em' },
        '.text-gnome-title-2':         { fontSize: '1.5rem',   fontWeight: '700', lineHeight: '1.3',  letterSpacing: '-0.01em' },
        '.text-gnome-title-3':         { fontSize: '1.25rem',  fontWeight: '700', lineHeight: '1.35', letterSpacing: '-0.005em' },
        '.text-gnome-title-4':         { fontSize: '1.125rem', fontWeight: '700', lineHeight: '1.4',  letterSpacing: '0' },
        '.text-gnome-heading':         { fontSize: '0.875rem', fontWeight: '700', lineHeight: '1.4',  letterSpacing: '0.04em', textTransform: 'uppercase' },
        '.text-gnome-body':            { fontSize: '1rem',     fontWeight: '400', lineHeight: '1.5',  letterSpacing: '0' },
        '.text-gnome-caption':         { fontSize: '0.75rem',  fontWeight: '400', lineHeight: '1.4',  letterSpacing: '0' },
        '.text-gnome-caption-heading': { fontSize: '0.75rem',  fontWeight: '700', lineHeight: '1.4',  letterSpacing: '0.06em', textTransform: 'uppercase' },

        // Dim label utility
        '.text-gnome-dim': { opacity: '0.55' },
      });
    },
  ],
};
```

### Tailwind Usage Examples

```jsx
// With Tailwind tokens:
<div className="bg-gnome-card-bg text-gnome-card-fg rounded-gnome-md shadow-gnome-1 p-gnome-3">
  Card surface
</div>

<button className="bg-gnome-accent-bg text-gnome-accent-fg rounded-gnome-sm h-gnome-target px-gnome-4 text-gnome-body font-medium">
  Suggested action
</button>

<p className="text-gnome-body text-gnome-window-fg text-gnome-dim">
  Secondary description text
</p>

<header className="bg-gnome-headerbar-bg text-gnome-headerbar-fg h-gnome-header border-b border-gnome-border flex items-center px-gnome-2">
  Header bar
</header>
```

> **Note**: `bg-gnome-card-bg` works because Tailwind resolves `colors.gnome['card-bg']` → `var(--card-bg-color)`, which
> is resolved by the browser at paint time. This means dark mode toggling (via `[data-theme="dark"]` on `<html>`)
> automatically updates all Tailwind classes that reference these tokens — no `dark:` prefix variants needed.
