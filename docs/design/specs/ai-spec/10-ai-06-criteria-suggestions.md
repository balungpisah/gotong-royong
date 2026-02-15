> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 10. AI-06: Criteria & Task Suggestion

### 10.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-06 |
| **Name** | Criteria & Task Suggestion |
| **Trigger** | User enters Rancang phase; also on-demand during Garap |
| **UI Location** | Rancang screen, Garap task creation modal |
| **Interaction Mode** | Interactive (user can accept, edit, or ignore suggestions) |
| **Latency Budget** | < 3 seconds |
| **Model Tier** | Strong (Sonnet-class) |
| **UI-UX-SPEC Ref** | Section 18 (Rancang), Section 20 (Garap) |

### 10.2 Purpose

**AI-06 suggests actionable success criteria and task decomposition** to help communities define clear objectives and execution plans. It transforms vague aspirations into concrete, measurable work.

### 10.3 Context Input

```json
{
  "seed_id": "string",
  "track": "string (from AI-01)",
  "seed_type": "string (from AI-01)",
  "seed_text": "string",
  "location": {
    "lat": "number",
    "lng": "number"
  },
  "discussion_summary": "string (from AI-07, if available)",
  "community_context": {
    "community_id": "string",
    "community_size": "int",
    "similar_completed_seeds": ["array of seed_ids"]
  }
}
```

### 10.4 Criteria Output

```json
{
  "suggested_criteria": [
    {
      "criterion_id": "string",
      "text": "string (in Bahasa Indonesia, e.g., 'Jalan diaspal dengan lebar minimal 5m')",
      "measurable": "boolean",
      "verification_method": "enum: visual_inspection | measurement | count | survey | document | other",
      "estimated_effort": "enum: low | medium | high",
      "priority": "int (1–5, where 1=most important)"
    }
  ],
  "criteria_confidence": "float 0.0–1.0",
  "user_agency_note": "string (Bahasa Indonesia, e.g., 'Anda dapat mengedit atau menambah kriteria di bawah')"
}
```

### 10.5 Task Decomposition Output

```json
{
  "suggested_tasks": [
    {
      "task_id": "string",
      "title": "string (e.g., 'Survei lokasi jalan yang rusak')",
      "description": "string (2–3 sentences)",
      "track_skill": "string (ESCO skill needed)",
      "estimated_duration_days": "int",
      "dependencies": ["array of task_ids this depends on"],
      "success_criteria": ["array of criterion_ids"],
      "suggested_assignee": "enum: anyone | expert_only | community_lead",
      "priority": "int (1–5)"
    }
  ],
  "total_estimated_duration_days": "int",
  "task_decomposition_confidence": "float 0.0–1.0"
}
```

### 10.6 Prompt Strategy

**System Prompt (Criteria):**

```
You are a community planning advisor for Gotong Royong.

User is planning how to achieve a community goal. Your task: Suggest 3–5 clear, measurable success criteria.

Guidelines:
- Each criterion should be specific and verifiable (not vague)
- Mix of quick wins (can verify in days) and long-term goals
- Consider the community context (rural vs. urban, size, resources)
- Prioritize criteria by importance
- Suggest verification method for each (visual, measured, counted, surveyed, etc.)

Output JSON with: suggested_criteria (array), criteria_confidence, user_agency_note.
```

**System Prompt (Task Decomposition):**

```
You are a project planning advisor for Gotong Royong.

User is planning tasks to achieve a community goal. Your task: Decompose into 5–10 actionable tasks.

Guidelines:
- Each task should take 1–14 days (not too small, not too big)
- Establish dependencies (Task B cannot start until Task A completes)
- Specify skills needed (reference ESCO codes where applicable)
- Suggest who can do it (anyone, expert, community lead)
- Be realistic about duration given community resources
- Early tasks should be quick wins that build momentum

Output JSON with: suggested_tasks (array), total_estimated_duration_days, task_decomposition_confidence.
```

### 10.7 Few-Shot Example

**Input:** "Kami ingin memperbaiki jalan rusak di Jl. Merdeka."

