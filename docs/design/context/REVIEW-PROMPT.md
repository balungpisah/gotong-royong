# Design Completeness Review — Gotong Royong

## Your Role

You are reviewing the design system for **Gotong Royong**, a witness-first community coordination platform built for Indonesian neighborhoods (RT/RW level). Your job is to assess **completeness** — are there gaps, missing flows, undefined edge cases, or contradictions that would block a development team from building this?

## What This Platform Does (30-second summary)

Residents observe things in their community — problems, ideas, questions, celebrations, proposals — and submit them through an AI-powered conversational triage (AI-00). The AI routes their input to one of three modes:

1. **Komunitas** — collaborative community action via 5 tracks (Tuntaskan/Wujudkan/Telusuri/Rayakan/Musyawarah), each with its own lifecycle
2. **Catatan Saksi** — private encrypted witness vault (for sensitive personal records)
3. **Siaga** — emergency broadcast (speed-first, one-tap)

A reputation engine called **Tandang** tracks contributions across 3 axes (Integrity/Competence/Judgment) with 5 credit types (A–E) and 5 tiers. Privacy is handled via a 4-level **Rahasia** system (L0 Terbuka → L3 Sangat Rahasia). 10 AI touch points (AI-00 through AI-09) support the platform with a strict "Suggest-Don't-Overwrite" principle.

## The Codebase You're Reviewing

All files are in the `docs/` folder. Start with **DESIGN-CONTEXT.md** — it's the master reference.

### Key Files (read in this order)

| Priority | File | What It Is |
|---|---|---|
| 1 | `DESIGN-CONTEXT.md` | **START HERE.** Master handoff doc with all terminology, locked decisions, design tokens, file map |
| 2 | `DESIGN-SEQUENCE.md` | Checklist of all 22 design steps with status and descriptions |
| 3 | `DESIGN-DNA-v0.1.md` | Formal design system spec (philosophy, tokens, components, patterns, architecture) |
| 4 | `AI-SPEC-v0.2.md` | AI layer spec — 10 touch points, model selection, guardrails, decision log |
| 5 | `UI-UX-SPEC-v0.5.md` | UI/UX spec — 29 sections covering every screen and interaction |

### HTML Prototypes (24 files, browser-viewable)

These are interactive reference prototypes — open in a browser to see live-rendered components.

| File | Contents |
|---|---|
| `A1-mood-vibe.html` | Design mood exploration (Tanah selected) |
| `A2-color-palette.html` | Full color token set with WCAG AA verification |
| `A3-typography-spacing.html` | Type scale, spacing, radii, shadows |
| `A4-core-components.html` | Atoms: buttons, badges, inputs, avatars, pills, indicators |
| `A+1-triage-screen.html` | AI-00 conversational triage + morphing context bar (8 states) |
| `A+2-catatan-saksi.html` | Vault UI — 5-state lifecycle + seal bar |
| `A+3-siaga-broadcast.html` | Emergency broadcast — 4-state lifecycle + broadcast bar |
| `B0-seed-card.html` | Universal card anatomy (5 views including Rahasia variants) |
| `B1-tuntaskan-card.html` | Tuntaskan track — 6 states + dual-tab + LLM architecture |
| `B2-wujudkan-card.html` | Wujudkan track — 7 states + milestones + Galang fundraising |
| `B3-telusuri-card.html` | Telusuri track — 5 states + hypotheses + evidence board |
| `B4-rayakan-card.html` | Rayakan track — 4 states + validation + appreciation wall |
| `B5-musyawarah-card.html` | Musyawarah track — 6 states + voting + Ketetapan document |
| `C1-rahasia-overlays.html` | Privacy system — 4 levels applied to same card |
| `C2-ai-surface-states.html` | AI badges, confidence bands, diff cards, moderation holds |
| `C3-navigation-feed.html` | 5-tab nav, feed, search, scope picker, profile (CV Hidup) |
| `C4-catatan-saksi-feed.html` | Vault timeline, sealed entries, wali management |
| `C5-tandang-credit.html` | Reputation: tiers, credit toasts, vouch, GDF weather |
| `C6-share-sheet.html` | External sharing: share sheet, OG previews, Siaga broadcast, invite |
| `D2-style-guide.html` | Live component reference (all tokens + components rendered) |
| `card-faces.html` | Cross-component quick reference (compact card variants) |
| `entity-evolution.html` | Entity lifecycle map and transition overlays |
| `prototype.html` | Prototype app shell and interaction scaffolding |
| `wireframe.html` | Low-fidelity screen layout baseline |

