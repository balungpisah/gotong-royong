> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 19. Bagikan Create Flow (REWRITTEN — AI-00 Triage)

### 19.1 Trigger

FAB [+] → opens Bagikan full-screen conversational interface.

### 19.2 AI-00 Conversational Triage

No empty textarea. AI-00 greets: "Ceritakan apa yang kamu lihat atau alami..." User tells their story. AI probes if needed. Morphing context bar above keyboard tracks classification state.

#### Context Bar States (8)

| State | Visual | Meaning |
|---|---|---|
| Listening | Empty bar, wave indicator | AI listening, no classification yet |
| Probing | Bar + signal bars | AI asking follow-up for clarification |
| Leaning | Tappable track pill | AI has initial guess (tappable preview) |
| Ready | Full card: track + confidence + seed type | Classified → Community flow |
| Vault-ready | Dark card (vault palette) | Story directed to Catatan Saksi |
| Siaga-ready | Red pulsing card | Emergency detected → Siaga broadcast |
| Split-ready | Split card | Story can split to 2 flows (linkability warning) |
| Manual | Grid: 5 tracks + vault | User tapped "Pilih sendiri" (Siaga not manually selectable) |

#### Conversation Flow

1. User taps [+] → AI greets
2. User describes situation (text, voice, or mixed)
3. AI may probe: urgency, privacy, scale, evidence
4. Context bar morphs through states as confidence builds
5. When ready → context bar shows classification card
6. User confirms or taps "Ubah" to change
7. Conversation text → first message in Bahas Diskusi tab (context carries over)

#### Vault/Siaga Detection

Vault signals: takut, rahasia, kekerasan, KDRT, pelecehan, ancaman, "jangan beritahu siapapun". AI: "Sepertinya ini bersifat pribadi. Ingin menyimpan sebagai Catatan Saksi?"

Siaga signals: kebakaran, banjir, gempa, darurat medis, kecelakaan. AI: "Ini terdeteksi sebagai situasi darurat. Ingin menyiarkan Siaga sekarang?" Red pulse on context bar.

### 19.3 Attachments & Settings

During conversation: attach photos/videos (max 5), auto-detected location (adjustable), Rahasia toggle (L0-L3), Rutin toggle (Tuntaskan/Wujudkan/Musyawarah only).

### 19.4 Preview & Submit

Context bar classification card → user confirms → seed created. ESCO skills auto-tagged from AI-00. Duplicate detection (AI-03): context bar pill "⚠ Saksi serupa (87%)" with comparison card. Redacted preview for Rahasia (AI-02 claim_summary, AI-08 blurred media).

### 19.5 Edit/Delete Rules

| Action | Condition | Mechanism |
|---|---|---|
| Edit | Within 15 min OR before first co-witness | Direct edit |
| Edit (after) | PIC flags factual error | Author 24h edit prompt |
| Delete | No co-witnesses yet | Direct delete |
| Delete (after co-witnesses) | Community consent | 24h consent window |
| Track reclassify (before Bahas) | Author changes freely | Direct change |
| Track reclassify (after Bahas) | Governed proposal | 1.5x quorum, 72h challenge |

---

