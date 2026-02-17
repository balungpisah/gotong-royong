> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 23. Catatan Saksi (Vault) (NEW)

Private witness vault — separate dark UI space for recording and sealing personal testimonies.

### 23.1 Entry

Via AI-00 vault-ready context bar state, or dedicated vault access from Profil tab.

### 23.2 Vault Palette

Exclusively uses vault colors: `--v-deep` (#263238) through `--v-wash` (#ECEFF1). No bottom navigation — vault is a separate space. Vault-bar header with compose [+] button.

### 23.3 Vault Header

Stats bar: catatan count / wali count / diterbitkan count. 4 filter tabs: Semua / Tersegel / Dengan Wali / Diterbitkan. Pattern alert banner (if AI detects patterns across entries).

### 23.4 Five States

| State | UI | Purpose | Seal Status |
|---|---|---|---|
| Menyimpan | Compose: text pre-filled from AI-00, attachment tools | Write sealed entry | Unsealed |
| Tersegel | SHA-256 hash, timestamp, encrypted badge | Tamper-proof evidence | Sealed 🔒 |
| Wali | Trustee search, permission list, tier badge | Appoint guardian | Sealed 🔒 |
| Terbitkan | Orange warning, 3 consequences, track picker, Rahasia L2 toggle | Publish to community (IRREVERSIBLE) | Sealed → Published |
| Pola | AI pattern detection, gentle alert, resource links | Detect violence/exploitation patterns | Sealed 🔒 |

### 23.5 Detail Tersegel

Full sealed entry view: unclamped text, complete SHA-256 hash with copy hint, attachment gallery, compact Wali section with permission icons, seal bar actions.

### 23.6 Seal Bar

Bottom bar that morphs between: Unsealed (edit mode, save button) → Sealed (locked, actions: Ganti Wali / Terbitkan).

### 23.7 Wali System

Wali (guardian) permissions: ✓ read, ✓ surface-with-consent, ✕ edit, ✕ share. Two-perspective hub: "Wali Anda" (your guardians, with entry counts) and "Anda Menjadi Wali" (you as guardian for others). Permission legend displayed prominently.

### 23.8 Publish Flow (Terbitkan)

Warning with 3 consequences: (1) identity may be revealed, (2) enters community lifecycle, (3) cannot undo. Track picker (5 tracks). Rahasia L2 toggle available. Split visual: dark vault side → warm Tanah community card preview.

### 23.9 Pattern Detection (Pola)

AI detects patterns across sealed entries (e.g., repeated violence, exploitation). Gentle alert: flagged entry timeline with excerpts + pola tags. 4 Indonesian crisis resources: Telepon Sahabat 119 ext 8, Komnas Perempuan 021-3903963, LPSK, LBH. Dismissible — never forced.

### 23.10 Reputation

Zero Tandang credit while sealed. Credit only accumulates if published to community via Terbitkan flow.

---

