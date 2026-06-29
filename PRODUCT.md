# Product

## Register

product

## Users

Tournament operators, stream techs, and bracket runners at live fighting game events. They use phones or tablets—often one-handed, in dim venue lighting, while matches are in progress. They need to update scores in seconds without breaking focus on the stream.

## Product Purpose

FGC Scoreboard is a lightweight scoreboard controller that syncs player names, scores, round, game, and optional counters to an OBS overlay. Success means zero friction during a set: tap to score, save when metadata changes, and trust that the overlay updates immediately.

## Brand Personality

Fast, reliable, no-nonsense. Tournament-floor practical—not flashy marketing, not enterprise SaaS. The overlay keeps the bold stream look; the controller stays out of the way.

## Anti-references

- Neo-brutalist drop shadows and heavy outlines on every control (visual noise under pressure)
- Side-stripe accent borders on cards (AI slop tell)
- Purple gradients, glassmorphism, decorative motion
- Dense dashboards with metrics nobody asked for
- Modal-first flows for simple field edits
- Uppercase-everything typography that slows scanning

## Design Principles

1. **Scores first** — The primary task (increment/decrement) gets the largest touch targets and least friction.
2. **Color identifies players** — Muted P1/P2 accents on steppers and headers; neutral card surfaces. The overlay keeps its own bold red/blue.
3. **Familiar controls** — Standard form patterns operators already know from other tools; no invented affordances.
4. **Progressive disclosure** — Counters and destructive actions stay tucked away until needed.
5. **Glanceable status** — Connection and save state visible without stealing attention from the match.

## Accessibility & Inclusion

- WCAG 2.1 AA contrast for text and interactive controls
- Minimum 48px touch targets on mobile
- Visible focus rings for keyboard use
- `prefers-reduced-motion` respected for all transitions
- Screen reader labels on score steppers and status updates
