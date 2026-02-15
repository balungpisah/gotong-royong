> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 25. Navigation & Feed (NEW)

### 25.1 Bottom Navigation (5 Tabs)

| Tab | Icon | Label | Function |
|---|---|---|---|
| 1 | 🏠 | Beranda | Community feed: all seeds, Community Pulse, horizontal track filter tabs |
| 2 | 📋 | Terlibat | Seeds where user is involved (author/PIC/contributor/voter). Streak banner + role badges + SVG progress rings |
| 3 | 🤝 | Bantu | Skill-matched opportunities via ESCO. Validated ● vs declared ○ pills. Volunteer counts |
| 4 | 🔔 | Notifikasi | Time-grouped: Hari Ini / Kemarin / Minggu Ini. 7 types: skill-match, credit, mention, stage, vote, stall, digest |
| 5 | 👤 | Profil | CV Hidup: hero + tier badge, I/C/J score cards, dual-layer skills, contributions, vouch, impact, QR |

### 25.2 App Header

```
[scope ▼]    Gotong Royong    [🔍] [+]
```

Scope selector (left): current area, e.g. "RT 05 ▼" → opens scope picker sheet. Search 🔍 (right): full-screen overlay with filters. Compose [+] (right): opens AI-00 triage.

### 25.3 Scope Hierarchy (7 Levels)

| Level | Name | Example | Approx Size |
|---|---|---|---|
| 7 | Nasional | Indonesia | 275 million |
| 6 | Provinsi | Jawa Barat | ~50 million |
| 5 | Kota/Kabupaten | Kota Depok | ~2 million |
| 4 | Kecamatan | Cimanggis | ~200 thousand |
| 3 | Kelurahan/Desa | Tugu | ~15 thousand |
| 2 | RW | RW 03 | ~1,000 |
| 1 | RT | RT 05 | ~150 |

Scope picker: bottom sheet with drag handle, 7-level breadcrumb, opacity gradient showing distance from home scope. Terapkan button to confirm.

### 25.4 Community Pulse Bar

In Beranda header: `☀️ Cerah · 14 aktif · 3 baru · 1 vote`. GDF Weather emoji + live stats. Tappable for detail.

### 25.5 Feed Priority (Action-Weighted, 5 Levels)

| Priority | Condition | Example |
|---|---|---|
| 1 — Your Action | Seed needs your action | PIC assigned you, vote open |
| 2 — Nearing | Deadline/milestone close | Garap H-3, vote 2h left |
| 3 — New | Created within 24h | New seed in your RT |
| 4 — Active | Recent activity | Ongoing discussion |
| 5 — Completed | Tuntas | Finished seeds |

### 25.6 Horizontal Track Tabs

Below Community Pulse: Semua (default) + 5 track-colored tabs (Tuntaskan, Wujudkan, Telusuri, Rayakan, Musyawarah). Swipeable.

### 25.7 Search

Full-screen overlay with 3 filter groups: track, ESCO skill, time range. Highlighted matched skill tags in results.

---

