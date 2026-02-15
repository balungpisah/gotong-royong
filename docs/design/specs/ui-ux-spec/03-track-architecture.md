> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 2. Track Architecture

Every entity starts as a signal — a testimony, idea, question, good news, or proposal. AI-00 conversational triage classifies the seed type and suggests a track; confirmation is implicit unless challenged during Bahas. Five tracks represent five fundamentally different journeys:

| Track | Indonesian | Spirit | Seed Type | Energy |
|---|---|---|---|---|
| TUNTASKAN | Tuntaskan | Fix a problem (reactive) | Keresahan (concern) | Tenaga + Modal |
| WUJUDKAN | Wujudkan | Build something new (proactive) | Gagasan (idea) | Tenaga + Modal |
| TELUSURI | Telusuri | Understand / investigate | Pertanyaan (question) | Pikiran |
| RAYAKAN | Rayakan | Honor an achievement | Kabar Baik (good news) | Hati |
| MUSYAWARAH | Musyawarah | Decide together (governance) | Usul (proposal) | Suara |

Universal entry point: "Bagikan" (share) via AI-00 conversational triage (see Section 19). The user tells their story; AI classifies and users can challenge/change classification (no explicit community-wide confirmation step).

Track changes are allowed at any stage via governed proposal + vote (1.5x quorum, 72h challenge window). All reputation earned carries over.

### 2.1 Track Component Summaries

**Tuntaskan (Fix It):** 6 states (Keresahan → Bahas → Rancang → Garap → Periksa → Tuntas). Unique components: Papan Gotong Royong (task list), contribution slots (anonymous), PIC-controlled readiness checklist, Periksa peer verification. Also locks LLM ↔ UI Architecture (7 block primitives, source tags, 4 triggers).

**Wujudkan (Build It):** 7 states (Gagasan → Bahas → Rancang → Galang → Garap → Rayakan → Tuntas). Unique components: Milestone tracker, Galang sub-lifecycle (6 financial fields protected 🔒), Rayakan celebration display.

**Telusuri (Explore It):** 5 states (Pertanyaan → Dugaan → Uji → Temuan → Tuntas). Unique components: Hypothesis cards (5 states: Diajukan/Diuji/Terbukti/Ditolak/Belum Jelas), evidence board with support/refute/neutral indicators, Temuan document with confidence meter, track-change suggestion card.

**Rayakan (Honor It):** 4 states (Kabar Baik → Sahkan → Apresiasi → Tuntas). Unique components: Validation panel (endorsement threshold), recognition card (badge + stats), appreciation wall (community messages + emoji reactions), optional post-Tuntas Dampak panel (time-trigger, before/after comparison).

**Musyawarah (Decide Together):** 6 states (Usul → Bahas → Putuskan → Jalankan → Tinjau → Tuntas). Unique components: Position board (options + support bars + supporter chips), vote panel (system-authoritative, quorum bar, timer, anonymous + immutable notice), Ketetapan formal document, Tinjau review panel (time-trigger).

---

