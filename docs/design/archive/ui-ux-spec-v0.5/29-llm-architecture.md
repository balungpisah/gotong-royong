> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 28. LLM ↔ UI Architecture (NEW)

### 28.1 Core Invariants

1. **Two data layers**: conversation (chat tab) + structured (content tab).
2. **AI never auto-applies**: always suggests via diff card. Human decides.
3. **Human edit = lock**: source flips `"ai"` → `"human"`, AI stops touching that item.
4. **Additive-first**: AI adds, suggests, drafts. Never deletes or overwrites human content.

### 28.2 7 Block Primitives

| Block | Renders As | AI Rule | Source Tag |
|---|---|---|---|
| `list` | Checklist, table, timeline, gallery | Additive. Nestable. Status-changeable. | Per-item |
| `document` | Rich text + tracked changes | AI drafts, human edits sections | Per-section |
| `form` | Labeled input fields | AI suggests per field. Protected = hands-off. | Per-field |
| `computed` | Read-only (progress bar, status) | System-derived. Nobody edits. | `system` |
| `display` | Presentation card (recognition) | One-way render. No edit. | `system` |
| `vote` | Voting interface + tally | System tallies. Not AI. | `system` |
| `reference` | Preview of linked card | Links to other cards. | `reference` |

### 28.3 Source Tags

| Tag | Meaning | Rule |
|---|---|---|
| `"ai"` | LLM-generated | Can be overwritten by next pass or human edit |
| `"human"` | Human-created/edited | AI stops touching. Locked. |
| `"system"` | System-computed (vote, progress) | Nobody edits |

### 28.4 4 Trigger Modes

| Mode | When | Touch Point | Output |
|---|---|---|---|
| Manual | User taps 🔄 Perbarui | AI-07 summarization | Diff card in structured tab |
| Milestone | Keyword/pattern at breakpoints | AI-05 resolution check | Stage transition suggestion |
| Time-Triggered | Scheduled interval | AI-04 stall, AI-06 Dampak | Alert in chat tab |
| Passive | Continuous monitoring | AI-08 sentiment, AI-03 anomaly | Badge/indicator only |

### 28.5 Diff Card Anatomy (Suggest-Don't-Overwrite)

For **list**: "Ditambah 2 item, dicentang 1" + evidence quotes. For **document**: tracked-changes style. For **form**: per-field comparison. Actions: **[Terapkan Semua] | [Tinjau Satu-satu] | [Abaikan]**. Protected fields (financial, identity): 🔒 DILINDUNGI badge, excluded from AI access.

---

