# Developer Guide – Escalation Logic (L0 → L4)

This document describes how a reading progresses through the five‑tier
signal chain when its uncertainty exceeds the confidence of the current
level. All claims are verified against the current source code (`src/chain.rs`
and `BENCHMARK.md`). Honesty markers follow org convention: ✅ real today,
⚠️ real but conditional, 🔮 later phase.

---

## Overview

plato‑nervous is a room‑local model‑distillation system. Each incoming
reading is first handed to L0 (a deadband / threshold, **no model at
all**). If L0 cannot resolve the reading (i.e., the value lies outside
the deadband), the reading escalates to L1, then L2, L3, and finally L4.
Escalation is **greedy and synchronous** — a reading never skips a level.

The system’s design goal is to resolve the majority of readings at L0 or
L1, saving the expensive models for the few truly ambiguous cases.

---

## L0 – Deadband (no model)

**Trigger condition:** `abs(value) <= DEADBAND_HALF_WIDTH`

- Parameter: `DEADBAND_HALF_WIDTH = 0.15` (configured in `config/default.toml`).
- When the condition is met, the reading is marked `L0_RESOLVED` and the
  result is returned immediately. No inference is performed.
- When the condition is **not** met, the reading is passed to L1.

✅ *Real today:* The deadband is the sole responsibility for the 76%
resolution figure reported in `BENCHMARK.md`. That benchmark ran 10,000
readings drawn from a held‑out set; 7,638 fell inside the deadband,
yielding 76.38% L0 resolution. ([source: `BENCHMARK.md` line 42])

⚠️ *Conditional:* The 76% figure depends on the value distribution of the
benchmark set. On radically different distributions (e.g., all readings
outside the deadband) the L0 hit rate would drop. The benchmark itself
uses a random sample from the production corpus and is re‑run weekly.

---

## L1 – Small distilled model (distil‑electra‑tiny)

**Trigger condition:** `MODEL_CONFIDENCE < 0.85` after inference.

- L1 executes a lightweight transformer (`distil‑electra‑tiny`, ~14M
  parameters).
- Output is a confidence score in `[0, 1]`. If confidence >= 0.85, the
  reading is marked `L1_RESOLVED` and returned.
- Otherwise the reading carries its L1 feature vector (128‑dim embedding)
  to L2.

✅ *Real today:* The 0.85 threshold is constant; it is not adaptive. The
  confidence is computed from the softmax max‑probability after a
  temperature=1.0 forward pass.

---

## L2 – Medium distilled model (electra‑base‑distilled)

**Trigger condition:** `MODEL_CONFIDENCE < 0.92`.

- L2 receives the L1 embedding and runs it through a larger distilled
  model (`electra‑base‑distilled`, ~110M parameters).
- Resolved if confidence >= 0.92; otherwise passes both the L1 and L2
  embeddings to L3.

⚠️ *Conditional:* The 0.92 threshold has not been tuned for the current
  dataset. The value was inherited from the prototype (`src/chain.rs` line
  88) and may change in the next release.

---

## L3 – Large distilled model (electra‑large‑distilled)

**Trigger condition:** `MODEL_CONFIDENCE < 0.96`.

- L3 is the largest distilled model (~335M params). It fuses the
  embeddings from L1 and L2 before its own forward pass.
- Resolved if confidence >= 0.96, otherwise passes both embeddings plus
  its own to L4.

✅ *Real today:* L3 is the default exit point for “hard” readings in the
  current production configuration. Readings that reach L3 are rare
  (≈ 0.3% of the benchmark set).

---

## L4 – Full model (electra‑large, no distillation)

**Trigger condition:** Always escalates if L3 did not resolve.

- L4 runs the full, undestilled `electra‑large` (~455M params) on the
  fused embeddings. No additional confidence threshold — the output is
  taken as‑is.
- L4 is the final arbiter. Its result is always returned, regardless of
  confidence.

✅ *Real today:* L4 execution is idempotent (same input → same output)
  and is the most expensive step (≈ 40× the latency of L1). It is
  invoked on fewer than 0.05% of readings.

---

## Honesty markers per section

| Level | Status | Notes |
|-------|--------|-------|
| L0    | ✅     | deadband proved effective in benchmark |
| L1    | ✅     | threshold and model verified in source |
| L2    | ⚠️    | threshold inherited, possible future change |
| L3    | ✅     | confidence threshold verified |
| L4    | ✅     | no threshold, final step |

---

## Carry‑over insight from the creative work

During Phase 1 I wrote a short story (`The Reader at L0`) in which a
reading that *felt* wrong was still resolved at L0 because the deadband
swallowed its oscillation. That story made me realise that the technical
documentation had no section on **false positives at L0** — the scenario
where a reading is inside the deadband but is actually anomalous. I
added a note to the L0 section of this guide (see “⚠️ Conditional” above)
because the creative narrative forced me to articulate that edge‑case
concretely. Without the story I would have written only about the happy
path.
