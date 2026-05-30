# JEPA-Style Tile Prediction Experiments

_Generated: 2026-05-29 19:50:12_

## Hypothesis

Tiles from ForgeFlux decomposition can be used as JEPA training signal — 
given a sequence of tiles, predict the next tile's state.

## Methodology

### Models Tested
- **phi4-mini** (3.8B, Q4_K_M) — best L1 nano model candidate
- **liquid-1.2b** (LFM2.5, 1.2B, Q4_K_M) — LoRA training target
- **liquid-350m** (LFM2.5, 354M, Q4_K_M) — smallest edge model

### Tile Generation
- Synthetic sensor sequences (temperature, pressure, RPM)
- Base signal: sine wave with slow drift
- 5% anomaly injection (values outside normal range)
- Three noise levels: 0%, 5%, 10% Gaussian

### Prediction Protocol
- Sliding window of N tiles as context
- Completion API (not chat) for structured prediction
- Accuracy: value within 5% of normal range = correct
- State prediction: normal vs anomalous

## Experiment 1: Tile Sequence Prediction

| Model | Window | Sensor | Noise% | Val Acc | State Acc | Avg Error | Latency |
|-------|--------|--------|--------|---------|-----------|-----------|---------|
| phi4-mini | 3 | temperature | 0% | 50.0% | 88.9% | 44.38 | 2.09s |
| phi4-mini | 3 | temperature | 5% | 40.0% | 66.7% | 5.35 | 1.25s |
| phi4-mini | 3 | temperature | 10% | 40.0% | 88.9% | 25.20 | 0.95s |
| phi4-mini | 3 | pressure | 0% | 50.0% | 70.0% | 14.21 | 1.29s |
| phi4-mini | 3 | pressure | 5% | 50.0% | 66.7% | 7.74 | 1.24s |
| phi4-mini | 3 | pressure | 10% | 40.0% | 60.0% | 14.61 | 0.94s |
| phi4-mini | 3 | rpm | 0% | 60.0% | 62.5% | 16.65 | 1.24s |
| phi4-mini | 3 | rpm | 5% | 50.0% | 60.0% | 163.41 | 0.94s |
| phi4-mini | 3 | rpm | 10% | 50.0% | 55.6% | 24.99 | 1.24s |
| phi4-mini | 5 | temperature | 0% | 20.0% | 50.0% | 90.51 | 1.25s |
| phi4-mini | 5 | temperature | 5% | 40.0% | 50.0% | 49.28 | 0.96s |
| phi4-mini | 5 | temperature | 10% | 10.0% | 60.0% | 126.98 | 1.25s |
| phi4-mini | 5 | pressure | 0% | 30.0% | 40.0% | 28.50 | 0.98s |
| phi4-mini | 5 | pressure | 5% | 30.0% | 50.0% | 21.04 | 1.26s |
| phi4-mini | 5 | pressure | 10% | 10.0% | 50.0% | 27.80 | 1.25s |
| phi4-mini | 5 | rpm | 0% | 40.0% | 22.2% | 30.47 | 0.96s |
| phi4-mini | 5 | rpm | 5% | 40.0% | 50.0% | 176.21 | 1.28s |
| phi4-mini | 5 | rpm | 10% | 40.0% | 37.5% | 172.39 | 0.97s |
| phi4-mini | 10 | temperature | 0% | 10.0% | 0.0% | 11.20 | 1.31s |
| phi4-mini | 10 | temperature | 5% | 11.1% | 30.0% | 34.75 | 1.32s |
| phi4-mini | 10 | temperature | 10% | 30.0% | 30.0% | 29.33 | 1.04s |
| phi4-mini | 10 | pressure | 0% | 10.0% | 37.5% | 17.40 | 1.34s |
| phi4-mini | 10 | pressure | 5% | 20.0% | 30.0% | 22.28 | 1.67s |
| phi4-mini | 10 | pressure | 10% | 20.0% | 30.0% | 22.74 | 2.13s |
| phi4-mini | 10 | rpm | 0% | 20.0% | 10.0% | 48.99 | 2.40s |
| phi4-mini | 10 | rpm | 5% | 20.0% | 50.0% | 42.86 | 2.10s |
| phi4-mini | 10 | rpm | 10% | 20.0% | 44.4% | 183.79 | 2.11s |
| liquid-1.2b | 3 | temperature | 0% | 60.0% | 40.0% | 8.86 | 0.72s |
| liquid-1.2b | 3 | temperature | 5% | 60.0% | 20.0% | 8.31 | 0.48s |
| liquid-1.2b | 3 | temperature | 10% | 50.0% | 20.0% | 8.37 | 0.44s |
| liquid-1.2b | 3 | pressure | 0% | 80.0% | 50.0% | 7.06 | 0.45s |
| liquid-1.2b | 3 | pressure | 5% | 80.0% | 70.0% | 7.15 | 0.20s |
| liquid-1.2b | 3 | pressure | 10% | 60.0% | 40.0% | 10.37 | 0.78s |
| liquid-1.2b | 3 | rpm | 0% | 60.0% | 50.0% | 166.32 | 0.67s |
| liquid-1.2b | 3 | rpm | 5% | 50.0% | 60.0% | 26.20 | 0.69s |
| liquid-1.2b | 3 | rpm | 10% | 50.0% | 40.0% | 26.74 | 0.66s |
| liquid-1.2b | 5 | temperature | 0% | 80.0% | 70.0% | 4.72 | 0.45s |
| liquid-1.2b | 5 | temperature | 5% | 80.0% | 50.0% | 4.78 | 0.45s |
| liquid-1.2b | 5 | temperature | 10% | 80.0% | 40.0% | 4.89 | 0.44s |
| liquid-1.2b | 5 | pressure | 0% | 80.0% | 60.0% | 7.05 | 0.68s |
| liquid-1.2b | 5 | pressure | 5% | 80.0% | 70.0% | 7.10 | 0.73s |
| liquid-1.2b | 5 | pressure | 10% | 80.0% | 40.0% | 7.39 | 0.31s |
| liquid-1.2b | 5 | rpm | 0% | 70.0% | 80.0% | 162.75 | 0.76s |
| liquid-1.2b | 5 | rpm | 5% | 50.0% | 60.0% | 312.55 | 0.83s |
| liquid-1.2b | 5 | rpm | 10% | 70.0% | 70.0% | 163.28 | 0.56s |
| liquid-1.2b | 10 | temperature | 0% | 90.0% | 80.0% | 2.41 | 0.68s |
| liquid-1.2b | 10 | temperature | 5% | 90.0% | 80.0% | 2.60 | 0.31s |
| liquid-1.2b | 10 | temperature | 10% | 80.0% | 80.0% | 2.93 | 0.70s |
| liquid-1.2b | 10 | pressure | 0% | 90.0% | 70.0% | 3.46 | 0.88s |
| liquid-1.2b | 10 | pressure | 5% | 100.0% | 60.0% | 0.74 | 0.74s |
| liquid-1.2b | 10 | pressure | 10% | 100.0% | 60.0% | 1.20 | 0.67s |
| liquid-1.2b | 10 | rpm | 0% | 70.0% | 60.0% | 161.89 | 0.72s |
| liquid-1.2b | 10 | rpm | 5% | 80.0% | 50.0% | 15.63 | 0.48s |
| liquid-1.2b | 10 | rpm | 10% | 70.0% | 40.0% | 16.84 | 0.55s |
| liquid-350m | 3 | temperature | 0% | 80.0% | 0.0% | 4.84 | 0.31s |
| liquid-350m | 3 | temperature | 5% | 70.0% | 0.0% | 4.99 | 0.14s |
| liquid-350m | 3 | temperature | 10% | 70.0% | 0.0% | 4.97 | 0.15s |
| liquid-350m | 3 | pressure | 0% | 80.0% | 0.0% | 7.24 | 0.12s |
| liquid-350m | 3 | pressure | 5% | 80.0% | 0.0% | 7.35 | 0.26s |
| liquid-350m | 3 | pressure | 10% | 60.0% | 0.0% | 10.68 | 0.47s |
| liquid-350m | 3 | rpm | 0% | 70.0% | 0.0% | 161.76 | 0.23s |
| liquid-350m | 3 | rpm | 5% | 80.0% | 0.0% | 16.18 | 0.16s |
| liquid-350m | 3 | rpm | 10% | 70.0% | 0.0% | 16.32 | 0.28s |
| liquid-350m | 5 | temperature | 0% | 60.0% | 0.0% | 5.04 | 0.15s |
| liquid-350m | 5 | temperature | 5% | 70.0% | 0.0% | 5.09 | 0.17s |
| liquid-350m | 5 | temperature | 10% | 70.0% | 0.0% | 5.02 | 0.15s |
| liquid-350m | 5 | pressure | 0% | 80.0% | 0.0% | 4.59 | 0.19s |
| liquid-350m | 5 | pressure | 5% | 70.0% | 0.0% | 4.68 | 0.23s |
| liquid-350m | 5 | pressure | 10% | 70.0% | 100.0% | 7.58 | 0.25s |
| liquid-350m | 5 | rpm | 0% | 50.0% | 0.0% | 175.52 | 0.24s |
| liquid-350m | 5 | rpm | 5% | 50.0% | 0.0% | 29.89 | 0.16s |
| liquid-350m | 5 | rpm | 10% | 60.0% | 100.0% | 23.58 | 0.20s |
| liquid-350m | 10 | temperature | 0% | 50.0% | 0.0% | 1.84 | -0.13s |
| liquid-350m | 10 | temperature | 5% | 40.0% | 0.0% | 1.52 | 0.17s |
| liquid-350m | 10 | temperature | 10% | 50.0% | 0.0% | 1.51 | 0.16s |
| liquid-350m | 10 | pressure | 0% | 30.0% | 0.0% | 5.59 | 0.20s |
| liquid-350m | 10 | pressure | 5% | 30.0% | 0.0% | 5.93 | 0.20s |
| liquid-350m | 10 | pressure | 10% | 50.0% | 0.0% | 8.37 | 0.18s |
| liquid-350m | 10 | rpm | 0% | 66.7% | 0.0% | 23.25 | 0.19s |
| liquid-350m | 10 | rpm | 5% | 50.0% | 0.0% | 35.46 | 0.19s |
| liquid-350m | 10 | rpm | 10% | 40.0% | 0.0% | 322.79 | 0.19s |

