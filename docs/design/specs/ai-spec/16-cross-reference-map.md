> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 16. Cross-Reference Map

This section maps AI-SPEC v0.2 concepts to UI-UX-SPEC v0.5 and Tandang backend.

| AI Touch Point | UI-UX-SPEC v0.5 Section | Backend Owner | Data Flow |
|---|---|---|---|
| **AI-00 Triage** | Section 19 (Bagikan) | GR + Tandang | User input → GR → Tandang indexing |
| **AI-01 Classification** | Implicit in Bagikan | GR | AI-00 calls AI-01; output to seed record |
| **AI-02 Redaction** | Background (no UI) | GR | Post-AI-01; output to moderation |
| **AI-03 Duplicate** | C2 (Duplikat view) | Tandang (vector) | Tandang search; GR displays results |
| **AI-04 Moderation** | C2 (Moderasi view) | GR + Tandang | Moderation queue; timer countdown |
| **AI-05 Gaming** | Moderator dashboard | Tandang + GR | Continuous background scan; flags to moderator |
| **AI-06 Criteria** | Planning / Execution phases | GR | User taps "Saran"; AI-06 output editable |
| **AI-07 Summary** | Section 18 (Percakapan) | GR + Tandang | Tandang provides messages; GR displays summary |
| **AI-08 Media** | Section 19 (Bagikan), media upload modal | GR | User uploads; AI-08 scans; user approves redaction |
| **AI-09 Credit** | Completion phase, credit distribution card | Tandang (audit log) + GR (UI) | Tandang tracks; GR proposes; PIC approves |

---

