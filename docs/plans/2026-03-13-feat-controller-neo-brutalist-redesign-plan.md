---
title: "feat: Controller Neo-Brutalist Redesign"
type: feat
date: 2026-03-13
---

# feat: Controller Neo-Brutalist Redesign

## Enhancement Summary

**Deepened on:** 2026-03-13
**Sections enhanced:** All
**Research agents used:** Frontend Design, Pico CSS Docs, Neo-Brutalist Best Practices, Performance Oracle, Security Sentinel, Architecture Strategist, Frontend Races Reviewer, Pattern Recognition, Institutional Learnings

### Key Improvements
1. Use `pico.classless.min.css` instead of full Pico (smaller, matches classless usage)
2. Transition only `transform` (not `box-shadow`) for GPU-composited mobile performance
3. Fix pre-existing `showStatus()` stale-timeout race and add `isSaving` guard against concurrent POSTs
4. Fix pre-existing stored XSS in overlay (`scoreboard.js` `.html()` → `.text()`)
5. Add `user-select: none` and `-webkit-tap-highlight-color: transparent` for clean mobile interaction
6. Pico dark-mode overrides must target both `[data-theme=dark]` AND `@media (prefers-color-scheme: dark)` selectors

### New Considerations Discovered
- `font-variant-numeric: tabular-nums` on score display prevents layout shift on digit changes
- Save button should use a distinct color (not P1 red) to avoid confusion — consider accent green/yellow
- Add `maxlength` to text inputs as defense-in-depth
- Add `aria-label` on stepper buttons (near-zero cost)
- Document Pico version in HTML comment, add to CLAUDE.md dependencies

---

## Overview

Full visual and UX overhaul of `controller.html` — the mobile score entry form used at FGC tournaments. Adds Pico CSS (vendored + CDN fallback) with neo-brutalist overrides, side-by-side P1/P2 layout on wider screens, inline score +/- buttons, restyled game datalist, P1 red / P2 blue color coding, and shadow-shift button press effects.

No changes to the data layer (npoint.io / LAN / localStorage sync logic). JS functions (`save()`, `swap()`, `resetScores()`, `clearAll()`, `getFormData()`, `populateForm()`) are updated only where the score stepper DOM change requires it. `getFormData()` itself needs zero changes.

## Problem Statement / Motivation

The current controller works but has UX pain points for tournament operators:
- Raw number inputs for scores require typing — awkward one-handed on a phone mid-set
- No visual distinction between P1 and P2 sections
- Overall aesthetic is dated (basic dark theme with no design language)

Tournament operators use this form on phones, often one-handed, during live matches. Every tap matters.

## Proposed Solution

### Framework: Pico CSS (Vendored + CDN)

```html
<!-- Pico CSS v2.x.x (classless variant), vendored from jsDelivr -->
<html data-theme="dark">
<head>
  <link rel="stylesheet" href="css/pico.classless.min.css">
</head>
```

- **Use the classless variant** (`pico.classless.min.css`) — smaller than full `pico.min.css` (skips modal, nav, tooltip, progress bar styles this controller never uses). Reduces CSS parse time on mobile.
- **Vendored into repo** at `css/pico.classless.min.css` — LAN mode (`server.py`) serves it locally, no internet required
- Force dark theme via `data-theme="dark"` on `<html>` (not `prefers-color-scheme`, which varies by device)
- Override Pico's CSS custom properties for neo-brutalist look

### Research Insights: Pico CSS v2

**Critical gotcha — dual dark-mode selectors:** Pico v2 defines dark-mode variables in two places: `@media (prefers-color-scheme: dark) { :root:not([data-theme=light]) { ... } }` and `[data-theme=dark] { ... }`. Any CSS variable override for dark mode must appear in **both** selectors, or users reaching dark mode via the other path won't see overrides. Since we force `data-theme="dark"`, our overrides on `[data-theme=dark]` will always apply — but if we also want to override Pico's `--pico-` variables, we should define them on `:root` to catch both paths.