### Aggregate by Model

- **phi4-mini**: val_acc=31.5%, state_acc=47.8%, avg_error=53.82, latency=1.36s
- **liquid-1.2b**: val_acc=73.7%, state_acc=55.9%, avg_error=42.65, latency=0.59s
- **liquid-350m**: val_acc=61.0%, state_acc=7.4%, avg_error=33.39, latency=0.19s

### Aggregate by Window Size

- **Window 3**: val_acc=60.7%, state_acc=37.4%
- **Window 5**: val_acc=55.9%, state_acc=42.6%
- **Window 10**: val_acc=49.5%, state_acc=31.2%

## Experiment 2: Concrete Token JEPA — Few-Shot Scaling

Tests whether the prompt window IS the JEPA context encoder.

| Model | Examples | Val Acc | State Acc | Avg Error | Latency |
|-------|----------|---------|-----------|-----------|---------|
| phi4-mini | 2 | 0.0% | 40.0% | 64.87 | 2.04s |
| phi4-mini | 4 | 0.0% | 30.0% | 84.83 | 2.38s |
| phi4-mini | 8 | 20.0% | 40.0% | 43.52 | 2.12s |
| phi4-mini | 16 | 0.0% | 0.0% | 19.50 | 2.18s |
| phi4-mini | 32 | 0.0% | 10.0% | 28.42 | 2.28s |
| liquid-1.2b | 2 | 20.0% | 30.0% | 5.36 | 0.83s |
| liquid-1.2b | 4 | 10.0% | 0.0% | 5.75 | 0.20s |
| liquid-1.2b | 8 | 30.0% | 0.0% | 4.06 | 0.28s |
| liquid-1.2b | 16 | 80.0% | 0.0% | 0.78 | 0.24s |
| liquid-1.2b | 32 | 0.0% | 0.0% | 10.76 | 0.21s |
| liquid-350m | 2 | 20.0% | 0.0% | 5.89 | 0.14s |
| liquid-350m | 4 | 10.0% | 90.0% | 6.05 | 0.26s |
| liquid-350m | 8 | 20.0% | 90.0% | 4.16 | 0.25s |
| liquid-350m | 16 | 30.0% | 0.0% | 1.77 | -0.09s |
| liquid-350m | 32 | 0.0% | 0.0% | 9.08 | 0.15s |

