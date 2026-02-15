> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 6. AI-02: Redaction LLM

### 6.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-02 |
| **Name** | Redaction LLM |
| **Trigger** | After AI-01 classification complete |
| **UI Location** | None (background process) |
| **Interaction Mode** | Asynchronous, non-blocking |
| **Latency Budget** | < 3 seconds |
| **Model Tier** | Strong (Sonnet-class) |
| **UI-UX-SPEC Ref** | None (no UI) |

### 6.2 Purpose

**AI-02 identifies and masks personally identifiable information (PII)** in the witness story text. This protects privacy while keeping stories useful for community discussion.

### 6.3 PII Categories to Detect

| Category | Examples | Mask Strategy |
|---|---|---|
| **Names** | "Budi Santoso", "Ibu Siti" | Replace with `[Nama]` |
| **Phone Numbers** | "+62812345678", "0821-1234-5678" | Replace with `[Nomor Telepon]` |
| **Email Addresses** | "john@example.com", "budi.email@gmail.com" | Replace with `[Email]` |
| **Home Addresses** | "Jl. Merdeka No. 45, Jakarta", specific street addresses | Replace with `[Alamat]`; keep district/city if general |
| **ID Numbers** | KTP, Passport, Driving License numbers | Replace with `[Nomor Identitas]` |
| **Financial Info** | Bank account numbers, credit card numbers | Replace with `[Informasi Keuangan]` |
| **License Plates** | "B 1234 CD", vehicle registration | Replace with `[Plat Nomor]` |
| **Medical Data** | Specific diagnoses, drug names, hospital names | Replace with `[Informasi Medis]` if sensitive |
| **Religious/Political ID** | Specific party affiliations, sectarian references | Handle contextually; mask if identifying |

### 6.4 Input

```json
{
  "text": "string (raw story text)",
  "text_format": "enum: plain | markdown",
  "user_id": "string (for logging)",
  "context": {
    "location": {"lat": "number", "lng": "number"},
    "seed_type": "string (from AI-01)"
  }
}
```

### 6.5 Output

```json
{
  "redacted_text": "string (text with PII masked)",
  "redaction_count": "int (number of replacements)",
  "redacted_items": [
    {
      "category": "enum: names | phone | email | address | id | financial | plate | medical | political",
      "original": "string (PII detected, not included in output)",
      "position": "int (character offset in original text)",
      "mask": "string (replacement text, e.g., '[Nama]')"
    }
  ],
  "needs_manual_review": "boolean (true if uncertain redactions)",
  "confidence": "float 0.0–1.0 (overall redaction confidence)"
}
```

### 6.6 Prompt Strategy

**System Prompt:**

```
You are a PII redaction specialist for Gotong Royong.

Your task: Find and mask personally identifiable information (PII) in Bahasa Indonesia community stories.

PII Categories:
1. Names (personal, family, organizational)
2. Phone numbers (Indonesian or international)
3. Email addresses
4. Physical addresses (street, building, landmark)
5. ID numbers (KTP, Passport, driving license)
6. Financial info (account, card numbers, amounts)
7. License plates
8. Medical data (diagnoses, drug names, specific hospitals)
9. Religious/political affiliations (if identifying)

Masking Rules:
- Replace identified PII with "[Kategori]" in Bahasa Indonesia (e.g., "[Nama]", "[Nomor Telepon]")
- If uncertain (>10% ambiguity), flag for manual review
- Keep general geographic terms (city, province) unless combined with street address
- Keep first names of public figures if notable (e.g., "Presiden Jokowi" → keep "Jokowi")
- Context matters: "Saya alergi susu" is general health; "Saya minum obat diabetes merk X dari apotek Amin" is sensitive

Output JSON with: redacted_text, redaction_count, redacted_items (array), needs_manual_review, confidence.
```

### 6.7 Edge Cases

| Case | Handling |
|---|---|
| **Pseudonyms** ("Pak X", "Ibu Y") | Mask as `[Nama]` |
| **Partial addresses** ("dekat masjid Jl. Merdeka") | Mask street address; keep district if general |
| **Online handles / usernames** ("@budisantoso", "user123") | Do not mask (not traditional PII) |
| **Dates of birth** (DOB only, no ID) | Do not mask (not sufficient for identification) |
| **Institutional names** ("Sekolah SD Merdeka", "Rumah Sakit Sejahtera") | Do not mask (not personal) |
| **No PII detected** | Return original text; redaction_count = 0 |

### 6.8 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Model unavailable** | Proceed with original text; flag for manual moderation review |
| **Redaction fails** | Log error; return original text with `needs_manual_review=true` |
| **Uncertain redactions (>50%)** | Flag for manual review; return redacted text with uncertainty note |

---