**Key Pico variables to override:**
- `--pico-border-radius: 0` (kill all rounding globally)
- `--pico-border-width: 3px` (thicken all borders)
- `--pico-box-shadow` (replace 6-layer soft shadow with hard offset)
- `--pico-button-box-shadow` / `--pico-button-hover-box-shadow`

**Pico `<section>` styling:** Pico applies specific container styles to `<section>` elements (padding, margin resets). Test that these don't conflict with `.player-section` overrides (background tint, border-left).

**References:**
- https://picocss.com/docs/css-variables — full list of 130+ custom properties
- https://picocss.com/docs/color-schemes — `data-theme` usage
- https://picocss.com/docs/classless — classless variant docs

### Neo-Brutalist Overrides

Keep overrides minimal — only override what Pico gets wrong for the neo-brutalist look. Use literal CSS values instead of a full custom property token system (this is a single file, not a design system).

```css
:root {
  --pico-border-radius: 0;
  --pico-border-width: 3px;
  --pico-box-shadow: 4px 4px 0 #000;
  --pico-button-box-shadow: 4px 4px 0 #000;
  --pico-button-hover-box-shadow: 5px 5px 0 #000;
}
```

Direct styling for colors, borders, and shadows — no intermediate `--bg`, `--surface`, `--accent` tokens. Values are used 1-3 times each; literal values are easier to read and change via find-and-replace in a single file.

### Color Scheme: Dark + Bold Accent

| Element | Value | Usage |
|---------|-------|-------|
| Page background | `#1a1a2e` | Body/html background (kept from current) |
| Section backgrounds | `#222244` | Card/section backgrounds (changed from `#16213e`) |
| Text | `#f0f0f0` | Body text |
| Save button | `#d4ff00` (acid green) or distinct accent | Primary action — **must differ from P1 red** to avoid confusion |
| P1 accent | `#ff4444` | P1 section border + background tint |
| P2 accent | `#4488ff` | P2 section border + background tint |
| Borders | `#000000` | Hard black, 3px |
| Shadows | `4px 4px 0 #000` | Hard offset, no blur |

**Note:** The accent color changes from `#c40a18` (current) to `#ff4444` (P1) and the section background changes from `#16213e` to `#222244`. These are intentional palette shifts for the neo-brutalist aesthetic.

### Research Insights: Neo-Brutalist Design

**Core principles (from NN/g, neobrutalism.dev, industry sources):**
- Restrict to 2-3 bold, high-contrast colors. Neo-brutalism rejects pastels.
- Hard-offset, solid-color shadows — never blurred, never multi-layered.
- No border-radius anywhere — the single most important rule for credibility.
- Bold typography hierarchy: headings ~2x body size, uppercase labels with letter-spacing.

**Button interaction — the signature pattern:**
- **Resting:** solid shadow, element appears to float
- **Hover (desktop):** element lifts slightly (`translate(-2px, -2px)`), shadow increases — ~200ms transition
- **Active/pressed:** element snaps flat (`translate(4px, 4px)`), shadow to 0 — very fast (~80ms)
- Speed differential (slow lift, instant press) is critical for the physical feel

**Typography recommendations:**
- Score numbers: minimum 48px, use `font-variant-numeric: tabular-nums` to prevent layout shift on digit changes
- Labels: uppercase, `letter-spacing: 0.1em`, smaller size
- Buttons: uppercase, bold, letter-spaced

**Mobile patterns:**
- Minimum 48px touch targets (Material Design guideline)
- 24-32px card padding for breathing room
- `user-select: none` on interactive elements to prevent accidental text selection during rapid tapping
- `-webkit-tap-highlight-color: transparent` for clean press behavior (we provide our own feedback via shadow-shift)

**References:**
- https://www.nngroup.com/articles/neobrutalism/
- https://www.neobrutalism.dev/
- https://www.joshwcomeau.com/animation/3d-button/

## Technical Approach

### Phase 1: Pico CSS + Visual Reskin + Layout

Add vendored Pico CSS (classless variant), replace the existing inline `<style>` block with neo-brutalist overrides, restructure HTML for the grid layout, add P1/P2 color coding, and apply button styling.

