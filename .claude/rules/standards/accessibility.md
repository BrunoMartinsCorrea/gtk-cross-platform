---
description: GTK4/Adwaita accessibility standards — icon-only buttons, AccessibleRole, focus management, touch targets, and color-state prohibition. Auto-loaded when editing UI code in src/window/ or data/resources/.
globs: ["src/window/**", "data/resources/**"]
---

# Accessibility (A11Y) Standards

## Icon-only buttons

Every icon-only button must have both a tooltip and an accessible label. Tooltip alone is not read
by screen readers:

```rust
let btn = gtk4::Button::new();
btn.set_icon_name("list-remove-symbolic");
btn.set_tooltip_text(Some(&gettext("Remove")));
btn.update_property(&[gtk4::accessible::Property::Label(&gettext("Remove"))]);
```

Use `resource_row::icon_button(icon, tooltip)` (`src/window/components/resource_row.rs`) — it
applies both properties in one call.

## AccessibleRole for live regions

Dynamic content that updates without a page reload must declare its semantic role:

```rust
// Status badge — announces state changes to screen readers
let label = gtk4::Label::builder()
    .label(status.label())
    .accessible_role(gtk4::AccessibleRole::Status)
    .build();

// Loading spinner
spinner.update_property(&[gtk4::accessible::Property::Label(&gettext("Loading…"))]);
```

`AccessibleRole::Status` marks the element as an ARIA live region — changes are announced without
requiring focus.

## Focus management

| Situation                       | Required action                                                                   |
|---------------------------------|-----------------------------------------------------------------------------------|
| Modal dialog closes             | Return focus to the widget that triggered it (in `connect_response`)              |
| Destructive action (remove row) | Move focus to the next row, or to the empty-state widget if the list is now empty |
| Navigation between panes        | `grab_focus()` on the default widget of the new pane                              |

## Touch targets

All interactive elements must meet the GNOME HIG minimum touch target:

```css
/* data/resources/style.css */
.action-button {
  min-width:  44px;
  min-height: 44px;
}
```

Apply the `.action-button` CSS class to any button that is an icon-only action button. The 44×44 sp
minimum applies to the entire tap area, not just the icon.

## Color-only state indicators

Color must never be the sole way to convey state — mandatory for WCAG 1.4.1 (colour contrast):

```rust
// WRONG — color alone (fails for colorblind users)
badge.set_css_classes(&["status-running"]);

// CORRECT — color + text label (StatusBadge enforces this)
let badge = StatusBadge::new(&container.status);  // renders color + status text
```

`StatusBadge` (`src/window/components/status_badge.rs`) is the canonical component — it always
renders a text label alongside the color indicator.

## GTK4 named color tokens

Use `@named-color` values from libadwaita instead of hardcoded hex. They adapt automatically to
dark mode and high-contrast themes:

| Token                   | Use case                    |
|-------------------------|-----------------------------|
| `@success_color`        | Running / healthy state     |
| `@warning_color`        | Paused / degraded state     |
| `@error_color`          | Stopped / failed state      |
| `@accent_color`         | Selected / active highlight |
| `@destructive_bg_color` | Destructive action button   |

## A11Y checklist

| Situation          | Requirement                                                   |
|--------------------|---------------------------------------------------------------|
| Icon-only button   | `set_tooltip_text` + `update_property(Property::Label)`       |
| Status badge       | `accessible_role(AccessibleRole::Status)` + visible text      |
| Modal dialog       | Return focus to trigger widget in `connect_response`          |
| Destructive remove | Move focus to next row or empty-state                         |
| Loading spinner    | `update_property(Property::Label)` with operation description |
| Color as state     | Always pair with text — never color alone                     |
| Touch target       | ≥ 44×44 sp via `.action-button` CSS class                     |