## What to Evaluate

### 1. Flow Completeness

- Can a user go from app install → first contribution → seeing results, without hitting an undefined screen?
- Is every lifecycle state for all 5 tracks fully specified (what the user sees, what actions are available)?
- Are transitions between states clear (what triggers the move, who triggers it, what happens to the UI)?
- Is the AI-00 triage → card handoff fully defined for all 3 modes?

### 2. Edge Cases & Error States

- What happens when things go wrong? (network errors, empty states, permission denied, content reported)
- What does a brand new community with 0 seeds look like?
- What happens when a user is the only person in their RT?
- What about offline/poor connectivity (common in Indonesian rural areas)?
- What if AI-00 can't classify at all? Is the fallback clear?
- What if a Siaga broadcast has no responders?
- What about content moderation appeals?

### 3. Cross-Feature Interactions

- Does Rahasia (privacy) interact correctly with Share (C6)? (L1+ should block sharing)
- Does Rahasia interact correctly with Tandang (reputation)? (L2 anonymous but still earns credit?)
- Does the dual-tab pattern work consistently across all tracks?
- Do AI touch points overlap or conflict with each other?
- Is the credit system (AI-09) consistent across all 5 tracks + all contribution types?

### 4. Missing Screens or Flows

Look for screens that a real app would need but may not be explicitly designed:

- Onboarding / first-time user experience
- Settings / preferences
- Account management (edit profile, change area, delete account)
- Notification permission prompts
- Push notification design (what appears on lock screen)
- Admin/moderator dashboard
- Reporting/flagging content flow
- Blocking/muting users
- Search results (beyond the overlay in C3)
- Deep link landing pages (when someone opens a shared link)
- App store listing / marketing page
- Terms of service / privacy policy consent

### 5. Technical Feasibility Concerns

- Is the LLM architecture (7 block primitives, 4 trigger modes) realistic to implement?
- Are the AI confidence thresholds practical (≥80% auto, 50-79% suggest, <50% uncertain)?
- Is the ESCO-ID skill taxonomy (13,000 skills) manageable for a community app?
- Is the SHA-256 sealing for Catatan Saksi specified enough for implementation?
- Is the GDF Weather system (community difficulty floor) calculable from available data?
- Are the 10 AI touch points too many for an MVP?

### 6. Consistency & Contradictions

- Do color tokens in HTML prototypes match DESIGN-DNA-v0.1.md?
- Do lifecycle names in card HTMLs match DESIGN-CONTEXT.md terminology?
- Does AI-SPEC-v0.2.md align with how AI is shown in the HTML prototypes?
- Does UI-UX-SPEC-v0.5.md match the actual HTML implementations?
- Are there any places where two documents say different things about the same feature?

### 7. Internationalization & Accessibility

- Is the Indonesian terminology consistent throughout?
- Are there any English-only labels that should be Indonesian?
- Is the design accessible (contrast ratios, touch targets, screen reader considerations)?
- How does the design handle right-to-left text (if ever needed)?
- Are font sizes adequate for older users (common in RT communities)?

## Output Format

Please structure your review as:

```
## Summary
[One paragraph: overall assessment]

## Completeness Score
[X/10 — how ready is this for a dev team to start building?]

## Critical Gaps (must fix before development)
1. [Gap description + which files are affected + suggested fix]
2. ...

## Important Gaps (should fix, but not blocking)
1. ...

## Nice-to-Have Improvements
1. ...

## Contradictions Found
1. [File A says X, File B says Y — which is correct?]

## Strongest Aspects
1. [What's done really well]

## Questions for the Design Team
1. [Things that are ambiguous and need clarification]
```

## Context

This design system was built iteratively over multiple sessions. All 22 design steps have been reviewed and locked. The intent is that a development team should be able to pick up these documents and build the platform without needing the original designers in the room for every question. Your review helps us find the gaps before that handoff happens.
