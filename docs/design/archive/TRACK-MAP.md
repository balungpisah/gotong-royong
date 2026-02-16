> **[ARCHIVED 2026-02-15]** Fixed track lifecycles superseded by Adaptive Path Guidance. See `design/context/ADAPTIVE-PATH-MAP.md` and `design/specs/ADAPTIVE-PATH-SPEC-v0.1.md`.

# Gotong Royong — Track Map

> ASCII reference for all 5 track lifecycles. Optional stages in `[ ]`, cross-cutting in `{ }`.

---

## TUNTASKAN (Keresahan → Tuntas)

```
                          ┌──────────┐
                          │ Keresahan│
                          └────┬─────┘
                               │
                          ┌────▼─────┐
                          │  Bahas   │ ◄── diagnostic: simple or complex?
                          └────┬─────┘
                               │
                 ┌─────────────┼─────────────┐
                 │ simple      │ complex      │
                 │             ▼              │
                 │       ┌──────────┐         │
                 │       │ Rancang  │         │
                 │       └────┬─────┘         │
                 │            │               │
                 │      ┌─────┼─────┐         │
                 │      │ optional  │         │
                 │      ▼           ▼         │
                 │  {Galang}   {Siarkan}      │
                 │      │           │         │
                 │      └─────┬─────┘         │
                 │            │               │
                 └────────────┤               │
                              │               │
                         ┌────▼─────┐         │
                         │  Garap   │ ◄───────┘
                         └────┬─────┘
                              │
                         ┌────▼─────┐
                         │ Periksa  │
                         └────┬─────┘
                              │
                        [┌────▼─────┐]
                        [│  Dampak  │] ◄── optional: significant cases
                        [└────┬─────┘]
                              │
                         ┌────▼─────┐
                         │  Tuntas  │
                         └──────────┘
```

**Garap sub-states:** Active → Stalled → Released


---

## WUJUDKAN (Gagasan → Tuntas)

```
                          ┌──────────┐
                          │ Gagasan  │
                          └────┬─────┘
                               │
                          ┌────▼─────┐
                          │  Bahas   │
                          └────┬─────┘
                               │
                          ┌────▼─────┐
                          │ Rancang  │
                          └────┬─────┘
                               │
                         [┌────▼─────┐]
                         [│ {Galang} │] ◄── optional
                         [└────┬─────┘]
                               │
                          ┌────▼─────┐
                          │  Garap   │
                          └────┬─────┘
                               │
                          ┌────▼─────┐
                          │ Rayakan  │ ◄── celebrates creation (replaces Periksa)
                          └────┬─────┘
                               │
                         [┌────▼─────┐]
                         [│  Dampak  │] ◄── optional
                         [└────┬─────┘]
                               │
                          ┌────▼─────┐
                          │  Tuntas  │
                          └──────────┘
```


---

## TELUSURI (Pertanyaan → Tuntas)

```
                          ┌───────────┐
                          │Pertanyaan │
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │  Dugaan   │ ◄── candidate explanations
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │    Uji    │ ◄── research / observe / test
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │  Temuan   │ ◄── best answer + confidence
                          └─────┬─────┘
                                │
                 ┌──────────────┼──────────────┐
                 │              │              │
          problem found    idea emerged    knowledge
                 │              │              │
                 ▼              ▼              ▼
           → Tuntaskan    → Wujudkan       Tuntas
           (track change) (track change)
```


---

## RAYAKAN (Kabar Baik → Tuntas)

```
                          ┌───────────┐
                          │ Kabar Baik│
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │  Sahkan   │ ◄── community validates
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │ Apresiasi │ ◄── public recognition
                          └─────┬─────┘
                                │
                         [┌─────▼─────┐]
                         [│  Dampak   │] ◄── optional
                         [└─────┬─────┘]
                                │
                          ┌─────▼─────┐
                          │  Tuntas   │
                          └───────────┘
```


---

## MUSYAWARAH (Usul → Tuntas)

```
                          ┌───────────┐
                          │   Usul    │
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │   Bahas   │ ◄── deliberate, present options
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │ Putuskan  │ ◄── community votes
                          └─────┬─────┘
                                │
                          ┌─────▼─────┐
                          │ Jalankan  │ ◄── implement decision
                          └─────┬─────┘
                                │
                         [┌─────▼─────┐]
                         [│  Tinjau   │] ◄── optional: review effectiveness
                         [└─────┬─────┘]
                                │
                          ┌─────▼─────┐
                          │  Tuntas   │
                          └───────────┘
```

**Output:** Ketetapan (decision/ruling)


---

## CROSS-CUTTING: GALANG (sub-lifecycle when activated)

```
    Sasaran → Kumpul → Salurkan → Lapor
    (target)  (collect) (distribute) (report)
```


## CROSS-CUTTING: SIARKAN (capability, no lifecycle)

```
    Reach tracker · Share actions · Media kit · Humas role
```


---

## TRANSITION RISK TABLE

| Transition         | Risk   | Mechanism                    | Window |
|--------------------|--------|------------------------------|--------|
| Seed → Bahas       | Low    | Consent (auto unless object) | 24h    |
| Bahas → Rancang    | Medium | Standard vote (quorum)       | 48h    |
| Rancang → Garap    | Medium | Standard vote                | 48h    |
| Garap → Periksa    | High   | Full vote + evidence review  | 72h    |
| Periksa → Tuntas (+ Dampak opsional) | High | Verification + challenge | 72h |
| Emergency          | Crit   | Fast-track + 7d post-hoc     | Now    |


---

*All stages reversible via Jury (dispute). Every transition opens a challenge window.*
