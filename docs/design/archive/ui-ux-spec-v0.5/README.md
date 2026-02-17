# ARCHIVED: UI/UX Specification v0.5

**Status:** Archived on 2026-02-17
**Superseded by:** `docs/design/specs/UI-GUIDELINE-v1.0.md`

## Why Archived

v0.5 used a **Dual-Tab Pattern** (swipeable Percakapan + Tahapan tabs with equal weight). v1.0 replaces this with a **Chat-First + Drawable Panel** model where conversation is the primary surface and structured data is ambient/on-demand.

## What Changed

| v0.5 Concept | v1.0 Replacement |
|-------------|------------------|
| Dual-Tab Pattern (equal tabs) | Chat-First + Drawable Phase Panel |
| "Living Card changes faces" per stage | Phase Breadcrumb with summary cards in drawer |
| Phase-Specific Detail Pages | Phase detail rendered in drawable panel |
| Feed-First as primary pillar | Chat-First as primary pillar |

## What Was Preserved

All domain logic was carried forward unchanged: Adaptive Path model, Navigation, Bagikan AI-00 triage, LLM Architecture (7 blocks), Governance, Roles, Reputation, Privacy/Rahasia, Vault, Siaga, Galang, Siarkan, Rutin, ESCO, Dispute, Onboarding, Transitions.

## Do Not Reference These Files for Implementation

Use `docs/design/specs/UI-GUIDELINE-v1.0.md` as the single source of truth.