**Files:**
- `controller.html` — HTML restructure + inline `<style>` rewrite
- `css/pico.classless.min.css` — new vendored file (download from CDN, verify hash)
- `CLAUDE.md` — add Pico CSS to vendored dependencies list

**Breaking change:** The current `max-width: 480px` on `body` must be removed/widened to allow side-by-side layout on tablets. On phones (< 768px), the stacked layout is preserved — this is the primary use case.

#### HTML Structure

```html
<!-- Pico CSS v2.x.x (classless), vendored from jsDelivr -->
<html data-theme="dark">
<head>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="css/pico.classless.min.css">
  <style>/* neo-brutalist overrides */</style>
</head>
<body>
  <h1>FGC Scoreboard</h1>
  <div id="status"></div>
  <div id="no-bin-warning"></div>

  <div class="players-grid">
    <section class="player-section p1">
      <h2>Player 1</h2>
      <!-- name, team, score stepper -->
    </section>
    <section class="player-section p2">
      <h2>Player 2</h2>
      <!-- name, team, score stepper -->
    </section>
  </div>

  <section class="match-section">
    <h2>Match Info</h2>
    <!-- round input, game input (datalist) -->
  </section>

  <div class="button-row">
    <!-- Save, Swap, Reset, Clear -->
  </div>
</body>
```

**Note:** Section headings use `<h2>` (not `<h3>`) to maintain proper document outline under the `<h1>` title.

#### Layout CSS

```css
.players-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}

@media (min-width: 768px) {
  .players-grid {
    grid-template-columns: 1fr 1fr;
  }
}
```

Note: The side-by-side layout is a nice-to-have for tablet/desktop. The stacked mobile layout is the core experience — phones are the primary use case.

#### P1/P2 Color Coding

Use `rgba()` for background tints (universal browser support, unlike `color-mix()`):

```css
.player-section.p1 {
  border-left: 5px solid #ff4444;
  background: rgba(255, 68, 68, 0.05);
}

.player-section.p2 {
  border-left: 5px solid #4488ff;
  background: rgba(68, 136, 255, 0.05);
}
```

#### Button Styling + Shadow-Shift Press Effect

```css
button {
  border: 3px solid #000;
  box-shadow: 4px 4px 0 #000;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 12px 20px;
  cursor: pointer;
  transition: transform 0.1s; /* only transform — box-shadow repaints are expensive on mobile */
  min-height: 48px; /* 48px per Material Design touch target guidelines */
  touch-action: manipulation; /* prevent double-tap zoom on mobile Safari */
  user-select: none; /* prevent text selection during rapid tapping */
  -webkit-tap-highlight-color: transparent; /* we provide our own press feedback */
}

button:active {
  transform: translate(4px, 4px);
  box-shadow: 0 0 0 #000; /* snaps instantly (no transition on box-shadow) */
}
```

### Research Insights: Performance

**Transition only `transform`, not `box-shadow`.** On low-end Android phones (common at local FGC events), `box-shadow` transitions trigger paint operations on every frame. The `transform` portion is GPU-composited and cheap. Remove `box-shadow` from the transition — the shadow snaps while the element moves, and the 0.1s duration is short enough that users won't notice.

**`pico.classless.min.css` vs `pico.min.css`.** The full variant includes styles for modals, navs, tooltips, progress bars — none of which this controller uses. The classless variant is smaller and reduces CSS parse time (CSS parsing is synchronous and blocks first paint on mobile).

**`will-change: transform` (optional).** Could be added to buttons to pre-promote them to their own compositor layer. With only ~6 buttons on the page, the GPU memory overhead is acceptable. Modern browsers often handle this automatically.

**Button hierarchy:**
- **Save** — distinct accent color (not P1 red), most prominent, consider taller (56px)
- **Swap** — secondary style, distinct from Save
- **Reset / Clear** — subdued/tertiary, Clear is least prominent to reduce accidental taps

#### Status Indicator + Polish

