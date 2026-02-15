> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 18. Guardrail Metrics & Monitoring

All AI touch points are monitored for accuracy, safety, and community impact.

### 18.1 Metrics by Touch Point

| Touch Point | Metric | Target | Frequency |
|---|---|---|---|
| **AI-00 Triage** | Manual bypass rate | < 15% | Daily |
| **AI-00 Triage** | Avg turns to classification | 2–4 | Daily |
| **AI-00 Triage** | Vault detection accuracy | > 85% | Daily |
| **AI-00 Triage** | Siaga detection accuracy | > 95% | Daily |
| **AI-01 Classification** | Track accuracy (vs. human review) | > 80% | Weekly |
| **AI-01 Classification** | Seed type accuracy | > 75% | Weekly |
| **AI-02 Redaction** | False positive rate (over-redaction) | < 10% | Weekly |
| **AI-02 Redaction** | False negative rate (missed PII) | < 5% | Weekly |
| **AI-03 Duplicate** | Duplicate detection recall (catch real dupes) | > 80% | Weekly |
| **AI-03 Duplicate** | Duplicate detection precision (avoid false positives) | > 85% | Weekly |
| **AI-04 Moderation** | Policy violation detection accuracy | > 90% | Daily |
| **AI-04 Moderation** | False positive rate (wrongly blocked) | < 3% | Daily |
| **AI-04 Moderation** | Moderation hold appeal rate | < 5% | Weekly |
| **AI-05 Gaming** | Gaming pattern detection sensitivity | > 80% | Weekly |
| **AI-05 Gaming** | False positive rate (false alarm) | < 2% | Weekly |
| **AI-06 Criteria** | User adoption rate (uses AI suggestions) | > 60% | Weekly |
| **AI-06 Criteria** | Criteria usefulness (survey) | > 7/10 | Monthly |
| **AI-07 Summary** | Summary comprehensiveness (covers key points) | > 75% | Weekly |
| **AI-07 Summary** | Summary accuracy (no hallucinations) | > 95% | Weekly |
| **AI-08 Media** | Face detection accuracy | > 95% | Weekly |
| **AI-08 Media** | License plate detection accuracy | > 90% | Weekly |
| **AI-09 Credit** | Credit dispute rate | < 5% | Weekly |
| **AI-09 Credit** | PIC acceptance rate of Tuntas distribution | > 85% | Weekly |
| **AI-09 Credit** | Dispute resolution time | < 72h | Weekly |

### 18.2 Monitoring Dashboard

A real-time dashboard displays all metrics, accessible to:
- AI Lead (daily review)
- Community Lead (weekly review)
- Moderators (gaming/moderation metrics)
- Data team (weekly report)

### 18.3 Alert Thresholds

| Metric | Alert Threshold | Action |
|---|---|---|
| Any metric > 10% above target | Yellow alert | Review & investigate |
| Any metric > 20% above target | Red alert | Pause feature; post-mortem |
| Moderation false positive > 5% | Red alert | Retrain classifier; emergency review all held seeds |
| Gaming detection < 50% recall | Yellow alert | Expand baselines; investigate missed patterns |

---

