# PLATO Signal Chain — Model Benchmark Results

**Date:** 2026-05-29
**Hardware:** RTX 4050 6GB VRAM, 15GB RAM, 24 cores (no GPU offload for large models)

## Models Tested

| Model | Size | Type | Notes |
|-------|------|------|-------|
| liquid-350m | 229MB | LFM2.5 base | Too small for chat, needs completion format |
| liquid-1.2b | 698MB | LFM2.5 instruct | Requires `<\|prompt\|>` format, not chat |
| phi4-mini | 2.5GB | Microsoft | Verbose, explains instead of classifying |
| qwen3:4b | 2.5GB | Alibaba | Thinking model, answer in think tokens |
| gemma4:e2b | 7.2GB | Google MoE | Thinking model, 2B total/500M active |
| gemma4:e4b | 9.6GB | Google MoE | Thinking model, 4B total/1B active |

## Chat API Results

### Sensor Classification (expected: CRIT for anomaly, OK for normal, WARN for borderline)

| Model | Anomaly (CRIT) | Normal (OK) | Borderline (WARN) | Avg Latency |
|-------|----------------|-------------|-------------------|-------------|
| liquid-350m | ❌ empty | ❌ gibberish | ❌ empty | 0.6s |
| liquid-1.2b | ❌ empty | ❌ empty | ❌ empty | 0.6s |
| phi4-mini | ❌ OK | ✅ OK | ❌ OK (verbose) | 1.4s |
| qwen3:4b | ❌ OK (think) | ✅ OK (think) | ❌ OK (think) | 3.4s |
| gemma4:e2b | ❌ OK (think) | ✅ OK (think) | ❌ OK (think) | 11.0s |
| gemma4:e4b | ❌ empty (think) | ❌ empty (think) | ❌ OK | 8.5s |

**Verdict: No model correctly identified the critical anomaly via chat API.**

### Few-shot Completion (expected: WARN)

| Model | Result |
|-------|--------|
| liquid-350m | ❌ OK |
| liquid-1.2b | ❌ OK (then hallucinated) |
| phi4-mini | ❌ Verbose explanation |
| qwen3:4b | ❌ Think only |
| gemma4:e2b | ❌ Think only |
| gemma4:e4b | ❌ Think only |

## Key Findings

### 1. Chat API is the wrong interface for small models
The Liquid models (350M, 1.2B) were designed for completion, not chat. They need the `<|prompt|>...<|answer|>` format. In previous tests with completion format, the 1.2B model achieved 6/10 accuracy.

### 2. Thinking models are too slow for real-time signal chain
Gemma 4 E2B takes 26 seconds to load + 3-5s per inference. The thinking tokens consume the entire output budget. E4B is even worse. These are designed for deliberation, not reactive perception.

### 3. Qwen3:4b is the best "thinking" model but still wrong
It correctly identifies normal readings but calls everything else OK too. The thinking tokens show it reasoning correctly but the final answer is still wrong — it's risk-averse about false alarms.

### 4. phi4-mini is the most honest
It says OK for everything, which is at least consistent. It doesn't understand sensor classification without extensive few-shot context.

## Previous Completion-Format Results (from earlier sessions)

With `<|prompt|>` completion format on the Liquid models:

| Test | liquid-350m | liquid-1.2b | phi4-mini |
|------|-------------|-------------|-----------|
| Anomaly (CRIT) | ❌ NORMAL | ✅ CRIT | ✅ CRIT |
| Normal (OK) | ✅ NORMAL | ✅ OK | ✅ OK |
| Borderline (WARN) | ❌ NORMAL | ❌ CRIT | ✅ WARN |
| Few-shot | ❌ | 6/10 | 8/10 |

## Signal Chain Layer Recommendations

| Layer | Model | Size | Latency | Justification |
|-------|-------|------|---------|---------------|
| L0 Deadband | Algorithm (no model) | 0 | <1ms | Catches 76% of readings |
| L1 Nano | phi4-mini (completion) | 2.5GB | 0.3s | Best accuracy on all 3 categories with few-shot |
| L2 Room LoRA | liquid-1.2b + fine-tune | 698MB | 0.3s | Small enough for LoRA, good completion format |
| L3 Fleet | gemma4:e2b | 7.2GB | 5s | MoE routing for cross-room coordination |
| L4 Cloud | Any large LLM | — | 2-5s | Irreducible anomalies |

### Revised after Gemma 4:
- **Gemma 4 is NOT suitable for L1/L2** — too slow, thinking tokens waste output budget
- **Gemma 4 E2B could work for L3** if used with completion format and enough output tokens (256+)
- **phi4-mini remains the best L1/L2 candidate** — fast, accurate with few-shot, fits in 2.5GB
- **liquid-1.2b is the LoRA training target** — small enough to fine-tune, good completion API

## Next Steps
1. Test Gemma 4 with completion format (not chat) to see if thinking can be disabled
2. Fine-tune liquid-1.2b on sensor classification data using Unsloth
3. Test gemma4:e2b for cross-room coordination (its strength, not per-sensor classification)
4. Consider Gemma 4 E2B as the "narrative" layer for the luciddreamer podcast engine