Status indicator (`#status`) spans full width above the players grid. Mode warning (`#no-bin-warning`) styled consistently with thick borders.

**Acceptance criteria:**
- [x] Vendored `css/pico.classless.min.css` loads locally (verify download hash against official release)
- [x] Pico version documented in HTML comment at top of `controller.html`
- [x] Pico CSS added to CLAUDE.md vendored dependencies section
- [x] Dark theme forced via `data-theme="dark"`
- [x] Pico defaults overridden: zero border-radius, 3px borders, hard shadows
- [x] `max-width: 480px` removed from body; layout now responsive
- [x] P1/P2 stack on mobile (< 768px), side-by-side on tablet+ (>= 768px)
- [x] P1 has red left border + faint red background tint (`rgba`)
- [x] P2 has blue left border + faint blue background tint (`rgba`)
- [x] Section headings are `<h2>` (proper outline under `<h1>` title)
- [x] All buttons have shadow-shift press effect (transition on `transform` only, not `box-shadow`)
- [x] `touch-action: manipulation` preserved on all buttons
- [x] `user-select: none` on all buttons and stepper controls
- [x] `-webkit-tap-highlight-color: transparent` on interactive elements
- [x] Save button visually distinct from P1 red; Clear button visually subdued
- [x] All buttons min-height 48px (Material Design touch target)
- [x] Status indicator styled with thick border, appropriate colors (green/red/yellow)
- [x] Round field and game field sit in a match-info section below the players grid
- [x] Page title styled with bold typography
- [x] Text inputs have `maxlength` attributes (names: 64, teams: 32, round: 64, game: 32)

### Phase 2: Score Stepper + Game Selector + Bug Fixes

Replace raw `<input type="number">` with +/- stepper buttons. Restyle the game datalist. Fix pre-existing race conditions in `showStatus()` and `save()`.

#### Score +/- Buttons

Use `<input type="text" readonly>` as the score display — it serves as both the visible display AND the value source. This means `getFormData()` needs zero changes (it still reads `#p1Score.value`), and `populateForm()`, `swap()`, `resetScores()`, `clearAll()` only need to set `.value` as they do today.

```html
<div class="score-stepper">
  <button type="button" class="score-btn" onclick="adjustScore('p1Score', -1)"
          tabindex="-1" aria-label="Decrease P1 score">-</button>
  <input type="text" readonly id="p1Score" value="0" class="score-display">
  <button type="button" class="score-btn" onclick="adjustScore('p1Score', 1)"
          tabindex="-1" aria-label="Increase P1 score">+</button>
</div>
```

**Score display styling:**
```css
.score-display {
  font-size: 48px;
  font-weight: 700;
  text-align: center;
  font-variant-numeric: tabular-nums; /* prevent layout shift on digit changes */
  user-select: none; /* suppress iOS copy/paste callout on readonly input */
}
```

**New JS function:**

```javascript
function adjustScore(id, delta) {
  var input = document.getElementById(id);
  var val = parseInt(input.value, 10) || 0;
  val = Math.max(0, Math.min(99, val + delta));
  input.value = String(val);
}
```

**Key constraints:**
- Scores clamped at 0 (floor) and 99 (ceiling)
- `<input type="text" readonly>` — `.value` works exactly like before, no hidden input or display span needed
- `readonly` preserves `.value` in form data (unlike `disabled` which excludes it)
- `tabindex="-1"` on +/- buttons prevents focus steal / keyboard popup on mobile
- `aria-label` on +/- buttons for screen reader accessibility (near-zero cost)
- No debouncing — each tap increments immediately, synchronous read/write means no race condition
- No long-press auto-repeat (unnecessary for single-digit scores)
- `font-variant-numeric: tabular-nums` prevents score display from shifting width as digits change

**JS function impact:**
- `getFormData()` — **no change** (still reads `#p1Score.value`)
- `populateForm()` — **no change** (still sets `#p1Score.value`)
- `swap()` — **no change** (still reads/writes `.value`)
- `resetScores()` — **no change** (still sets `.value = '0'`)
- `clearAll()` — **no change** (still sets `.value = '0'`)

