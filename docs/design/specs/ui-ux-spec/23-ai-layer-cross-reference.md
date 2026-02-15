> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 22. AI Layer Cross-Reference (UPDATED — 10 Touch Points)

| AI ID | Name | This Spec Section | UI Element | Mode |
|---|---|---|---|---|
| AI-00 | Conversational Triage | 19.2 | Bagikan screen, morphing context bar | Conversational |
| AI-01 | Track & Seed Classifier | 19.2 (internal) | Called by AI-00 internally | One-shot |
| AI-02 | Redaction LLM | 14.3, 19.4 | Background; redacted preview | One-shot |
| AI-03 | Duplicate Detector | 19.4 | Context bar pill + comparison card | One-shot |
| AI-04 | Content Moderation | 14, 19.4 | Invisible unless triggered | One-shot |
| AI-05 | Gaming Detection | 14.7 | PIC/Peninjau dashboard alerts | Async batch |
| AI-06 | Criteria Suggestion | 4 | Bahas editor chat + cards | Conversational |
| AI-07 | Discussion Summary | 14.8 | Thread summary card, digest | One-shot |
| AI-08 | Media Redaction | 14.3, 19.3 | Background; redacted media | One-shot (CV) |
| AI-09 | Credit Accreditation | 13 | Toast, chat nudge, Tuntas diff card | Mixed |

What is NOT AI: ESCO extraction, difficulty estimation (Tandang), PIC suggestion (Tandang C_eff), Galang discrepancy (backend), jury selection, anti-collusion, PageRank, vouch graph (Tandang), state machines, quorum, timers, role enforcement, OTP, ledger, notifications (backend).

See AI-SPEC v0.2 for full specifications of all 10 touch points.

---

