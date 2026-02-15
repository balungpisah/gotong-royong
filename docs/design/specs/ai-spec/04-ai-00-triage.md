> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 4. AI-00: Conversational Triage (NEW)

### 4.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-00 |
| **Name** | Conversational Triage Agent |
| **Trigger** | User taps [+] compose button anywhere in app |
| **UI Location** | Bagikan screen — full-screen conversational interface with morphing context bar |
| **Interaction Mode** | Conversational (multi-turn, typically 2–5 exchanges) |
| **Latency Budget** | < 2 seconds per AI response |
| **Model Tier** | Strong (Sonnet-class) |
| **UI-UX-SPEC Ref** | Section 19 (Bagikan), file: A+1-triage-screen.html |

### 4.2 Purpose

**AI-00 replaces the old empty-textarea flow.** When a user taps [+] to compose, they are met with a conversational greeting instead of a blank text field. AI-00:

1. Greets the user and invites them to describe what they saw or experienced
2. Listens actively, extracting the witness story
3. May ask 1–3 clarifying questions to determine urgency, privacy needs, scope, and evidence type
4. Internally invokes AI-01 (classification) as context develops
5. Guides the user through the entry flow (Community, Catatan Saksi vault, or Siaga emergency broadcast)
6. Morphs a context bar above the keyboard to visualize AI's understanding in real-time

User never sees an empty text field. The conversation becomes the seed's first message in Bahas Diskusi (context carries over).

### 4.3 Context Bar — 8 States

The context bar sits above the keyboard and visualizes AI's classification confidence. It morphs through these 8 states:

| State | Visual | Meaning | Interaction |
|---|---|---|---|
| **Listening** | Empty bar, wave indicator | AI listening, no classification yet | User types freely |
| **Probing** | Bar + signal bars (1–3) | AI asking follow-up question | User answers, AI listens |
| **Leaning** | Tappable track pill + icon | AI has initial guess (not confident) | User can tap to preview flow |
| **Ready** | Full card: track + confidence % + seed type badge | Classification confident → Community flow | User confirms or taps "Ubah" |
| **Vault-ready** | Dark card (vault palette: purple/dark) | Story directed to Catatan Saksi (private vault) | User confirms or changes |
| **Siaga-ready** | Red pulsing card with warning icon | Emergency detected → Siaga broadcast | User confirms or changes |
| **Split-ready** | Split card (2 pills side-by-side) | Story can split to 2 flows (with linkability warning) | User chooses which takes primary |
| **Manual** | Grid: 5 tracks + vault | User tapped "Pilih sendiri" to bypass AI | User picks manually (Siaga not manually selectable) |

### 4.4 Input

```json
{
  "text": "string (accumulated user messages, full conversation)",
  "media_urls": ["string array, optional attachments"],
  "location": {
    "lat": "number",
    "lng": "number"
  },
  "conversation_history": [
    {
      "role": "user | assistant",
      "content": "string",
      "timestamp": "ISO 8601"
    }
  ],
  "user_id": "string",
  "session_id": "string"
}
```

### 4.5 Output

```json
{
  "entry_flow": "enum: community | vault | siaga",
  "track": "enum: tuntaskan | wujudkan | telusuri | rayakan | musyawarah (only if community)",
  "confidence": "float 0.0–1.0",
  "seed_type": "enum: Keresahan | Gagasan | Pertanyaan | Kabar Baik | Usul",
  "context_bar_state": "enum: listening | probing | leaning | ready | vault-ready | siaga-ready | split-ready | manual",
  "esco_skills": ["array of skill codes extracted from conversation"],
  "follow_up_question": "string | null (if probing state)",
  "reasoning": "string (one sentence in Bahasa Indonesia, e.g., 'Cerita tentang banjir terdeteksi sebagai situasi darurat yang memerlukan siaran langsung.')",
  "is_split_candidate": "boolean (true if could go to 2 flows)",
  "split_flows": ["array of secondary flows, if is_split_candidate=true"],
  "conversation_summary": "string (1–2 sentences capturing the core issue)"
}
```

### 4.6 Conversation Flow

1. **User taps [+]** → Bagikan screen opens with conversational interface
2. **AI greets** → "Halo! Ceritakan apa yang kamu lihat atau alami di komunitas. Kami siap mendengarkan."
3. **User describes** → Types or records a voice message (async transcription)
4. **AI listens** → Context bar enters "Listening" state; AI extracts intent, urgency, privacy signals
5. **AI may probe** → Context bar enters "Probing" state:
   - If urgency unclear: "Apakah ini situasi darurat yang perlu bantuan sekarang?"
   - If privacy unclear: "Apakah kamu ingin cerita ini bersifat pribadi / rahasia?"
   - If scope unclear: "Berapa banyak orang yang terlibat? Apakah ini masalah lokal atau lebih luas?"
6. **Context bar morphs** → As AI gains confidence, bar transitions: Listening → Probing → Leaning → Ready/Vault-ready/Siaga-ready
7. **User confirms** → When bar shows "Ready", user taps "Lanjutkan" or "Ubah"
8. **Entry flow begins** → If Community: Bagikan Step 1 (Rancang track, seed type, skills). If Vault: direct to Catatan Saksi form. If Siaga: launch Siaga broadcast composer.
9. **Context carries over** → Full conversation becomes first message in seed's Bahas Diskusi tab; user doesn't repeat themselves

### 4.7 Prompt Strategy

**System Prompt Core Principles:**

