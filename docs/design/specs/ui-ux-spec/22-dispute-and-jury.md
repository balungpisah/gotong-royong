> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 21. Dispute & Stochastic Jury

### 21.1 Dispute Triggers

Disputed verification, disputed transition, misuse allegation, defamation (via Laporkan → escalate), challenged vote.

### 21.2 Stochastic Jury Selection

Pool: verified, I >= 0.4, not involved, active 30d. Size: 5 (<200) or 7 (200+). 24h accept/decline. Max 2 declines/year.

### 21.3 Process Timeline

| Phase | Duration | Activity |
|---|---|---|
| Dispute filed | Day 0 | Evidence submitted |
| Jury formation | 24h | Selection, acceptance |
| Evidence review | 72h | Review, clarification |
| Deliberation | 48h | Anonymous discussion |
| Verdict | Day 6 | Dikabulkan/Ditolak/Bukti Kurang |
| Appeal window | 24h | New evidence only |
| If appeal | 5 days | New jury, compressed |
| Total max | ~8 days | First round |

### 21.4 Privacy During Process

Jury identities hidden during process. Jurors see all evidence (Rahasia logged). Deliberation thread destroyed 30d after verdict.

### 21.5 Outcomes

| Outcome | Entity Effect | Offender | Disputer |
|---|---|---|---|
| Upheld | Action reversed | I penalty, possible Shadow | None |
| Rejected | No change | None | I -0.01 |
| Insufficient Evidence | No change, monitoring | None | None |
| Split (e.g. 3-2) | Verdict stands, flagged | None | Mediation offered |

### 21.6 Dispute UI Elements

Keberatan button (yellow/orange ⚠️), Laporkan button (gray/red 🚩). Dispute form modal. Jury dashboard (push screen). Verdict card on entity.

### 21.7 Dispute vs Report

| | Ajukan Keberatan | Laporkan |
|---|---|---|
| Purpose | Process outcome disagreement | Harmful content/behavior |
| Triggers | Stochastic Jury | Kontak Darurat first |
| Timeline | Max 8 days | 48h → escalate to jury at 7d |
| Available on | Post-verification, post-vote | Any content, any stage |

### 21.8 Appeal Rights

Only direct parties (disputer or target). One appeal each. New evidence only (photos/videos/documents/witness statements). New jury, compressed 5-day review.

---

