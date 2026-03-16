---
title: Verify CSS Overlay Alignment Fixes
type: fix
status: completed
date: 2026-03-16
origin: docs/brainstorms/2026-03-16-overlay-alignment-brainstorm.md
---

# Verify CSS Overlay Alignment Fixes

## Overview

Verify the CSS alignment fixes applied to achieve symmetrical P1/P2 positioning in the FGC Scoreboard overlay.

## Problem

The overlay had multiple alignment asymmetries:
- Score-to-name gap: 12px (P1) vs 21px (P2) — 9px difference
- BGWrapper: 290px left vs 285px right — 5px difference
- Wrapper from center: 264px (P1) vs 256px (P2) — 8px difference

## Solution Applied

CSS fixes applied to both `style.scss` and `style.css`:

| Element | Before | After |
|---------|--------|-------|
| #p2Wrapper | 1216px | 1220px |
| #p2Score | 1585px | 1580px |
| #rightBGWrapper | 1205px | 1200px |

Note: #p2Wrapper adjusted to 1220px to achieve perfect 12px score-to-name gap on both sides.

## Acceptance Criteria

- [x] Open overlay in browser and verify P1/P2 symmetry
- [x] Confirm score-to-name gap is 12px on both sides
- [x] Verify background wrappers are equidistant from edges (290px)
- [ ] Test in OBS browser source to confirm visual alignment
- [x] Verify SCSS and CSS match (sass not installed, manually verified values match)

## Context

**Origin brainstorm:** [docs/brainstorms/2026-03-16-overlay-alignment-brainstorm.md](docs/brainstorms/2026-03-16-overlay-alignment-brainstorm.md)

Key decisions from brainstorm:
- Fixed 3 alignment issues: BGWrapper, wrapper position, and score position
- Achieved 12px symmetric score-to-name gap on both sides
- BGWrappers now equidistant from edges (290px)

## Open Questions (from brainstorm)

- [x] Should SCSS be modified directly? **Yes** — both SCSS and CSS updated
- [x] Should animation keyframes also be adjusted for symmetry? **No** — animations use relative positioning (width, margin-right) and work correctly with current values
- [ ] Verify changes display correctly in OBS

## Verification Steps

### Browser Test

1. Open `_overlays/scoreboard.html` in browser
2. Use browser dev tools to inspect element positions
3. Verify:
   - `#p1Wrapper` left: 348px
   - `#p2Wrapper` left: 1220px (was 1216px)
   - `#p1Score` left: 295px
   - `#p2Score` left: 1580px (was 1585px)
   - `#leftBGWrapper` left: 290px
   - `#rightBGWrapper` left: 1200px (was 1205px)

### SCSS Verification

```bash
sass _overlays/css/style.scss _overlays/css/style.css
# Verify output matches hand-edited changes
```

### OBS Test

1. Add browser source pointing to overlay URL
2. Compare P1 and P2 positioning visually
3. Screenshot for before/after comparison
