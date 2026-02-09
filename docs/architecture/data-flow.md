# Data Flow

## Overview

This document describes the key data flows through the Gotong Royong platform, from task creation to reputation updates.

## 1. Task Creation and Assignment Flow

```mermaid
sequenceDiagram
    participant Organizer
    participant API
    participant DB
    participant Cache

    Organizer->>API: POST /tasks (create new task)
    API->>API: Validate task data
    API->>DB: INSERT INTO tasks
    DB->>API: task_id
    API->>Cache: Invalidate task list cache
    API->>Organizer: 201 Created (task_id)

    Organizer->>API: POST /tasks/{id}/assign (assign to contributor)
    API->>DB: UPDATE tasks SET assignee_id = ?
    API->>DB: INSERT INTO task_assignments
    DB->>API: Success
    API->>Organizer: 200 OK

    Note over DB: Contributor sees assigned task in their dashboard
```

**Steps**:
1. Task organizer creates task with title, description, skills required
2. System validates input and stores in database
3. Task list cache is invalidated
4. Organizer assigns task to specific contributor or leaves open
5. Contributor sees task in their assigned tasks list

**Database Tables Used**:
- `tasks` - Main task record
- `task_skills` - Skills required for task
- `task_assignments` - Assignment history

## 2. Task Completion with Evidence Submission Flow

```mermaid
sequenceDiagram
    participant Contributor
    participant API
    participant S3
    participant DB
    participant Queue
    participant Worker
    participant Markov

    Contributor->>API: POST /tasks/{id}/complete (with evidence files)
    API->>API: Validate completion data
    API->>S3: Generate presigned upload URLs
    S3->>API: Upload URLs
    API->>Contributor: Return upload URLs

    Contributor->>S3: Upload evidence files (photos, GPS logs)
    S3->>Contributor: Upload complete

    Contributor->>API: POST /tasks/{id}/evidence/confirm
    API->>API: Compute media hashes
    API->>DB: BEGIN TRANSACTION
    API->>DB: INSERT INTO contributions
    API->>DB: INSERT INTO evidence
    API->>DB: UPDATE tasks SET status = 'completed'
    API->>DB: COMMIT

    API->>Queue: Enqueue webhook (contribution_created)
    API->>Contributor: 200 OK (contribution recorded)

    Worker->>Queue: Dequeue webhook job
    Worker->>Worker: Generate HMAC signature
    Worker->>Markov: POST /webhook (contribution_created)
    Markov->>Worker: 200 OK (processed)
    Worker->>DB: Log webhook delivery
```

**Steps**:
1. Contributor marks task as complete and uploads evidence
2. API generates presigned S3 URLs for direct upload
3. Contributor uploads files directly to S3
4. API confirms upload, computes media hashes
5. Database transaction: Create contribution, store evidence, mark task complete
6. Webhook event queued for Markov Engine (async)
7. Background worker delivers webhook with HMAC signature
8. Markov Engine processes event and updates reputation

**Database Tables Used**:
- `contributions` - Contribution record
- `evidence` - Evidence metadata
- `tasks` - Task status update
- `webhook_events` - Event queue

## 3. Proof of Reality (PoR) Evidence Validation Flow

```mermaid
sequenceDiagram
    participant Contributor
    participant API
    participant Validator
    participant DB
    participant Queue
    participant Markov

    Contributor->>API: POST /evidence (submit PoR evidence)
    API->>Validator: Validate evidence structure

    alt Photo with Timestamp
        Validator->>Validator: Check timestamp <= 30 days
        Validator->>Validator: Validate media_hash (hex, 32+ chars)
        Validator->>Validator: Extract EXIF metadata
    else GPS Verification
        Validator->>Validator: Check timestamp <= 30 days
        Validator->>Validator: Validate lat (-90 to 90)
        Validator->>Validator: Validate lon (-180 to 180)
    else Witness Attestation
        Validator->>Validator: Check timestamp <= 30 days
        Validator->>Validator: Validate witnesses array (min 1)
    end

    alt Validation Success
        Validator->>API: Evidence valid
        API->>DB: INSERT INTO evidence
        API->>Queue: Enqueue webhook (por_evidence)
        API->>Contributor: 200 OK (evidence accepted)
        Queue->>Markov: Webhook delivery
        Markov->>Markov: Store evidence, update verification
    else Validation Failure
        Validator->>API: Error (reason)
        API->>Contributor: 400 Bad Request (validation error)
    end
```

