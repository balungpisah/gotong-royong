> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 12. AI-08: Sensitive Media Detection & Redaction

### 12.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-08 |
| **Name** | Sensitive Media Detection & Redaction |
| **Trigger** | At submission (for all attached images/videos) |
| **UI Location** | Media preview, overlay warnings |
| **Interaction Mode** | Synchronous, blocking (if critical issues found) |
| **Latency Budget** | < 3 seconds per image/video |
| **Model Tier** | Computer Vision pipeline (face detection, plate recognition, etc.) |
| **UI-UX-SPEC Ref** | Section 19 (Bagikan), media upload modal |

### 12.2 Purpose

**AI-08 detects and offers to redact:**
- Human faces (privacy protection)
- Vehicle license plates
- Private home interiors or identifying landmarks
- Medical equipment or prescriptions (implied health data)

This prevents accidental doxxing while preserving evidence value of photos.

### 12.3 Detectable Sensitive Elements

| Element | Examples | Action |
|---|---|---|
| **Faces** | Person's face visible in photo | Offer blur/pixelate; user can approve |
| **License plates** | Vehicle registration plate readable | Offer blur; automatic approval likely |
| **Home interiors** | Interior of home with identifying details | Flag for review; suggest redaction |
| **Medical items** | Prescription bottle, doctor's note visible | Flag; automatic blur option |
| **Identifying landmarks** | Street sign with address, house number | Flag; suggest geographic generalization |

### 12.4 Input

```json
{
  "media_urls": ["string array (URLs to images/videos)"],
  "media_types": ["enum array: image | video"],
  "seed_id": "string",
  "author_id": "string",
  "context": {
    "seed_text": "string (for contextual understanding)"
  }
}
```

### 12.5 Output

```json
{
  "scans": [
    {
      "media_url": "string",
      "media_type": "enum: image | video",
      "detections": [
        {
          "element_type": "enum: face | license_plate | home_interior | medical | landmark",
          "confidence": "float 0.0–1.0",
          "location": {
            "x": "float (0–1, normalized to image width)",
            "y": "float (0–1, normalized to image height)",
            "width": "float",
            "height": "float"
          },
          "severity": "enum: low | medium | high",
          "suggested_action": "enum: blur | pixelate | crop | redact_metadata | flag_for_review"
        }
      ],
      "overall_safety": "enum: safe | needs_review | sensitive",
      "redacted_media_url": "string (URL to redacted version if user approves, or null)"
    }
  ],
  "summary": "string (e.g., 'Ditemukan 2 wajah yang dapat di-blur')"
}
```

### 12.6 User Interaction

**In UI:**
1. User attaches image to seed
2. AI-08 scans
3. If detections found:
   - Show overlay preview with bounding boxes
   - Offer checkboxes: "Blur ini?" / "Crop ini?" / "Abaikan?"
4. User selects actions
5. Redacted media stored; original archived (not deleted)

### 12.7 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **CV model unavailable** | Proceed with original media; flag for manual moderation review |
| **High-resolution video** (>5 min, >500 MB) | Skip AI scan; require manual moderation |
| **Unrecognizable image** (too blurry, low res) | Proceed; no detections |
| **User rejects all redactions** | Proceed with original media; moderation will review |

---

