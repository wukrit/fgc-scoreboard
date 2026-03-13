---
title: Neo-Brutalist Controller Redesign with Pico CSS and Score Stepper Pattern
date: 2026-03-13
category: ui-redesign
tags:
  - pico-css
  - neo-brutalist
  - css-framework
  - score-stepper
  - readonly-input-pattern
  - mobile-ux
  - xss-fix
  - race-condition
  - dark-mode
  - touch-optimization
module: controller
symptoms:
  - Raw number inputs require typing, awkward one-handed on phone
  - No visual distinction between P1 and P2 sections
  - Dated aesthetic with no design language
  - XSS vulnerability in overlay via .html() calls
  - Race conditions in showStatus() and concurrent save()
root_cause: Controller UX not optimized for one-handed mobile use at tournaments
---

# Neo-Brutalist Controller Redesign with Pico CSS

## Problem

The FGC Scoreboard controller (`controller.html`) had several UX, security, and reliability issues:

- **Score entry**: Raw `<input type="number">` required typing on a phone keyboard — awkward one-handed mid-tournament-set
- **No player distinction**: P1 and P2 sections looked identical
- **Dated styling**: Basic dark theme with no design language
- **XSS vulnerability**: Overlay rendered user data via jQuery `.html()`, allowing script injection
- **Race conditions**: `showStatus()` timeout could hide in-flight save status; concurrent `save()` calls could POST out of order

## Solution

### 1. Readonly Input Pattern for Score Stepper (Key Insight)

Replaced `<input type="number">` with `<input type="text" readonly>` flanked by +/- buttons.

**Why this matters:** `.value` works identically on both `type="number"` and `type="text"` inputs. This meant all five existing JS functions (`getFormData`, `populateForm`, `swap`, `resetScores`, `clearAll`) needed **zero code changes**. The DOM element changed, but the property interface stayed the same.

```html
<div class="score-stepper">
  <button type="button" class="score-btn"
    onclick="adjustScore('p1Score', -1)" tabindex="-1"
    aria-label="Decrease P1 score">&minus;</button>
  <input type="text" readonly id="p1Score" value="0" class="score-display">
  <button type="button" class="score-btn"
    onclick="adjustScore('p1Score', 1)" tabindex="-1"
    aria-label="Increase P1 score">+</button>
</div>
```

**Auto-save on score change**: Since stepper values are always in a "complete" state (unlike number inputs where the user might be mid-typing), `adjustScore()` safely calls `save()` automatically — eliminating a tap from the operator's workflow. The `isSaving` guard prevents concurrent POSTs.

### 2. Pico CSS (Vendored, Classless Variant)

- Used `pico.classless.min.css` (not the full variant) — smaller, no unused modal/nav/tooltip styles
- **Vendored locally** at `css/pico.classless.min.css` — LAN tournaments have no internet
- Forced dark theme via `data-theme="dark"` on `<html>` element

**Gotcha — dual dark-mode selectors:** Pico v2 defines dark variables in both `[data-theme=dark]` and `@media (prefers-color-scheme: dark)`. Overrides must appear in both, or users reaching dark mode via the other path won't see them. Forcing `data-theme="dark"` avoids this entirely.

### 3. Performance: Transform-Only Transitions

Button press effects use `transition: transform 0.08s` — no `box-shadow` in the transition. `transform` is GPU-composited; `box-shadow` triggers per-frame repaints. The shadow snaps instantly while the element moves, and the 80ms duration is too short for users to notice.

```css
button {
  box-shadow: 4px 4px 0 #000;
  transition: transform 0.08s;
}
button:active {
  transform: translate(4px, 4px);
  box-shadow: 0 0 0 #000; /* snaps, not animated */
}
```

### 4. Browser Compatibility: rgba() over color-mix()

Used `rgba(255, 68, 68, 0.06)` instead of `color-mix(in srgb, ...)` for P1/P2 background tints. Tournament PCs often run older browsers; `color-mix()` lacks support before Chrome 111 / Firefox 113.

### 5. Mobile Touch UX (Near-Zero-Cost Wins)

| Technique | Purpose |
|-----------|---------|
| `touch-action: manipulation` | Eliminates 300ms tap delay on mobile Safari |
| `user-select: none` | Prevents accidental text selection during rapid tapping |
| `-webkit-tap-highlight-color: transparent` | Removes iOS blue flash (we provide shadow-shift feedback) |
| `font-variant-numeric: tabular-nums` | Prevents score display width shift between digits |
| `tabindex="-1"` on +/- buttons | Prevents keyboard popup when tapping stepper |
| 48px min touch targets | WCAG / Material Design guidelines |

### 6. Bug Fixes

**showStatus() stale timeout:** Store timeout ID, `clearTimeout` before setting new status. Prevents previous timeout from hiding in-flight save indicator.

**Concurrent save() race:** `isSaving` boolean guard with `.finally()` cleanup. Prevents overlapping POSTs from arriving at the server out of order.

**Defense-in-depth score clamp:** `getFormData()` now clamps scores `Math.max(0, Math.min(99, ...))` before sending, since `readonly` is a UI hint, not a security boundary.

### 7. XSS Fix in Overlay

Changed all `$('#p1Name').html(p1Name)` calls to `.text()` in `_overlays/js/scoreboard.js`. The overlay only displays plain text — `.html()` was interpreting HTML, enabling stored XSS via malicious player names.

### 8. Layout Shift Prevention

Status indicator moved below the form and uses `visibility: hidden` (not `display: none`) to always reserve its space. Elements above it never shift when save status appears/disappears.

## Prevention

- **Prefer `<input type="text" readonly>` over hidden inputs + display spans** when replacing input types — preserves `.value` contract
- **Vendor CSS frameworks** for offline/LAN use cases — never depend on CDN for tournament tools
- **Always use `.text()` (not `.html()`)** when rendering user-supplied data in jQuery
- **Guard async operations** with simple boolean flags + `.finally()` to prevent concurrent execution
- **Use `visibility: hidden`** (not `display: none`) for status indicators that should reserve space

## Related Documentation

- Brainstorm: `docs/brainstorms/2026-03-13-controller-redesign-brainstorm.md`
- Implementation plan: `docs/plans/2026-03-13-feat-controller-neo-brutalist-redesign-plan.md`
- StreamControl migration: `docs/solutions/integration-issues/streamcontrol-to-npoint-remote-controller.md`
- Pico CSS docs: https://picocss.com/docs/css-variables
- Neo-brutalist principles: https://www.nngroup.com/articles/neobrutalism/
