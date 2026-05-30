# Gemma 4 Completion API Benchmark

**Date:** 2026-05-30  
**Endpoint:** `POST /api/generate` (completion, NOT chat)  
**Ollama:** localhost:11434

## Executive Summary

| Model | Size | Type | Sensor Classification Usable? | Verdict |
|-------|------|------|-------------------------------|---------|
| **gemma4:e4b** | 9.6GB | Thinking | ⚠️ Yes but **unreliable** — gave OK for CRIT readings | ❌ Not suitable |
| **gemma4:e2b** | 7.2GB | Thinking | ⚠️ Yes but **wrong** on most tests | ❌ Not suitable |
| **liquid-350m** | 229MB | Standard | ❌ Wrong on all classification tasks | ❌ Too small |
| **liquid-1.2b** | 698MB | Standard | ⚠️ Got T1 right (CRIT), but T3 wrong (said CRIT for OK) | ❌ Inconsistent |
| **phi4-mini** | 2.5GB | Standard | ❌ Wrong on all 5 tests (OK for CRIT, CRIT for OK) | ❌ Bad sensor logic |
| **qwen3:4b** | 2.5GB | Thinking | ⚠️ Thinking consumes all tokens; needs np=1024+ | ⚠️ Works but slow |

## Key Finding: Thinking Models + Completion API

Both **Gemma 4** and **Qwen3:4b** are thinking models. When using the `/api/generate` endpoint:

- **Small `num_predict` (16-256):** Thinking tokens consume the budget → empty `response` field
- **Large `num_predict` (1024):** Thinking tokens + response fit → non-empty response
- **`/no_think` prefix:** Did NOT suppress thinking for Gemma 4 or Qwen3 — still empty responses at low num_predict

Gemma 4 does NOT expose a `thinking` field in the response (thinking is internal/hidden).  
Qwen3:4b DOES expose `thinking` field — you can see thinking content but it eats the token budget.

## Detailed Results (num_predict=1024 for thinking, 64 for standard)

### Test 1: CRIT Detection (RPM=1650, Coolant=228F, Oil=28PSI → CRIT)

| Model | Time | Response | Correct? |
|-------|------|----------|----------|
| gemma4:e4b | 41.8s | **OK** — claims all values normal | ❌ WRONG |
| gemma4:e2b | 36.3s | **WARN** — only flags coolant temp | ❌ WRONG |
| liquid-350m | 2.0s | WARN | ❌ |
| liquid-1.2b | 1.7s | CRIT | ✅ |
| phi4-mini | 9.7s | OK | ❌ WRONG |
| qwen3:4b | 23.4s | Mentions thresholds, implies CRIT | ✅ |

### Test 2: Few-shot WARN (1498rpm 209F 38psi → WARN)

| Model | Time | Response | Correct? |
|-------|------|----------|----------|
| gemma4:e4b | 35.7s | OK | ❌ |
| gemma4:e2b | 27.9s | OK | ❌ |
| liquid-350m | 0.1s | OK | ❌ |
| liquid-1.2b | 0.6s | OK (loops) | ❌ |
| phi4-mini | 1.3s | Narrative, not classification | ❌ |
| qwen3:4b | 19.2s | Empty (thinking consumed all) | ❌ |

### Test 3: OK Detection (RPM=1450, Coolant=195F, Oil=62PSI → OK)

| Model | Time | Response | Correct? |
|-------|------|----------|----------|
| gemma4:e4b | 25.0s | **OK** with rationale | ✅ |
| gemma4:e2b | 25.3s | **WARN** — over-cautious | ❌ |
| liquid-350m | 0.1s | WARN | ❌ |
| liquid-1.2b | 0.1s | CRIT | ❌ |
| phi4-mini | 1.2s | CRIT | ❌ |
| qwen3:4b | 17.4s | Empty (thinking consumed all) | ❌ |

### Test 4: JSON Output (expect status=CRIT)

| Model | Time | Response | Correct? |
|-------|------|----------|----------|
| gemma4:e4b | 21.2s | `{"status": "OK", ...}` | ❌ WRONG |
| gemma4:e2b | 10.7s | `{"status": "data received", ...}` | ❌ |
| liquid-350m | 0.1s | `"status"}` | ❌ |
| liquid-1.2b | 0.6s | `"success", ...}` | ❌ |
| phi4-mini | 1.0s | `{"status": ...}` (partial JSON) | ❌ |
| qwen3:4b | 20.4s | Empty | ❌ |

### Test 5: Cross-room Coordination (RELATED)

| Model | Time | Response | Correct? |
|-------|------|----------|----------|
| gemma4:e4b | 34.8s | **Potentially Related** | ✅ |
| gemma4:e2b | 7.3s | **RELATED** | ✅ |
| liquid-350m | 0.3s | RRR... (garbage) | ❌ |
| liquid-1.2b | 0.5s | Lists events, no classification | ❌ |
| phi4-mini | 1.2s | Related (descriptive) | ✅ |
| qwen3:4b | 17.4s | MOST LIKELY RELATED | ✅ |

## Scorecard

| Model | T1 (CRIT) | T2 (WARN) | T3 (OK) | T4 (JSON) | T5 (Related) | **Score** | Avg Time |
|-------|-----------|-----------|---------|-----------|--------------|-----------|----------|
| gemma4:e4b | ❌ | ❌ | ✅ | ❌ | ✅ | **2/5** | 31.7s |
| gemma4:e2b | ❌ | ❌ | ❌ | ❌ | ✅ | **1/5** | 21.5s |
| liquid-350m | ❌ | ❌ | ❌ | ❌ | ❌ | **0/5** | 0.5s |
| liquid-1.2b | ✅ | ❌ | ❌ | ❌ | ❌ | **1/5** | 0.7s |
| phi4-mini | ❌ | ❌ | ❌ | ❌ | ✅ | **1/5** | 2.9s |
| qwen3:4b | ✅ | ❌ | ❌ | ❌ | ✅ | **2/5** | 19.5s |

## Conclusions

1. **No model passed sensor classification reliably.** The best score was 2/5 (gemma4:e4b, qwen3:4b).
2. **Gemma 4's sensor understanding is poor** — it classified 228F coolant + 28PSI oil pressure as "OK", suggesting it doesn't understand engine sensor thresholds.
3. **Thinking models are impractical for real-time sensor classification:**
   - Gemma 4: 25-42s per inference, hidden thinking tokens
   - Qwen3:4b: 17-23s, thinking often consumes entire output budget
4. **Small models (liquid) are too small** for this task — basically random guessing.
5. **None of these models have built-in engine sensor domain knowledge.** A fine-tuned model or structured rule engine would be far more reliable.

### Recommendation
For sensor classification, use a rule-based system or fine-tune a small model on labeled sensor data. These general-purpose models don't reliably distinguish CRIT/WARN/OK thresholds without explicit threshold values in the prompt.
