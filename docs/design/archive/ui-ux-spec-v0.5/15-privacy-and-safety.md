> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 14. Privacy & Safety Enforcement (UPDATED)

### 14.1 Verified Member

Profile complete (name + kelurahan) + >= 1 contribution + I >= 0.2 + active 90 days.

### 14.2 Designated Authorities

RT/RW head (registered). Safety contacts (voted). Max 3-5 per community. I >= 0.7 + J >= 0.5.

### 14.3 Rahasia Levels (4-Level Overlay System)

| Level | Name | Author | Content | Media | Hex | Reversibility |
|---|---|---|---|---|---|---|
| L0 | Terbuka | Full name | Public | Full | — | — |
| L1 | Terbatas | Full name | Verified only | Full | #8D6E63 | Reversible |
| L2 | Rahasia | Anonymous (gray) | Request-access gate | Blur | #5D4037 | Up: reversible. Down: IRREVERSIBLE |
| L3 | Sangat Rahasia | Hidden | Redacted hatched | Hatched | #3E2723 | IRREVERSIBLE identity |

### 14.4 Redacted Summary

AI-02 `claim_summary` for feed/search. Canonical stub for internal moderation/notification only.

### 14.4a AI Review States

**Dalam peninjauan:** AI-02 redaction confidence < 0.70. Yellow badge. Excluded from search. L2+ hidden from non-participants. Peninjau 24h SLA.

**Menunggu moderasi:** AI-04 unavailable AND author tier ≤ 1. Not published. Author sees status. Async review within 1h.

### 14.5 Reporting

Categories: Hoax, Harassment, Danger, Spam, Other. 3 reports = flag. 5 reports = hide. Laporkan → Kontak Darurat first (48h), escalate to Jury if unresolved after 7 days.

### 14.6 Defamation Safeguards

Evidence + corroboration before visibility. Right to response (48h). False accusation: I -0.05, J -0.03. Repeat: Shadow status. 72h mandatory delay on Level 2.

### 14.7 PIC/Peninjau Safety Dashboard

AI-05 gaming alerts as cards. Severity badge, anonymized actors, evidence summary, recommended action. PIC: Abaikan/Pantau/Eskalasi. 3x recurrence → auto-escalate.

### 14.8 Thread Ringkasan UI

>10 messages → Ringkasan button. Collapsed card: "Ringkasan • [N] peserta • [sentiment]". Key decisions, open questions, feedback controls. Weekly digest: Monday 07:00 local. Rahasia excluded. <3 active witnesses → no digest.

---