**Validation Rules**:
- **Timestamp**: Must be <= 30 days old
- **Media Hash**: Hex string, minimum 32 characters
- **GPS Coordinates**: Lat (-90, 90), Lon (-180, 180)
- **Witnesses**: At least 1 witness required for attestation

**Error Examples**:
- `"Timestamp is too old: 45 days"`
- `"Invalid latitude: 95.0. Must be between -90 and 90"`
- `"witness_attestation requires at least one witness"`

See [Validation Rules](../por-evidence/validation-rules.md) for complete specifications.

## 4. Vouch Submission Flow

```mermaid
sequenceDiagram
    participant Voucher
    participant API
    participant DB
    participant Queue
    participant Markov

    Voucher->>API: POST /vouch (vouch for another contributor)
    API->>API: Validate voucher has reputation > min threshold
    API->>API: Validate vouchee exists
    API->>API: Check no duplicate vouch (voucher → vouchee, skill)

    API->>DB: INSERT INTO vouches
    DB->>API: vouch_id

    API->>Queue: Enqueue webhook (vouch_submitted)
    API->>Voucher: 200 OK (vouch recorded)

    Queue->>Markov: POST /webhook (vouch_submitted)
    Markov->>Markov: Calculate vouch weight
    Markov->>Markov: Update reputation graph
    Markov->>Queue: 200 OK
```

**Steps**:
1. Voucher submits endorsement for another contributor
2. System validates voucher eligibility (minimum reputation)
3. Check for duplicate vouches (same skill)
4. Store vouch in database
5. Queue webhook for Markov Engine
6. Markov calculates vouch impact on reputation

**Vouch Weight Calculation** (in Markov Engine):
- Based on voucher's own reputation
- Skill-specific weights
- Decay over time (older vouches have less weight)

**Database Tables Used**:
- `vouches` - Vouch records
- `users` - Voucher/vouchee profiles

## 5. Reputation Query Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant Cache
    participant Markov

    User->>API: GET /users/{id}/reputation
    API->>Cache: Check Redis cache

    alt Cache Hit
        Cache->>API: Cached reputation data
        API->>User: 200 OK (reputation)
    else Cache Miss
        API->>Markov: GET /users/{markov_id}/reputation
        Markov->>API: Reputation data (score, tier, breakdown)
        API->>Cache: Store in Redis (TTL: 5 min)
        API->>User: 200 OK (reputation)
    end
```

**Caching Strategy**:
- **TTL**: 5 minutes
- **Invalidation**: Optional on webhook success
- **Key**: `reputation:{user_id}`

**Reputation Data Structure**:
```json
{
  "user_id": "user123",
  "reputation_score": 2550,
  "tier": "advanced",
  "contributions_count": 51,
  "vouches_received": 12,
  "skills": [
    { "skill_id": "gardening", "competence": 0.85 },
    { "skill_id": "logistics", "competence": 0.72 }
  ]
}
```

## 6. Verification Consensus Flow

```mermaid
sequenceDiagram
    participant Verifier1
    participant Verifier2
    participant Verifier3
    participant API
    participant DB
    participant Markov

    Note over Verifier1,Verifier3: Multiple verifiers review same contribution

    Verifier1->>API: POST /contributions/{id}/verify (approve)
    API->>DB: INSERT INTO verifications (outcome: approved)
    API->>Markov: Webhook (verification)

    Verifier2->>API: POST /contributions/{id}/verify (approve)
    API->>DB: INSERT INTO verifications (outcome: approved)
    API->>Markov: Webhook (verification)

    Verifier3->>API: POST /contributions/{id}/verify (rejected)
    API->>DB: INSERT INTO verifications (outcome: rejected)
    API->>Markov: Webhook (verification)

    Note over Markov: Calculate consensus (2/3 approve = approved)
    Markov->>Markov: Update contribution verification_status
    Markov->>Markov: Calculate reputation delta
