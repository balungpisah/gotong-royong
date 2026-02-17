> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 6. Governance, Voting & Participation

### 6.1 Governance Voting

**Standard Vote**
1 person = 1 vote. Quorum: min 30% verified (max 50). Duration: 48h. Result: >50% wins.

**Weighted Vote**
Critical governance only. Weight by I-score (I^1.5). Quorum: 40% weighted. Duration: 72h. Result: >60% weighted wins.

**1.5x Quorum (Governed Proposal)**
Plan reclassification, removal votes. 1.5× quorum. 72h + 72h challenge. Challenge → Jury.

**Consensus Check**
Consent window: X hours. Auto-advance if no objection.

**Display on Cards**
Badge: [Vote Type] | [Hours Remaining] | [Tally]. Weight visualization for weighted votes.

### 6.2 Evidence: The Triad

Testimony (I): Direct witness account. Corroboration (C): Co-witness support. Document (D): Photo, video, receipt. EQ = (I + 0.5×C + 0.3×D) / 2. Max 5 media items. Verification at Periksa: EQ score, visual inspection, timeline check, logic check. Result: Pass, Needs Clarification, Challenged.

### 6.3 Voting Power & Transparency

1 person = 1 vote (default). Weighted: I^1.5. Vote privacy: hidden during vote, revealed after close. Reasoning optional. Delegation planned for v0.6.

### 6.4 Reputable Member Progression

| Tier | Indonesian | Criteria | Permissions |
|---|---|---|---|
| Newcomer | Pengguna Baru | Just joined | Browse, follow, favorites |
| New Member | Anggota Baru | Profile + 0-1 contrib | Witness, comment, no vote |
| Verified | Terverifikasi | Profile + 1+ contrib + I >= 0.2 + 90d | Full permissions |
| Pillar | Pilar | I >= 0.6, J >= 0.5, history | Leadership nomination |
| Key | Kunci | I >= 0.8, J >= 0.7, C_eff >= 0.5 | Highest trust, critical vote weight |

---
