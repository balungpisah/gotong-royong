> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 13. AI-09: Credit Accreditation (NEW)

### 13.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-09 |
| **Name** | Tandang Credit Accreditation |
| **Trigger** | 1) Continuous silent tracking. 2) Computed block at plan completion (final phase). 3) On-demand credit distribution form. |
| **UI Location** | Credit toast (inline), AI nudge in Percakapan tab chat, Kontribusi diff card at plan completion |
| **Interaction Mode** | Mixed — passive tracking + one-shot summary at plan completion |
| **Latency Budget** | Toast: instant. Completion summary: < 5 seconds. |
| **Model Tier** | Medium (Haiku-class) |
| **UI-UX-SPEC Ref** | Execution phases, Completion phase, credit distribution card |

### 13.2 Purpose (New v0.2)

**AI-09 tracks community contributions throughout an adaptive path's lifecycle** and proposes fair credit (reputation point) distribution when the plan reaches completion (final phase completed). Credit rewards:
- Seeding the issue (idea creation)
- Thoughtful discussion (idea refinement)
- Task execution and problem-solving
- Peer validation and expert verification
- Personal risk-taking (vouching for credibility)
- Fundraising efforts

### 13.3 Credit Types (A–E)

| Type | Name | Scoring Model | When Award | Examples |
|---|---|---|---|---|
| **A** | Binary Verification | Did / Didn't | Immediate after action | Submit seed, vote, complete task |
| **B** | Time-Weighted | Effort over duration | Batched daily | Hours spent in planning, days managing Galang |
| **C** | Peer Consensus | Group validates quality | After peer review threshold | Validation count (Sahkan), verification (Periksa) |
| **D** | Quality Spectrum | Quality rating | After PIC or peer rate | Discussion quality, proposal quality (1–5 stars) |
| **E** | Stake-Weighted | Reputation at stake | After action confirmed | Vouch (Jaminkan), personal guarantee |

### 13.4 Five-Step Credit Flow

**Step 1: Silent Tracking**
- System logs every action: who, what, when, duration
- Stored in Tandang audit log (immutable)

**Step 2: Instant Feedback**
- Type A/B → immediate toast: "💡 Kontribusi tercatat (Tipe A · Keputusan)"
- Type C → toast when threshold hit: "✓ 5 orang validasi — kontribusi tercatat"
- Type D → toast after PIC rate: "⭐⭐⭐⭐ Kualitas tinggi — kontribusi tercatat"
- Type E → toast after vouch: "🔐 Jaminkan diterima — reputasi Anda tercatat"

**Step 3: AI Nudge in Chat**
- AI-09 sends inline message in Percakapan tab during discussion:
  > "💡 Diskusi berkualitas — kontribusi Anda dicatat (Tipe D · Kompetensi)"
- Message counts toward discussion participation but is not replicated

**Step 4: Completion Summary**
- When plan reaches completion (final phase), AI-09 proposes Kontribusi distribution as **diff card**
- Diff card shows: Proposed allocation (who gets how many points for what action)
- PIC reviews: **Terapkan** (apply) / **Tinjau** (review & edit) / **Tolak** (reject)

**Step 5: Dispute Mechanism**
- Participants can flag automated credit within 72h: "Ini tidak adil karena..."
- Dispute goes to peer review (3–5 community members vote)
- If consensus is reached, AI-09 mediates adjustment
- If no consensus, seed credit locked and issue escalated to community council

### 13.5 GR Action → Tandang Credit Mapping (11 Mappings)

| GR Action | Context Axis | Credit Type | Base Points | Notes |
|---|---|---|---|---|
| **Submit seed** | Initiative (I+) | A | 10 | Fixed: one per seed |
| **Diskusi di phase pembahasan** | Context (C) | D | Variable (1–5) | Based on discussion quality rating |
| **Kontribusi phase perencanaan** | Context (C) | B | Variable | 1 point per 2 hours, capped 20 |
| **Selesaikan task phase pelaksanaan** | Context (C) | A | Variable | 10 + task complexity bonus (0–10) |
| **Validasi di phase validasi** | Judging (J) | C | 5 + consensus | 5 base + 1 per additional validating peer |
| **Vote di phase keputusan** | Initiative (I+) | A | 2 | 2 points per vote, any direction |
| **Verifikasi di phase verifikasi/tinjauan** | Judging (J) | C | 8 | Expert verification weight |
| **Kontribusi Galang** | Context (C) | B | Variable | 1 point per Rp 100k raised, capped 20 |
| **Vouch (Jaminkan)** | Initiative + Stake (I+S) | E | 15 | Highest-stake action; reputation on line |
| **Ajukan hipotesis** | Context (C) | D | 3–7 | Based on discussion impact |
| **Kumpulkan bukti** | Context (C) | B | 5 + evidence quality | 5 base + 0–5 for evidence relevance |

### 13.6 Tandang → AI-09 Data Flow

```
┌─────────────────────────────────────────┐
│   GR User Action                        │
│   (e.g., submit vote, complete task)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Tandang Audit Log                     │
│   {user_id, action, timestamp, ...}     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   AI-09 Tracking Service                │
│   Aggregates actions by user & seed     │
└──────────────┬──────────────────────────┘
               │
        ┌──────┴──────┐
        │             │
   ┌────▼────┐  ┌────▼────┐
   │  Toast  │  │  Chat   │
   │(instant)│  │ Nudge   │
   └────┬────┘  └────┬────┘
        │            │
        └──────┬─────┘
               │
        ┌──────▼──────────┐
        │  At completion: │
        │  Aggregate →    │
        │  Propose Dist.  │
        │  Diff Card      │
        └─────────────────┘
```

### 13.7 Diff Card Layout (at Plan Completion)

```
┌──────────────────────────────────────────────┐
│  📊 Kontribusi & Reputasi                     │
├──────────────────────────────────────────────┤
│                                              │
│  Budi (Seeder)                               │
│    Inisiatif: 10 poin ✓                      │
│    Perencanaan: 5 jam → 10 poin ✓           │
│    Total: 20 poin                            │
│                                              │
│  Siti (Diskusi)                              │
│    Kualitas Diskusi: ⭐⭐⭐⭐ → 4 poin ✓     │
│    Total: 4 poin                             │
│                                              │
│  Ahmad (Eksekusi)                            │
│    Execution Task 1: 15 poin ✓              │
│    Execution Task 2: 15 poin ✓              │
│    Jaminkan (Vouch): 15 poin ✓              │
│    Total: 45 poin                            │
│                                              │
├──────────────────────────────────────────────┤
│  [Terapkan]  [Tinjau]  [Tolak]               │
└──────────────────────────────────────────────┘
```

### 13.8 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Tracking failure** | Log gap detected → async reconciliation within 24h |
| **Completion summary unavailable** | PIC manually distributes credit using simple form (text input) |
| **Credit dispute model unavailable** | Peer review only (no AI mediation); manual resolution by council |
| **User credit balance error** | Audit log used as source of truth; reconcile forward |

### 13.9 Community Override

- **PIC has final say** on credit distribution at plan completion
- **Participants can dispute** within 72h window
- **Disputes are logged** and can lead to credit adjustments
- **Community council** arbitrates disputed credits

### 13.10 Protected Fields

**The following financial fields are EXCLUDED from AI-09 access:**

- `target` (Galang target amount)
- `terkumpul` (amount collected)
- `sisa` (remaining amount)

These fields are marked with 🔒 **DILINDUNGI** (protected) badge and managed by Tandang backend only. AI-09 can see that fundraising happened, but not the actual amounts.

---