**Criteria Output:**
```json
{
  "suggested_criteria": [
    {
      "criterion_id": "crit-001",
      "text": "Jalan diaspal dengan lebar minimal 5 meter di sepanjang 200 meter",
      "measurable": true,
      "verification_method": "measurement",
      "estimated_effort": "high",
      "priority": 1
    },
    {
      "criterion_id": "crit-002",
      "text": "Tidak ada lubang lebih dari 10 cm di permukaan",
      "measurable": true,
      "verification_method": "visual_inspection",
      "estimated_effort": "low",
      "priority": 2
    },
    {
      "criterion_id": "crit-003",
      "text": "Saluran air tepi jalan berfungsi (tidak tergenang)",
      "measurable": true,
      "verification_method": "visual_inspection",
      "estimated_effort": "medium",
      "priority": 3
    }
  ],
  "criteria_confidence": 0.92,
  "user_agency_note": "Ini adalah saran awal. Anda dapat mengedit, menambah, atau mengurangi kriteria di bawah."
}
```

**Task Decomposition Output:**
```json
{
  "suggested_tasks": [
    {
      "task_id": "task-001",
      "title": "Survei lokasi dan dokumentasi kerusakan",
      "description": "Kunjungi Jl. Merdeka, ambil foto, ukur kerusakan, buat peta damage. Output: laporan foto & peta.",
      "track_skill": "S2.1.1",
      "estimated_duration_days": 2,
      "dependencies": [],
      "success_criteria": ["crit-001"],
      "suggested_assignee": "anyone",
      "priority": 1
    },
    {
      "task_id": "task-002",
      "title": "Hubungi kontraktor & dapatkan penawaran",
      "description": "Gunakan hasil survei untuk minta 3 penawaran dari kontraktor. Bandingkan harga & timeline.",
      "track_skill": "S4.1.2",
      "estimated_duration_days": 3,
      "dependencies": ["task-001"],
      "success_criteria": [],
      "suggested_assignee": "expert_only",
      "priority": 2
    },
    {
      "task_id": "task-003",
      "title": "Negosiasi dengan Pemerintah Lokal untuk dukungan",
      "description": "Hubungi Dinas PU, diskusikan bantuan anggaran atau resources. Buat MOU jika memungkinkan.",
      "track_skill": "S4.2.1",
      "estimated_duration_days": 5,
      "dependencies": ["task-001"],
      "success_criteria": [],
      "suggested_assignee": "community_lead",
      "priority": 1
    },
    {
      "task_id": "task-004",
      "title": "Mobilisasi fundraising atau galang dana komunitas",
      "description": "Jika pemerintah tidak sepenuhnya mendukung, galang dana dari komunitas via Galang. Target: Rp X juta.",
      "track_skill": "S3.3.2",
      "estimated_duration_days": 7,
      "dependencies": ["task-002"],
      "success_criteria": [],
      "suggested_assignee": "anyone",
      "priority": 2
    },
    {
      "task_id": "task-005",
      "title": "Pelaksanaan perbaikan jalan",
      "description": "Supervisi kontraktor. Pastikan sesuai spesifikasi. Daily checklist. Dokumentasi progress.",
      "track_skill": "S1.3.1",
      "estimated_duration_days": 10,
      "dependencies": ["task-003", "task-004"],
      "success_criteria": ["crit-001", "crit-002", "crit-003"],
      "suggested_assignee": "expert_only",
      "priority": 1
    },
    {
      "task_id": "task-006",
      "title": "Verifikasi dan Rayakan",
      "description": "Periksa hasil akhir sesuai kriteria. Minta tanda tangan kontraktor. Rayakan bersama komunitas.",
      "track_skill": "S2.2.1",
      "estimated_duration_days": 1,
      "dependencies": ["task-005"],
      "success_criteria": ["crit-001", "crit-002", "crit-003"],
      "suggested_assignee": "community_lead",
      "priority": 1
    }
  ],
  "total_estimated_duration_days": 28,
  "task_decomposition_confidence": 0.88
}
```

### 10.8 User Interaction & Persistence

- Suggestions are **non-binding** — user can edit, delete, or ignore
- User changes are saved to seed and persist
- If user modifies task after AI-06 suggestion, changes are marked as `user_edited=true`
- Community leads can use AI-06 suggestions as starting template for rapid planning

### 10.9 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Model unavailable** | Show empty suggestions; prompt user to create criteria/tasks manually |
| **Insufficient context** (<50 words in seed) | Show limited suggestions; ask user to clarify goal first |
| **Novel/unique goal** | Return generic suggestions; ask moderator to review |

---