**Defense-in-depth:** Add output-side validation in `getFormData()` for scores, since `readonly` is a UI hint, not a security boundary:
```javascript
// In getFormData(), clamp scores before sending:
p1Score: String(Math.max(0, Math.min(99, parseInt(document.getElementById('p1Score').value, 10) || 0)))
```

#### Game Selector — Restyled Datalist (Keep Current Approach)

Keep the existing `<input type="text" list="games">` + `<datalist>` pattern. This already supports arbitrary game names with zero extra JS. Just restyle the input with neo-brutalist borders and shadows.

The current datalist already lists 12 games. No `<select>` conversion needed — the datalist allows both picking from the list and typing custom game names (e.g., "GGST", "TEKKEN8"). This avoids the complexity of a `<select>` + "Other..." fallback + `__other__` sentinel value + conditional logic in `getFormData()` and `populateForm()`.

#### Bug Fix: `showStatus()` Stale Timeout Race

**Pre-existing bug:** If the operator taps Save, sees "Saved" (2s hide timeout queued), then taps Save again quickly, the previous timeout fires and hides the status indicator while the second save is still in flight. The operator sees no feedback.

**Fix:** Store the timeout ID and clear it before setting a new one:

```javascript
var statusTimeoutId = null;
function showStatus(msg, type) {
  var el = document.getElementById('status');
  if (statusTimeoutId !== null) {
    clearTimeout(statusTimeoutId);
    statusTimeoutId = null;
  }
  el.textContent = msg;
  el.className = 'status-' + type;
  el.style.display = 'block';
  if (type === 'ok') {
    statusTimeoutId = setTimeout(function() {
      el.style.display = 'none';
      statusTimeoutId = null;
    }, 2000);
  }
}
```

#### Bug Fix: Concurrent `save()` Race Condition

**Pre-existing bug:** Nothing prevents rapid Save taps from firing concurrent `fetch()` POSTs that race to the server and arrive out of order, causing score flickering on the overlay.

**Fix:** Add a simple `isSaving` guard:

```javascript
var isSaving = false;
function save() {
  if (isSaving) return;
  isSaving = true;
  // ... existing fetch logic ...
  fetch(url, { ... })
    .then(...)
    .catch(...)
    .finally(function() { isSaving = false; });
}
```

The new +/- stepper makes rapid saves more likely (operator taps score then immediately Save), so this fix is timely.

**Acceptance criteria:**
- [x] +/- buttons visible and tappable for both P1 and P2 scores
- [x] +/- buttons have `aria-label` attributes for accessibility
- [x] Score increments/decrements on tap, clamped 0-99
- [x] Score display: large (48px+), centered, bold, `tabular-nums`
- [x] Score display has `user-select: none` (suppresses iOS copy/paste callout)
- [x] `getFormData()` still returns scores as strings (no code change needed)
- [x] `getFormData()` includes defense-in-depth clamp on score values
- [x] `swap()`, `resetScores()`, `clearAll()`, `populateForm()` all work unchanged
- [x] +/- buttons don't steal focus (no keyboard popup on mobile)
- [x] Min touch target 48x48px on +/- buttons
- [x] Scores populate correctly on page load from all three modes (npoint/LAN/localStorage)
- [x] Game input restyled with neo-brutalist treatment (thick border, hard shadow)
- [x] Game datalist still allows custom game names
- [x] `showStatus()` stale-timeout race fixed (clearTimeout before new status)
- [x] Concurrent `save()` calls guarded with `isSaving` flag

### Phase 3 (Recommended): Fix Overlay XSS

**Pre-existing critical vulnerability:** The overlay renders all user-supplied data using `$('#p1Name').html(p1Name)` in `scoreboard.js`. The `.html()` method interprets HTML, meaning any value containing `<script>` tags or event handlers will execute as code in the overlay's browser context.

**Fix:** Change all `.html()` calls to `.text()` in `_overlays/js/scoreboard.js`. The overlay only displays plain text — there is no reason to interpret HTML.