- Greet warmly in Bahasa Indonesia; maintain conversational tone
- Define all 5 Community tracks with clear examples (see Track Definitions below)
- Define vault indicators (see Section 4.7a)
- Define siaga indicators (see Section 4.7b)
- Extract ESCO skills naturally during conversation (no explicit skill-picking UI)
- Probe for: urgency, privacy needs, scale, evidence type, affected parties
- After 2–3 user messages, have enough context to classify; offer classification via context bar
- If user says "Pilih sendiri", show Manual state and allow grid selection
- Always validate assumptions with user before confirming

**Track Definitions (few-shot examples):**

| Track | Bahasa | Example | Seed Types |
|---|---|---|---|
| **Tuntaskan** | Solve a problem | "Ada jalan rusak di Jl. Merdeka yg bikin macet" | Keresahan, Pertanyaan, Usul |
| **Wujudkan** | Realize an aspiration | "Kami ingin buat perpustakaan komunitas" | Gagasan, Usul |
| **Telusuri** | Investigate an issue | "Siapa yg bertanggung jawab atas polusi di sungai?" | Pertanyaan, Kabar Baik (evidence) |
| **Rayakan** | Celebrate achievement | "Kerja bakti kemarin berhasil!" | Kabar Baik |
| **Musyawarah** | Deliberate & decide | "Bagaimana kita harus alokasikan dana komunitas?" | Pertanyaan, Gagasan |

### 4.7a Vault Detection Signals

**Keywords & Patterns:**
- takut, khawatir, takut-takut
- rahasia, pribadi, tidak ingin diketahui
- kekerasan, pelecehan, abuse, penyalahgunaan
- KDRT, domestic violence, trafficking
- ancaman, terancam, terancam bahaya
- tidak aman, merasa tidak aman, takut pada [person]
- "jangan beritahu siapapun", "ini antar kita saja"
- Contextual: divorce, affair, mental health crisis, addiction, self-harm, suicidal ideation

**Context Bar Transition:**
When vault signals detected, context bar transitions to **Vault-ready** state (dark card, purple palette). AI offers clarification:

> "Sepertinya cerita ini bersifat pribadi dan mungkin sensitif. Apakah kamu ingin menyimpan ini sebagai **Catatan Saksi** (bersifat rahasia, hanya terlihat oleh kamu)? Atau kamu lebih suka berbagi dengan komunitas?"

**Vault Guarantee:**
- Catatan Saksi is stored in encrypted Tandang vault
- Not indexed in public discussion
- Zero credit accreditation (see AI-09)
- User can share individual secrets to Diskusi later if desired

### 4.7b Siaga Detection Signals

**Keywords & Patterns:**
- kebakaran, api, fire, burning
- banjir, flooding, water emergency
- gempa bumi, earthquake, seismic
- darurat medis, medical emergency, ambulans
- kecelakaan, accident, crash, tabrakan
- runtuh, collapse, terguling
- "tolong!", "help!", "SOS", "Cepat!"
- ancaman keamanan langsung: penembakan, pemboman, penculikan, terorisme
- Contextual: person trapped, unconscious, bleeding, choking, overdose

**Context Bar Transition:**
When siaga signals detected, context bar transitions to **Siaga-ready** state (red pulsing card, warning icon). AI offers immediate escalation:

> "⚠ **Situasi Darurat Terdeteksi.** Ini memerlukan bantuan sekarang. Ingin menyiarkan **Siaga** (broadcast darurat ke komunitas) sekarang?"

**Siaga Guarantee:**
- Siaga broadcast publishes immediately to all app users within 5km radius
- Broadcast includes emergency type, location, user's contact info (if public)
- Responders can join in real-time via Siaga flow
- Zero moderation hold
- Tandang logs all responders for post-emergency debrief

### 4.8 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Model unavailable** | Show Manual context bar state immediately (grid: 5 tracks + vault). Keep 112 emergency CTA always visible. |
| **Model slow (>5s)** | Show "Pilih sendiri" option in context bar; user can bypass while waiting |
| **Conversation confused** (>5 exchanges without classification) | Auto-show Manual state; suggest manual selection |
| **Duplicate detected** (see AI-03) | Show duplicate pill in context bar with tappable comparison; allow merge or proceed as new |
| **Media upload failure** | Log attachment attempt; proceed with text only; notify user asynchronously |

### 4.9 Community Override

Users always have the option to:
- Tap **"Pilih sendiri"** at any time to see Manual grid (5 tracks + vault). Siaga remains one-tap via 112 CTA and AI detection.
- Tap **"Ubah"** to change the suggested track/flow
- Directly select a community flow without confirmation (context bar pill or Manual grid)

### 4.10 Context Carryover

When user confirms classification and enters the entry flow:

1. Full conversation history (AI-00) is stored as `triage_transcript`
2. Triage transcript becomes the **first message** in the seed's **Bahas Diskusi** tab
3. Attribution: "**[User]** tela cerita melalui triage AI"
4. Subsequent messages in Diskusi tab are authored by community members
5. User can refer back to triage transcript for context without re-reading the entire conversation

### 4.11 Relationship with AI-01

- **AI-01 is internal to AI-00.** User does not see AI-01 directly.
- **AI-00 calls AI-01 multiple times** as conversation develops (not just once at end).
- **Final classification** comes from AI-00's last AI-01 inference (when confidence is high).
- **AI-01 also runs independently** at submission time if classification changes (e.g., Kabar reclassification).

---

