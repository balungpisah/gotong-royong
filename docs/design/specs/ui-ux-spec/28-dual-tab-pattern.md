> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 27. Dual-Tab Pattern (NEW)

Every non-bookend stage has 2 swipeable tabs:

| Tab | Content | Style |
|---|---|---|
| Tab 1 | Structured content (task board, criteria, verification, etc.) | Purpose-built per stage |
| Tab 2 | Conversation (discussion thread) | WhatsApp-style chat bubbles |

Tab bar below app bar. Active tab: underline in track color. Hint: "‹ geser ›". Notification dot (red 6px) for unread content.

Chat style: `.chat-bubble.other` (left, white bg), `.chat-bubble.self` (right, track-soft bg). AI inline cards appear as special cards within chat flow.

### 27.1 Tab Mapping Example (Tuntaskan)

| Phase | Tab 1 | Tab 2 |
|---|---|---|
| Pembahasan | 💬 Diskusi | 📋 Rangkuman |
| Perencanaan | 📋 Papan GR | 💬 Koordinasi |
| Pelaksanaan | ✅ Progres | 💬 Koordinasi |
| Verifikasi | 📊 Laporan | 💬 Tanggapan |

---

