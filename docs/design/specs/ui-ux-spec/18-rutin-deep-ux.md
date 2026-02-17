> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 17. Recurring Activities (Rutin)

### 17.1 Template-Based Recurrence

When a plan reaches completion, users can save it as a **Recurring Case Template** for reuse. Templates capture the structural pattern of a successful case without its case-specific data.

**Template structure:**
| Element | Preserved | Cleared |
|---|---|---|
| Phase titles, objectives | ✓ | |
| Checkpoint titles | ✓ | |
| Role assignment pattern (PIC type, distribution) | ✓ | |
| Schedule (daily/weekly/monthly/custom) | ✓ | |
| Rahasia level | ✓ | |
| Case-specific evidence, votes, data | | ✓ |
| Participant identities | | ✓ |
| Timestamps, progress | | ✓ |
| Conversation history | | ✓ |

**UI flow:**
```
[Plan reaches completion]
     ↓
[System prompts: "Simpan sebagai template?"]
     ↓
[Editor taps "Simpan Template"]
     ↓
[System sanitizes: strips evidence, votes, identities]
     ↓
[Template tagged with community_id, keywords, track_hint (optional)]
     ↓
[Stored for reuse]
```

### 17.2 Spawn Behavior

When a recurring case is triggered (manually or via schedule), the system creates a **new plan instance** from the template, then the LLM re-evaluates it against current community context.

**Trigger options:**
- **Manual spawn:** User/editor taps "Buat dari template" during entry (Bagikan screen)
- **Scheduled spawn:** System auto-creates instance 48 hours before next scheduled date

**LLM re-evaluation:**
```
[Template cloned as new PathPlan v1]
     ↓
[LLM reviews: "Apakah struktur template ini masih sesuai dengan kondisi komunitas sekarang?"]
     ↓
[LLM may suggest modifications:]
  - Add checkpoint for new regulatory requirement
  - Adjust phase duration based on seasonal context
  - Propose different resource approach
     ↓
[Suggestions posted in Percakapan with consent timeout (Section 3.5 ADAPTIVE-PATH-ORCHESTRATION)]
     ↓
[Editor accepts/rejects modifications]
```

**Community notification (scheduled cases):**
| When | Notification | Action |
|---|---|---|
| 48h before spawn | "Kerja Bakti Bulanan akan dimulai Minggu depan. Siap?" | Skip, Defer, Confirm |
| 24h before spawn | Reminder to PIC + active participants | — |
| On spawn | New case instance created, Percakapan tab opens with LLM suggestions | Editor reviews suggestions |

**Schedule options:**
- Harian (setiap hari)
- Mingguan (pilih hari)
- Bulanan (tanggal atau "Senin minggu ke-2", dsb.)
- Custom (cron-like; e.g., "setiap 3 bulan pada hari Jumat")

### 17.3 Inheritance & Reset

When a recurring case spawns from a template:

**Inherits from template:**
- Phase structure (titles, objectives, checkpoint titles)
- PIC assignment strategy (Fixed individual, round-robin pool, volunteer-based, etc.)
- Membership scope (same community group or broader)
- Rahasia classification level
- Schedule (next occurrence calculated automatically)

**Resets to fresh state:**
- Votes → empty (no prior votes carried forward)
- Evidence → empty (new evidence collected for this cycle)
- Progress / checkpoint status → all "planned" (not yet started)
- Timestamps → current date/time
- Conversation history → empty (new discussion for this cycle)

**Bi-directional links:**
```
[New case instance] ←→ [Previous cycle case]
                        ↓
                   [Template that spawned both]
```

Users can view:
- Link to previous cycle (to review what was accomplished)
- Link to template (to understand the recurring structure)
- Estimate based on previous cycle (duration, effort, participation)

### 17.4 PIC & Task Rotation

PIC (Penanggung Jawab) assignment for recurring cases follows the same rotation rules as one-off cases:

**Rotation strategies:**
| Strategy | Behavior | Use case |
|---|---|---|
| **Fixed** | Same person each cycle | Single capable individual; no rotation needed |
| **Round-robin** | Cycles through a pre-defined list | Fair burden distribution; multiple capable members |
| **Volunteer-based** | Opens for self-nomination before each spawn | Engagement-driven; flexible participation |
| **Hybrid** | Primary PIC fixed; secondary rotates | Accountability + fairness balance |

**Fairness tracking:**
- System logs who held PIC in past N cycles
- Round-robin respects gap constraints ("don't ask someone who did it last month")
- Skipped cycles are tracked → 3 consecutive skips flags relevance question

**Skip-cycle protocol:**
```
[PIC proposes skip for upcoming cycle]
     ↓
[Consent window: 24 hours for community objection]
     ↓
[If no objection → skip recorded; next rotation calculates accordingly]
     ↓
[After 3 consecutive skips → System nudges: "Kegiatannya masih relevan?"]
```

---

