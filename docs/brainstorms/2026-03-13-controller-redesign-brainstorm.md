# Controller Page Redesign — Neo-Brutalist UX Overhaul

**Date:** 2026-03-13
**Status:** Ready for planning

## What We're Building

A full visual and UX overhaul of `controller.html` — the mobile-friendly score entry form used at FGC tournaments. The goal is a modern neo-brutalist aesthetic with improved ergonomics for tournament operators using phones/tablets.

## Why This Approach

- **Pico CSS via CDN** — classless CSS framework (~10KB), zero install, zero build tools. Matches the project's no-build philosophy. Semantic HTML "just works."
- **Neo-brutalist custom overrides** — thick borders, hard shadows, sharp corners, bold accent colors layered on top of Pico's defaults.
- **Dark + bold accent color scheme** — keeps visual cohesion with the existing overlay's dark theme. Red accent for energy.
- **Full UX overhaul** — the current form works but doesn't leverage modern layout or mobile-first patterns. Tournament operators use this on phones mid-set; every tap matters.

## Key Decisions

1. **Library: Pico CSS via CDN** — no install, no build step, just a `<link>` tag. Override Pico's CSS custom properties for neo-brutalist look (zero border-radius, thick borders, hard shadows).

2. **Color scheme: Dark + bold accent**
   - Background: `#1a1a2e` (dark navy, carried over)
   - Surface: `#222244` (card backgrounds)
   - Text: `#f0f0f0` (white)
   - Accent: `#ff4444` (bold red)
   - Borders: `#000000` (hard black)
   - Shadows: solid offset shadows using accent or black

3. **Layout: Side-by-side P1/P2** — on wider screens, P1 and P2 inputs sit side by side. On mobile, they stack but are visually distinct (color-coded or labeled).

4. **Score controls: Inline +/- buttons** — instead of raw number inputs, add tap-friendly increment/decrement buttons flanking the score display. Critical for one-handed phone use during tournaments.

5. **Game selector: Styled `<select>` dropdown** — replace the text input + datalist with a proper `<select>` element, styled with thick borders and hard shadow to match the neo-brutalist theme. Simpler than chips, takes less vertical space.

6. **Mobile ergonomics** — larger touch targets (min 44px), better spacing, thumb-reachable action buttons.

## Scope

### In scope
- Restyle all form elements with Pico CSS + neo-brutalist overrides
- Rethink form layout (side-by-side players, grouped controls)
- Add inline score +/- buttons
- Improve game selector UX
- Responsive design polish for phone/tablet
- Larger touch targets throughout

### Out of scope
- No changes to data layer (npoint.io / LAN / localStorage logic)
- No changes to scoreboard overlay
- No new features (roster management, bracket integration, etc.)
- No build tools or package manager

7. **Player color-coding: P1 red / P2 blue** — subtle color tints on section borders or backgrounds to match FGC conventions. Helps tournament operators instantly identify which side they're editing.

8. **Button press effect: Shadow-shift** — on click/tap, buttons translate down-right and shadow shrinks to zero, simulating a physical button press. Classic neo-brutalist interaction.

## Open Questions

- Exact shade of P1 red vs P2 blue tints on the dark background — needs visual testing
- Should the score +/- buttons wrap around or clamp (e.g., can score go negative?)

## Technical Notes

- Pico CSS CDN: `https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css`
- All styling stays inline in `controller.html` (no separate CSS file for the controller today)
- JS logic for save/load/swap/reset remains unchanged — only the DOM structure and styles change
- Scores must continue to be sent as strings (`String(input.value)`)
