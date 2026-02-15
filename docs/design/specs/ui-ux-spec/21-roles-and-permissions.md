> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 20. Roles & Permissions Lifecycle

### 20.1 Role Definitions

| Role | Indonesian | Assignment | Requirements |
|---|---|---|---|
| Author | Penulis | Automatic | Any user |
| Co-witness | Saksi | Self-assign | Any user |
| Participant | Peserta | Self-assign | Any user |
| Responsibility Owner | PIC | Bahas → vote | Terverifikasi |
| Treasurer | Bendahara | Galang → vote | I >= 0.5, clean J, ≠ PIC |
| Communications | Humas | Siarkan → vote | Terverifikasi |
| Verifier | Verifikator | Auto-selected | I >= 0.4, not involved, min 3 |
| Reviewer | Peninjau | Auto-selected | I >= 0.4, impartial |
| Jury | Juri | Stochastic | I >= 0.4, verified, not involved |

### 20.2 Permission Matrix

| Action | Penulis | Saksi | Peserta | PIC | Bendahara | Humas | Verifikator |
|---|---|---|---|---|---|---|---|
| Edit seed (15min) | ✓ | | | | | | |
| Co-witness | | ✓ | | | | | |
| Comment/discuss | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Propose transition | ✓ | | | ✓ | | | |
| Vote on transition | ✓ | ✓ | ✓ | ✓ | | | |
| Assign tasks | | | | ✓ | | | |
| Claim tasks | | | ✓ | ✓ | | | |
| Submit heartbeat | | | ✓ | ✓ | | | |
| Manage funds | | | | | ✓ | | |
| Approve disbursement | | | | ✓ | ✓ | | |
| Broadcast/share | | | | | | ✓ | |
| Verify completion | | | | | | | ✓ |
| Challenge result | ✓ | ✓ | ✓ | ✓ | | | |
| Set Rahasia level | ✓ | | | ✓ | | | |
| File dispute | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

---