```

**Consensus Logic** (in Markov Engine):
- Require minimum N verifiers (configurable, default: 3)
- Majority vote determines outcome
- Weight verifications by verifier reputation
- Example: 2 approve + 1 reject = approved (if both approvers have higher reputation)

**Database Tables Used**:
- `verifications` - Individual verification records
- `contributions` - Aggregate verification status

## 7. Task Discovery and Browsing Flow

```mermaid
sequenceDiagram
    participant Contributor
    participant API
    participant Cache
    participant DB

    Contributor->>API: GET /tasks?status=open&skill=gardening
    API->>Cache: Check cached task list

    alt Cache Hit
        Cache->>API: Cached task list
    else Cache Miss
        API->>DB: SELECT tasks WHERE status = 'open' AND skill = 'gardening'
        DB->>API: Task list
        API->>Cache: Store in Redis (TTL: 1 min)
    end

    API->>Contributor: 200 OK (task list with metadata)

    Contributor->>API: GET /tasks/{id}
    API->>DB: SELECT task details + evidence + verifications
    DB->>API: Full task data
    API->>Contributor: 200 OK (task details)

    Contributor->>API: POST /tasks/{id}/claim
    API->>DB: UPDATE tasks SET assignee_id = contributor_id
    API->>Cache: Invalidate task list cache
    API->>Contributor: 200 OK (task claimed)
```

**Task List Caching**:
- **TTL**: 1 minute (short, because tasks change frequently)
- **Invalidation**: On task creation, assignment, completion
- **Segmentation**: Cache by filters (status, skill, location)

## 8. Multi-Perspective Evidence Submission Flow

```mermaid
sequenceDiagram
    participant Contributor
    participant Witness1
    participant Witness2
    participant API
    participant DB
    participant Markov

    Note over Contributor,Witness2: Same contribution, multiple evidence sources

    Contributor->>API: POST /evidence (photo_with_timestamp)
    API->>DB: INSERT INTO evidence (contribution_id, type: photo)
    API->>Markov: Webhook (por_evidence)

    Witness1->>API: POST /evidence (witness_attestation)
    API->>DB: INSERT INTO evidence (contribution_id, type: witness)
    API->>Markov: Webhook (por_evidence)

    Witness2->>API: POST /evidence (gps_verification)
    API->>DB: INSERT INTO evidence (contribution_id, type: gps)
    API->>Markov: Webhook (por_evidence)

    Note over Markov: Multiple evidence types increase confidence
    Markov->>Markov: Calculate evidence_quality_score
    Markov->>Markov: Apply bonus to reputation
```

**Multi-Perspective Benefits**:
- Higher reputation bonus for multiple evidence types
- Increased tamper resistance
- Better audit trail

**Evidence Quality Score**:
- Single evidence type: 1.0x multiplier
- Two evidence types: 1.2x multiplier
- Three evidence types: 1.5x multiplier

## 9. Webhook Retry and Dead Letter Flow

```mermaid
sequenceDiagram
    participant Queue
    participant Worker
    participant Markov
    participant DB
    participant Alert

    Queue->>Worker: Dequeue webhook job (attempt 1)
    Worker->>Markov: POST /webhook
    Markov--xWorker: 503 Service Unavailable
    Worker->>Queue: Retry (backoff: 1s)

    Queue->>Worker: Dequeue webhook job (attempt 2)
    Worker->>Markov: POST /webhook
    Markov--xWorker: Timeout (10s)
    Worker->>Queue: Retry (backoff: 2s)

    Queue->>Worker: Dequeue webhook job (attempt 3)
    Worker->>Markov: POST /webhook
    Markov->>Worker: 200 OK
    Worker->>DB: Log success
    Worker->>Queue: Mark complete

    Note over Queue,Alert: Alternative failure path

    Queue->>Worker: Attempt 5 (final retry)
    Worker->>Markov: POST /webhook
    Markov--xWorker: Connection refused
    Worker->>DB: INSERT INTO webhook_failures
    Worker->>Alert: Send alert (dead letter queue)
    Alert->>Alert: Notify ops team
