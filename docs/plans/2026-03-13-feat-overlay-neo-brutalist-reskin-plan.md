---
title: "feat: Overlay Neo-Brutalist Reskin"
type: feat
date: 2026-03-13
---

# feat: Overlay Neo-Brutalist Reskin

## Overview

Reskin the scoreboard overlay (`_overlays/css/style.scss`) to match the neo-brutalist design language established in the controller redesign. Update colors, borders, shadows, and font while preserving all sizing, spacing, positioning, and animation behavior.

No JS changes. No layout changes. No animation timing changes.

## Proposed Solution

### SCSS Variable Updates

```scss
// OLD
$main-color: rgba(0, 0, 0, 0.75);
$accent-color: #c40a18;
$font-color: white;
$team-color: #e5e5e5;

// NEW
$main-color: rgba(0, 0, 0, 0.85);
$p1-accent: #ff4444;
$p2-accent: #4488ff;
$font-color: white;
$team-color: #e5e5e5;
```

Remove the single `$accent-color` and replace with `$p1-accent` / `$p2-accent` for P1 red / P2 blue color coding.

### Font: Archivo Black (Vendored)

Download Archivo Black from Google Fonts, vendor at `_overlays/fonts/ArchivoBlack-Regular.ttf`.

```scss
@font-face {
  src: url('../fonts/ArchivoBlack-Regular.ttf');
  font-family: "Archivo Black";
}

body {
  font-family: "Archivo Black", "Arial Black", sans-serif;
  letter-spacing: 0.05em;
}
```

Keep existing `text-transform: uppercase`, `text-shadow`, and `-webkit-text-stroke` on scores.

### Borders: 1px → 3px

All `outline: 1px solid` becomes `outline: 3px solid`:

```scss
// Player BG boxes
.playerBG {
  outline: 3px solid $p1-accent; // P1 default, P2 overridden below
}

// Score BG boxes
.scoreBG {
  outline: 3px solid black;
}

// Round BG
#roundBG {
  outline: 3px solid $p1-accent; // or neutral color
}
```

### P1/P2 Color Differentiation

P1 elements use `$p1-accent` (#ff4444), P2 elements use `$p2-accent` (#4488ff):

```scss
// Player backgrounds — outlines
#p1PlayerBG {
  outline-color: $p1-accent;
}
#p2PlayerBG {
  outline-color: $p2-accent;
}

// Score backgrounds — fill color
#p1ScoreBG {
  background-color: $p1-accent;
}
#p2ScoreBG {
  background-color: $p2-accent;
}
```

### Shadows: Blurred → Hard Offset

Replace `filter: drop-shadow(0 6px 10px rgba(0, 0, 0, 0.6))` with hard offset `filter: drop-shadow(3px 3px 0 rgba(0, 0, 0, 0.8))`.

Must use `filter: drop-shadow()` (not `box-shadow`) because overlay elements use `transform: skewX()` — `box-shadow` doesn't respect CSS transforms.

```scss
.playerBG {
  filter: drop-shadow(3px 3px 0 rgba(0, 0, 0, 0.8));
}
.scoreBG {
  filter: drop-shadow(3px 3px 0 rgba(0, 0, 0, 0.8));
}
#roundBG {
  filter: drop-shadow(3px 3px 0 rgba(0, 0, 0, 0.8));
}
```

### Background Opacity

Increase from `rgba(0, 0, 0, 0.75)` to `rgba(0, 0, 0, 0.85)` for a more solid, opaque neo-brutalist feel. The current semi-transparency lets game visuals bleed through too much.

### Recompile SCSS → CSS

```bash
sass _overlays/css/style.scss _overlays/css/style.css
```

## Files Changed

- `_overlays/css/style.scss` — all style changes
- `_overlays/css/style.css` — recompiled output
- `_overlays/fonts/ArchivoBlack-Regular.ttf` — new vendored font
- `CLAUDE.md` — add Archivo Black to vendored dependencies

## Acceptance Criteria

- [x] SCSS variables updated: `$accent-color` split into `$p1-accent` / `$p2-accent`
- [x] Background opacity increased to 0.85
- [x] Archivo Black font vendored and loaded (fallback to Arial Black)
- [x] All borders thickened from 1px to 3px
- [x] P1 elements use red accent (#ff4444) — outline and score background
- [x] P2 elements use blue accent (#4488ff) — outline and score background
- [x] All blurred drop-shadows replaced with hard offset (3px 3px 0)
- [x] SCSS compiled to CSS
- [ ] Overlay loads correctly in browser (all three sync modes)
- [ ] Animations still play correctly (slide-in, slide-down)
- [ ] Game-specific position adjustments still work
- [x] No sizing or spacing changes from current overlay
- [x] Archivo Black added to CLAUDE.md vendored dependencies

## Dependencies & Risks

| Risk | Mitigation |
|------|------------|
| Hard shadows look muddy on stream compression | Test with OBS recording; can increase offset or adjust opacity |
| Archivo Black too wide/tall for existing text boxes | Keep same font-size; Archivo Black is condensed, should fit. Test with long names. |
| P2 blue hard to read on certain game backgrounds | Test against common FGC game backgrounds; adjust blue shade if needed |
| SCSS compiler not available | Any Sass compiler works (`sass`, `node-sass`, `dart-sass`). Document command in plan. |

## References

- Brainstorm: `docs/brainstorms/2026-03-13-overlay-neo-brutalist-brainstorm.md`
- Controller redesign (reference implementation): `controller.html`
- Controller solution doc: `docs/solutions/ui-redesign/controller-neo-brutalist-pico-css-redesign.md`
- Current overlay SCSS: `_overlays/css/style.scss`
- Archivo Black on Google Fonts: https://fonts.google.com/specimen/Archivo+Black