**Files:** `_overlays/js/scoreboard.js`

This is out of scope for the controller redesign proper, but the controller produces the data the overlay consumes. Fixing it here prevents a stored XSS attack where a malicious player name injected via the controller executes in the OBS browser source.

**Acceptance criteria:**
- [x] All `.html()` calls for user data in `scoreboard.js` changed to `.text()`
- [x] Overlay still renders player names, scores, round, and game correctly

## Dependencies & Risks

| Risk | Mitigation |
|------|------------|
| Pico CSS unavailable | Vendored locally — no CDN dependency. `server.py` serves `css/pico.classless.min.css` from CWD. Works in all three modes (LAN, GitHub Pages, file://). |
| Score string coercion breaks | `<input type="text" readonly>` has `.value` — `getFormData()` unchanged. Defense-in-depth clamp added. |
| Pico `<section>` styling conflicts | Test Pico's default padding/margin on `<section>` elements against `.player-section` overrides during implementation. |
| Swap/Reset/Clear broken by DOM changes | Score stepper uses readonly `<input>` — these functions need zero changes. Test each anyway. |
| Side-by-side layout too narrow on phones | 768px breakpoint ensures phones always get stacked layout. |
| `touch-action: manipulation` lost | Explicitly preserved in button CSS. Prevents double-tap zoom on mobile Safari. |
| Palette shift surprises | Documented: accent `#c40a18` → `#ff4444`, sections `#16213e` → `#222244`. |

## Known Gaps

- **`populateForm()` overwrites in-progress edits:** Pre-existing — if operator types before the page-load fetch completes, `populateForm()` overwrites their input. A `userHasInteracted` flag would fix this but is out of scope for this redesign.
- **CSP incompatibility:** Inline `onclick` handlers and `<style>` block require `'unsafe-inline'`. If `server.py` ever adds CSP headers, these would need to move to external files. Not actionable now.
- **`bin` parameter unsanitized:** Pre-existing — the `?bin=` URL parameter is concatenated directly into the API URL. Low risk (npoint.io would 404 on invalid IDs) but worth noting.

## Success Metrics

- Tournament operator can complete a full set (name entry, score tracking, game/round selection, swap sides) on a phone one-handed
- No regressions in data sent to overlay (scores as strings, full JSON POST)
- Controller loads and works in all three modes (npoint.io, LAN, localStorage)
- Touch targets >= 48px on all interactive elements
- No concurrent-save race conditions (isSaving guard)
- No stale status indicator (timeout properly cleared)

## Implementation Checklist

Before starting:
- [ ] Download `pico.classless.min.css` from CDN and verify hash against official release
- [ ] Add `css/pico.classless.min.css linguist-vendored` to `.gitattributes`

After implementation:
- [ ] Test all three sync modes (npoint.io, LAN via `server.py`, localStorage via `file://`)
- [ ] Test on a real phone (score stepper rapid tapping, Save button, swap sides)
- [ ] Test Pico's `<section>` default styling doesn't conflict with overrides
- [ ] Verify score string coercion end-to-end (controller → JSON → overlay comparison)

## References & Research

### Internal References
- Brainstorm: `docs/brainstorms/2026-03-13-controller-redesign-brainstorm.md`
- Current controller: `controller.html` (383 lines, all inline)
- Overlay score comparison logic: `_overlays/js/scoreboard.js` (string comparison on scores)
- StreamControl migration learnings: `docs/solutions/integration-issues/streamcontrol-to-npoint-remote-controller.md`

### External References
- Pico CSS docs: https://picocss.com/docs
- Pico CSS variables: https://picocss.com/docs/css-variables
- Pico CSS classless: https://picocss.com/docs/classless
- Pico CDN (for vendoring): `https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.classless.min.css`
- Neo-brutalist principles: https://www.nngroup.com/articles/neobrutalism/
- Neo-brutalist components: https://www.neobrutalism.dev/
- 3D button technique: https://www.joshwcomeau.com/animation/3d-button/
