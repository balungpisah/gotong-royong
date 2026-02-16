> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 3. Adaptive Phase Patterns

> **Replaces:** "Stages Per Track" (archived at `docs/design/archive/04-stages-per-track.md`).

The adaptive path model does not prescribe fixed stage sequences. Instead, the LLM proposes phases and checkpoints tailored to each case. However, common **phase patterns** emerge from the nature of the work. These patterns are not enforced — they are templates the LLM may draw on.

### 3.1 Problem-Solving Pattern (cf. Tuntaskan)

Typical phases when a concern needs resolution:

| Phase | Objective | Common Checkpoints |
|---|---|---|
| Stabilisasi | Understand the problem and gather information | Collect reports, assign PIC, assess urgency |
| Perencanaan | Design the fix | Break into tasks, estimate resources, get approval |
| Pelaksanaan | Do the work | Task completion, evidence collection, heartbeats |
| Verifikasi | Confirm the fix works | Peer review, challenge window, sign-off |

### 3.2 Creation Pattern (cf. Wujudkan)

Typical phases when building something new:

| Phase | Objective | Common Checkpoints |
|---|---|---|
| Pematangan Ide | Refine the vision | Feasibility check, scope definition, PIC assignment |
| Persiapan | Plan and resource | Task breakdown, resource pooling (Galang), timeline |
| Pembangunan | Build it | Milestone delivery, progress heartbeats |
| Perayaan | Celebrate the creation | Showcase, community recognition |

### 3.3 Investigation Pattern (cf. Telusuri)

Typical phases when seeking understanding:

| Phase | Objective | Common Checkpoints |
|---|---|---|
| Perumusan | Frame the question | Define scope, propose hypotheses |
| Pengujian | Research and test | Evidence collection, experiment tracking |
| Penemuan | Synthesize findings | Confidence assessment, conclusion document |

Exit: findings may spawn a new plan (problem found → new problem-solving plan, idea emerged → new creation plan).

### 3.4 Celebration Pattern (cf. Rayakan)

Typical phases when honoring achievement:

| Phase | Objective | Common Checkpoints |
|---|---|---|
| Validasi | Community validates the achievement | Endorsement threshold met |
| Apresiasi | Public recognition | Recognition card, appreciation wall |
| Dampak | Track lasting impact (optional) | Before/after comparison, attestations |

### 3.5 Governance Pattern (cf. Musyawarah)

Typical phases when deciding together:

| Phase | Objective | Common Checkpoints |
|---|---|---|
| Pembahasan | Deliberate and present options | Position board populated, debate concluded |
| Keputusan | Community votes | Quorum reached, result declared |
| Pelaksanaan | Implement the decision | Assignments created, timeline set |
| Tinjauan | Review effectiveness (optional) | Effectiveness assessment, follow-up actions |

Output: Ketetapan (formal decision/ruling document).

### 3.6 How the LLM Uses Patterns

The LLM is not hard-coded to these patterns. It uses them as heuristics when the user's situation aligns. The LLM may combine patterns (e.g., investigation phase followed by execution phases), skip phases, or create entirely novel phases based on context. The `track_hint` metadata may guide pattern selection but does not constrain it.

---
