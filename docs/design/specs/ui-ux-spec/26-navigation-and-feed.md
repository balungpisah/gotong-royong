> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 25. Navigation & Feed (NEW)

### 25.1 Bottom Navigation (5 Tabs)

| Tab | Icon | Label | Function |
|---|---|---|---|
| 1 | 🏠 | Beranda | Community feed: all seeds, Community Pulse, horizontal track filter tabs |
| 2 | 📝 | Catatan | Catatan Komunitas: lightweight public notes (prices, status, schedules), concept pills, progressive disclosure (S3-B4) |
| 3 | 🤝 | Bantu | Skill-matched opportunities via ESCO. Validated ● vs declared ○ pills. Volunteer counts |
| 4 | 🔔 | Notifikasi | Time-grouped: Hari Ini / Kemarin / Minggu Ini. 7 types: skill-match, credit, mention, stage, vote, stall, digest |
| 5 | ☰ | Lainnya | Hamburger menu: CV Hidup (Profil), Terlibat, Template Saya (S3-C3), Pengaturan |

> **Change (S3-MD3):** Catatan Komunitas replaces Terlibat as primary tab (lowest barrier to entry, best for user acquisition). Terlibat and Profil move to hamburger menu.

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
| 2 — Nearing | Deadline/milestone close | Pelaksanaan H-3, vote 2h left |
| 3 — New | Created within 24h | New seed in your RT |
| 4 — Active | Recent activity | Ongoing discussion |
| 5 — Completed | Selesai | Completed plans |

### 25.6 Horizontal Track Tabs

Below Community Pulse: Semua (default) + 5 track-colored tabs (Tuntaskan, Wujudkan, Telusuri, Rayakan, Musyawarah). Swipeable.

### 25.7 Search

Full-screen overlay with 3 filter groups: track, ESCO skill, time range. Highlighted matched skill tags in results.

---