```

**Retry Schedule**:
| Attempt | Delay | Action |
|---------|-------|--------|
| 1 | 0s | Immediate |
| 2 | 1s | First retry |
| 3 | 2s | Second retry |
| 4 | 4s | Third retry |
| 5 | 8s | Final retry |
| Failed | - | Move to DLQ |

## 10. Complete End-to-End Flow

**Scenario**: Contributor completes a community gardening task with photo evidence and receives peer vouch

```mermaid
sequenceDiagram
    participant C as Contributor (Alice)
    participant V as Voucher (Bob)
    participant API
    participant S3
    participant DB
    participant Queue
    participant Markov

    Note over C,Markov: Day 1: Task Completion

    C->>API: POST /tasks/garden-001/complete (with photo)
    API->>S3: Upload photo evidence
    API->>DB: Store contribution + evidence
    API->>Queue: Enqueue (contribution_created)
    API->>C: 200 OK (contribution recorded)

    Queue->>Markov: Webhook: contribution_created
    Markov->>Markov: +50 reputation (base task)

    Note over C,Markov: Day 2: Evidence Verification

    C->>API: POST /evidence (photo_with_timestamp)
    API->>API: Validate timestamp, media_hash
    API->>DB: Store PoR evidence
    API->>Queue: Enqueue (por_evidence)
    API->>C: 200 OK (evidence verified)

    Queue->>Markov: Webhook: por_evidence
    Markov->>Markov: +20 reputation (PoR bonus)

    Note over C,Markov: Day 3: Peer Vouch

    V->>API: POST /vouch (for Alice, skill: gardening)
    API->>DB: Store vouch
    API->>Queue: Enqueue (vouch_submitted)
    API->>V: 200 OK (vouch recorded)

    Queue->>Markov: Webhook: vouch_submitted
    Markov->>Markov: +15 reputation (weighted by Bob's rep)

    Note over C,Markov: Day 4: Reputation Query

    C->>API: GET /users/alice/reputation
    API->>Markov: GET /reputation (cache miss)
    Markov->>API: reputation: 2635 (was 2550)
    API->>C: Display updated reputation
```

**Reputation Timeline**:
- Initial: 2550
- After task: 2550 + 50 = 2600
- After PoR evidence: 2600 + 20 = 2620
- After vouch: 2620 + 15 = 2635

## Performance Considerations

### Bottlenecks

1. **Evidence Upload**: Large files (photos, videos)
   - **Solution**: Direct S3 upload with presigned URLs

2. **Webhook Delivery**: Synchronous blocking
   - **Solution**: Background job queue (async)

3. **Reputation Queries**: Frequent API calls to Markov
   - **Solution**: Redis caching with 5-minute TTL

### Optimization Strategies

- **Database Indexing**: Index on `user_id`, `task_id`, `status`, `created_at`
- **Connection Pooling**: Reuse HTTP connections to Markov
- **Batch Webhooks**: Send multiple events in single request (future)
- **CDN**: Serve static evidence files from CDN

## Next Steps

For implementation details:
- [Webhook Specification](../api/webhook-spec.md)
- [Event Payloads](../api/event-payloads.md)
- [Database Schema](../database/schema-requirements.md)
- [Evidence Validation](../por-evidence/validation-rules.md)
