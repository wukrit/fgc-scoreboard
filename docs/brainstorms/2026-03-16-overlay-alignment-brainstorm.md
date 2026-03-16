# Brainstorm: CSS Overlay Alignment Fix

**Date:** 2026-03-16
**Status:** Resolved

## What We're Building

Fix CSS alignment issues in the FGC Scoreboard overlay to ensure P1 and P2 sides are symmetrically aligned and all elements display properly in OBS.

## Why This Approach

The current overlay has multiple alignment asymmetries between the P1 (left) and P2 (right) sides. These cause visual inconsistencies when displayed in OBS, particularly:
1. Score-to-name spacing differs by 9px between sides
2. Background wrappers are off-center by 5px
3. Player name wrappers are 8px asymmetric from center

## Key Decisions

### Identified Asymmetries (Current State)

| Element | P1 Position | P2 Position | Asymmetry |
|---------|-------------|-------------|-----------|
| Score-to-name gap | 12px | 21px | **9px** |
| BGWrapper from edge | 290px left | 285px right | **5px** |
| Wrapper from center | 264px | 256px | **8px** |
| Score from center | 644.5px | 645.5px | **1px (OK)** |
| ScoreBG from edge | 290px | 291px | **1px (OK)** |

### Root Cause

The CSS uses hardcoded pixel values that weren't calculated for perfect symmetry around the 1920px center point (960px).

### Proposed Fix Values

To achieve perfect symmetry:

1. **rightBGWrapper**: Change from `1205px` to `1200px` (290px from each edge)
2. **p2Wrapper**: Change from `1216px` to `1212px` (matches P1 at 348px from center)
3. **p2Score**: Change from `1585px` to `1580px` (matches P1 at 295px from edge)

This ensures:
- Score-to-name gap becomes 12px on both sides
- Background wrappers are equidistant from edges
- All elements centered around 960px

## Open Questions

- [ ] Should the SCSS be modified directly, or just the compiled CSS?
- [ ] Should animation keyframes also be adjusted for symmetry?
- [ ] Do you want to test the changes before committing?

## Next Steps

1. Apply CSS fixes to achieve symmetric layout
2. Recompile SCSS if needed
3. Test in OBS/browser to verify alignment
