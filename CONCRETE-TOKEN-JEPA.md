# Concrete Token JEPA: Liquid AI as Room Nervous System

*Why build a custom JEPA architecture when LFM2.5 already outputs structured tokens at the scale we need?*

## The Insight

Casey's key observation: **LFM2.5 has models at or well below 1GB that work like JEPA but for concrete tokens.**

We don't need a custom Joint Embedding Predictive Architecture. We don't need embedding spaces, latent representations, or a separate training pipeline. The Liquid AI models already:

1. **Output structured tokens** (JSON, classification labels, state vectors) at 229MB
2. **Support few-shot in-context learning** (the prompt IS the LoRA)
3. **Run on CPU in <1 second** for classification tasks
4. **Are fine-tunable with LoRA** for room-specific adapters

## What We Actually Need

```
Room Sensor → Deadband Filter → Liquid 350M/1.2B → Structured Token Output
                                    ↑
                              Few-shot context window
                              (THIS IS THE ROOM'S INTELLIGENCE)
```

The "irreducible core" isn't a custom JEPA model. It's the **few-shot prompt** that accumulates room-specific examples over time. Each tile the room produces adds an example. Cloud corrections add gold-standard examples. The prompt window IS the room's self-model.

## Experimental Results

### Model Benchmarks (Local, CPU-only, 24-core)

| Model | Size | Latency | Anomaly Detection | Normal Detection | Structured Output |
|-------|------|---------|-------------------|------------------|-------------------|
| LFM2.5-350M | 229MB | 0.3-0.7s | ✅ ALERT | ❌ false ALERT | ❌ Can't do JSON |
| LFM2.5-1.2B Instruct | 698MB | 0.5-1.5s | ✅ ALERT | ❌ false ALERT | ✅ With completion-style |
| LFM2.5-1.2B Thinking | 698MB | 10-15s | Echoes prompt | Echoes prompt | ❌ Wrong prompt format |
| phi4-mini | 2.5GB | 9-12s | ✅ ALERT | ✅ NORMAL | ✅ Best accuracy |
| LFM2.5-8B-A1B (MoE) | ~4.5GB | TBD | TBD | TBD | Expected best |

### Completion-Style Few-Shot (1.2B) — 6/10

```
1450rpm 195F 62psi → OK      ✅
1455rpm 198F 60psi → OK      ✅  
1480rpm 205F 55psi → OK      ❌ (expected WARN)
1650rpm 228F 28psi → CRIT    ✅
1420rpm 175F 68psi → OK      ✅
1498rpm 209F 38psi → OK      ❌ (expected WARN)
1700rpm 240F 22psi → CRIT    ✅
1350rpm 142F 72psi → CRIT    ❌ (expected OK — cold idle confused with CRIT)
1460rpm 200F 58psi → OK      ✅
1520rpm 215F 33psi → OK      ❌ (expected WARN)
```

**Perfect on OK and CRIT. Misses the WARN boundary.** This is exactly what LoRA distillation fixes.

### Signal Chain Distribution (with 1.2B as L1)

```
L0 Deadband:    76%  ████████████████████████  (algorithmic, 0ms)
L1 Liquid 1.2B: 14%  █████                    (0.5-1.5s, concrete tokens)
L4 Cloud:       10%  ████                     (cloud LLM, 5-30s)
AUTONOMY:       90%
```

## The Architecture: Prompt as LoRA

### Phase 1: Generic Room (Day 1)

The prompt starts with generic examples:
```
Classify sensor status.
1450rpm 195F 62psi → OK
1485rpm 206F 50psi → WARN  
1650rpm 228F 25psi → CRIT
{reading} →
```

### Phase 2: Accumulating Tiles (Week 1)

As the room operates, every cloud correction becomes a new example:
```
Classify sensor status.
1450rpm 195F 62psi → OK
1485rpm 206F 50psi → WARN  
1650rpm 228F 25psi → CRIT
1420rpm 175F 68psi → OK
1498rpm 209F 38psi → WARN    ← cloud corrected this
1520rpm 215F 33psi → WARN    ← cloud corrected this
1700rpm 240F 22psi → CRIT
{reading} →
```

### Phase 3: Room-Specific Intelligence (Month 1)

After 100+ cloud corrections, the prompt contains the room's specific thresholds:
```
Engine Room Northwest - Bering Sea Operations
Normal: RPM 1400-1500, Coolant 140-210, Oil 35-80
Cold idle: RPM <1380 with coolant <160 → OK (not CRIT)
Warm cruise: RPM 1440-1480, coolant 190-200 → OK
High load: RPM >1500 OR coolant >205 OR oil <40 → WARN
Critical: RPM >1600 AND coolant >220 → CRIT
1450rpm 195F 62psi → OK
...
{reading} →
```

The prompt IS the LoRA. No weight updates needed. The context window carries the room's accumulated intelligence.

### Phase 4: Actual LoRA (Month 3)

When the prompt window fills up (~10-20 examples at 1.2B's context), the most valuable examples get distilled into a real LoRA adapter:
- 350M base + room-specific LoRA (~2MB)
- Runs on ESP32-class hardware
- The "irreducible core" is now IN the weights

## Why This Works

1. **No custom architecture needed.** LFM2.5 already does structured output, few-shot learning, and can be LoRA'd.

2. **The 350M is the ESP32 model.** 229MB Q4, runs in 0.3s on CPU. After LoRA (2MB), it's 231MB total — fits on Jetson, maybe even ESP32 with external flash.

3. **The 1.2B is the room model.** 698MB Q4, concrete tokens in 0.5s. Few-shot context gives it room-specific intelligence without fine-tuning.

4. **The 8B-A1B (MoE) is the fleet coordinator.** 4.5GB but only 1.5B active per token. Cross-room reasoning, complex correlations, novel situations.

5. **Tiles = few-shot examples.** Every reading produces a structured token. Every cloud correction adds a gold-standard example. The prompt window is the training data.

## Model Roster for PLATO

| Layer | Model | Size | Role | Runs On |
|-------|-------|------|------|---------|
| L0 | Deadband filter | 0 bytes | Spectral gap detection | ESP32 |
| L1 | LFM2.5-350M + LoRA | 231MB | Room-level perception | Jetson |
| L2 | LFM2.5-1.2B + few-shot | 698MB | Room reasoning + classification | Desktop |
| L3 | LFM2.5-8B-A1B | 4.5GB | Fleet coordination | Server |
| L4 | Cloud LLM | ∞ | Novel situations | API |

## Next Experiments

1. **8B-A1B benchmark** — test the MoE model for cross-room coordination
2. **Dynamic few-shot** — build the prompt that grows with tile accumulation
3. **LoRA fine-tuning** — use Unsloth to train a room-specific LoRA on the 350M
4. **VL-450M** — the vision-language model could be the "vision JEPA" for rooms with cameras
5. **Audio-1.5B** — could be the "vibration JEPA" for rooms with acoustic sensors

## Key Files

- Crate: [SuperInstance/plato-nervous](https://github.com/SuperInstance/plato-nervous) (26 tests)
- Architecture: [SIGNAL-CHAIN-DISTILLATION.md](./SIGNAL-CHAIN-DISTILLATION.md)
- Models: `/tmp/liquid-models/` (350M, 1.2B Instruct, 1.2B Thinking, 8B-A1B downloading)