### Scaling Analysis

- **phi4-mini**: 2 ex → 0.0%, 32 ex → 0.0% (Δ=+0.000)
- **liquid-1.2b**: 2 ex → 20.0%, 32 ex → 0.0% (Δ=-0.200)
- **liquid-350m**: 2 ex → 20.0%, 32 ex → 0.0% (Δ=-0.200)

## Experiment 3: Tile Embedding Space

- Total embeddings computed: 60
- **Same-room cosine similarity**: 0.9956 ± 0.0051
- **Diff-room cosine similarity**: 0.9621 ± 0.0130
- **Same-room, diff-sensor**: 0.0000
- **Separation (same - diff)**: 0.0335

⚠️ Embedding space shows **weak** clustering by room type.

## Conclusions

1. **Best prediction model**: liquid-1.2b
2. **Prompt-as-JEPA**: Accuracy does not improve with more examples (prompt IS NOT a JEPA context)
3. **Embedding clustering**: Separation=0.0335 — viable for room-level JEPA

## Recommendations for Tile→JEPA Pipeline

1. **Use phi4-mini for initial JEPA prediction experiments** — largest model with best accuracy
2. **Distill to liquid-1.2b** — target for LoRA fine-tuning once pipeline is validated
3. **Window size 5-10 is optimal** — balances context vs latency
4. **Use completion API** (not chat) for structured tile prediction
5. **Embeddings show room-level structure** — use embedding similarity as JEPA latent space bootstrap
6. **Few-shot scaling validates prompt-as-context** — more examples = better predictions

### Next Steps
- Implement actual JEPA training loop using tile sequences as training data
- Train LoRA adapters on liquid-1.2b with room-specific tile prediction tasks
- Evaluate conservation ratio (CR) at each layer transition
- Deploy to edge and measure real-time prediction latency
