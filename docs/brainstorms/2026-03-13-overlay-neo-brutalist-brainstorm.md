# Overlay Neo-Brutalist Redesign — Style Alignment with Controller

**Date:** 2026-03-13
**Status:** Ready for planning

## What We're Building

A visual refresh of the scoreboard overlay (`_overlays/css/style.scss` + `scoreboard.html`) to match the neo-brutalist design language established in the controller redesign. The goal is style consistency between what the operator sees (controller) and what viewers see (overlay) — same design DNA, adapted for broadcast.

**Constraint:** Keep sizing, spacing, and animation timing/behavior essentially unchanged. This is a reskin, not a layout overhaul.

## Why This Approach

- The controller now has a distinct neo-brutalist identity (thick borders, hard shadows, bold colors, P1 red / P2 blue)
- The overlay should echo that language so the product feels cohesive
- The overlay already has sharp edges and skew transforms — it's partway there
- Minimal sizing/spacing changes reduce risk of breaking game-specific positioning adjustments

## Key Decisions

1. **Color palette alignment:**
   - Accent: `#c40a18` → `#ff4444` (P1 red) and new `#4488ff` (P2 blue)
   - Backgrounds: more opaque (current `rgba(0,0,0,0.75)` may get adjusted)
   - Text: keep white, keep text-stroke on scores

2. **P1 red / P2 blue color coding:**
   - P1 score box and name area use `#ff4444` accent
   - P2 score box and name area use `#4488ff` accent
   - Matches what the controller operator sees — visual continuity

3. **Font: Archivo Black (Google Fonts, vendored):**
   - Replace Valorant font with Archivo Black — bold, condensed, loud
   - Vendor the font file locally (same pattern as Pico CSS — LAN tournaments have no internet)
   - Keep `text-transform: uppercase` and letter-spacing

4. **Borders: 1px → 3px solid:**
   - Thicken all borders to 3px to match controller's neo-brutalist weight
   - Keep accent-color borders on player/round boxes

5. **Shadows: blurred → hard offset:**
   - Replace `drop-shadow(0 6px 10px rgba(0,0,0,0.6))` with hard offset (`drop-shadow(3px 3px 0 #000)`)
   - Uses CSS `filter: drop-shadow()` since overlay elements use skew transforms (box-shadow doesn't work well with transforms)

6. **Keep skew transforms:** The angular skew on P1/P2 elements is a strong gaming aesthetic — keep it.

7. **Keep all sizing/spacing:** Player box widths, score box sizes, round box dimensions, and animation distances stay the same to preserve game-specific positioning adjustments.

## Scope

### In scope
- Update SCSS color variables (`$main-color`, `$accent-color`, `$font-color`, `$team-color`)
- Add P1/P2 color differentiation
- Replace Valorant font with vendored Archivo Black
- Thicken borders to 3px
- Replace blurred shadows with hard offset shadows
- Recompile SCSS → CSS

### Out of scope
- No changes to scoreboard.js animation logic
- No changes to element sizing or positioning
- No changes to game-specific adjustment groups
- No changes to data flow or polling

## Open Questions

- Exact P2 blue shade on stream — may need testing against common game backgrounds
- Whether the round box should have its own color treatment or stay neutral
- Whether the 3D perspective tilt on the round box conflicts with the flat neo-brutalist aesthetic
