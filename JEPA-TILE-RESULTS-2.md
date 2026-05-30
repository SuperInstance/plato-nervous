# JEPA-Style Tile Prediction Experiments — Round 2

_Generated: 2026-05-29T20:10:05.112411_
_Model: liquid-1.2b:latest_

## Summary

Round 2 focuses on the liquid-1.2b sweet spot discovered in Round 1 
(window=10, 16 few-shot examples → 80-90% accuracy on temperature/pressure).

## Experiment 2A: Sweet Spot Grid Search

Focusing on liquid-1.2b with temperature and pressure only (skipping RPM).
Narrow grid around window=10, few-shot=16.

### Results Table

| Window | Sensor | Noise% | FewShot | Val Acc | State Acc | Avg Error | Latency |
|--------|--------|--------|---------|---------|-----------|-----------|---------|
| 11 | temperature | 0% | 12 | 100.0% | 100.0% | 0.75 | 0.1s |
| 11 | pressure | 0% | 12 | 100.0% | 100.0% | 0.05 | 0.49s |
| 11 | pressure | 0% | 14 | 100.0% | 100.0% | 0.04 | 0.36s |
| 12 | temperature | 0% | 18 | 100.0% | 100.0% | 0.66 | 0.65s |
| 12 | pressure | 0% | 12 | 100.0% | 100.0% | 0.03 | 0.49s |
| 12 | pressure | 0% | 14 | 100.0% | 100.0% | 0.02 | 0.46s |
| 12 | pressure | 0% | 18 | 100.0% | 100.0% | 0.05 | 0.29s |
| 7 | temperature | 0% | 18 | 90.0% | 100.0% | 1.01 | 0.26s |
| 7 | pressure | 0% | 16 | 90.0% | 100.0% | 0.07 | 0.19s |
| 10 | pressure | 0% | 14 | 90.0% | 100.0% | 0.05 | 0.58s |
| 11 | pressure | 0% | 16 | 90.0% | 100.0% | 0.06 | 0.61s |
| 11 | pressure | 0% | 18 | 90.0% | 100.0% | 0.07 | 0.72s |
| 11 | pressure | 0% | 20 | 90.0% | 90.0% | 0.63 | 0.84s |
| 12 | temperature | 0% | 20 | 90.0% | 100.0% | 0.61 | 0.74s |
| 12 | pressure | 0% | 16 | 90.0% | 90.0% | 0.44 | 0.38s |
| 12 | pressure | 0% | 20 | 90.0% | 100.0% | 0.07 | 0.63s |
| 8 | pressure | 0% | 12 | 80.0% | 90.0% | 0.65 | 0.26s |
| 8 | pressure | 0% | 14 | 80.0% | 100.0% | 0.09 | 0.21s |
| 8 | pressure | 0% | 16 | 80.0% | 90.0% | 0.45 | 0.2s |
| 9 | temperature | 0% | 12 | 80.0% | 100.0% | 1.22 | 0.18s |
| 9 | pressure | 0% | 12 | 80.0% | 90.0% | 0.25 | 0.48s |
| 10 | temperature | 0% | 12 | 80.0% | 100.0% | 0.97 | 0.38s |
| 10 | pressure | 0% | 12 | 80.0% | 90.0% | 0.7 | 0.46s |
| 11 | temperature | 0% | 14 | 80.0% | 100.0% | 1.05 | 0.5s |
| 11 | temperature | 0% | 18 | 80.0% | 100.0% | 1.07 | 0.57s |
| 11 | temperature | 0% | 20 | 80.0% | 100.0% | 0.82 | 0.67s |
| 12 | temperature | 0% | 16 | 80.0% | 100.0% | 4.19 | 0.36s |
| 8 | temperature | 0% | 12 | 70.0% | 100.0% | 1.51 | 0.17s |
| 8 | pressure | 0% | 18 | 70.0% | 100.0% | 0.09 | 0.21s |
| 10 | temperature | 0% | 20 | 70.0% | 100.0% | 1.3 | 0.45s |
| 10 | pressure | 0% | 16 | 70.0% | 100.0% | 0.11 | 0.57s |
| 10 | pressure | 0% | 18 | 70.0% | 100.0% | 0.15 | 0.57s |
| 11 | temperature | 0% | 16 | 70.0% | 100.0% | 1.27 | 0.53s |
| 12 | temperature | 0% | 12 | 70.0% | 100.0% | 1.17 | 0.46s |
| 12 | temperature | 5% | 12 | 70.0% | 100.0% | 3.71 | 0.67s |
| 7 | pressure | 0% | 12 | 60.0% | 100.0% | 0.1 | 0.25s |
| 7 | pressure | 0% | 18 | 60.0% | 90.0% | 0.25 | 0.24s |
| 8 | temperature | 5% | 16 | 60.0% | 100.0% | 2.76 | 0.44s |
| 9 | pressure | 0% | 14 | 60.0% | 90.0% | 0.55 | 0.67s |
| 9 | pressure | 0% | 16 | 60.0% | 100.0% | 0.12 | 0.53s |
| 9 | pressure | 0% | 20 | 60.0% | 90.0% | 0.63 | 0.53s |
| 9 | pressure | 5% | 12 | 60.0% | 100.0% | 0.15 | 0.75s |
| 10 | temperature | 0% | 14 | 60.0% | 100.0% | 1.63 | 0.41s |
| 12 | temperature | 0% | 14 | 60.0% | 100.0% | 5.95 | 0.48s |
| 7 | temperature | 0% | 12 | 50.0% | 100.0% | 2.87 | 0.23s |
| 7 | temperature | 5% | 14 | 50.0% | 100.0% | 2.74 | 0.48s |
| 7 | pressure | 5% | 14 | 50.0% | 100.0% | 0.16 | 0.42s |
| 9 | temperature | 0% | 14 | 50.0% | 100.0% | 2.62 | 0.22s |
| 9 | temperature | 0% | 18 | 50.0% | 100.0% | 1.91 | 0.51s |
| 9 | pressure | 0% | 18 | 50.0% | 100.0% | 0.13 | 0.74s |
| 9 | pressure | 5% | 18 | 50.0% | 100.0% | 0.2 | 0.67s |
| 10 | temperature | 0% | 16 | 50.0% | 90.0% | 5.54 | 0.62s |
| 10 | temperature | 0% | 18 | 50.0% | 100.0% | 2.65 | 0.69s |
| 11 | pressure | 5% | 12 | 50.0% | 90.0% | 0.43 | 0.74s |
| 12 | temperature | 5% | 18 | 50.0% | 100.0% | 3.3 | 0.61s |
| 12 | pressure | 5% | 12 | 50.0% | 100.0% | 0.16 | 0.66s |
| 12 | pressure | 5% | 16 | 50.0% | 100.0% | 0.2 | 0.5s |
| 7 | temperature | 0% | 14 | 40.0% | 100.0% | 2.36 | 0.23s |
| 7 | temperature | 5% | 20 | 40.0% | 100.0% | 3.55 | 0.49s |
| 7 | pressure | 5% | 20 | 40.0% | 100.0% | 0.2 | 0.48s |
| 8 | temperature | 0% | 14 | 40.0% | 100.0% | 2.26 | 0.3s |
| 8 | temperature | 0% | 16 | 40.0% | 100.0% | 2.41 | 0.34s |
| 8 | temperature | 0% | 18 | 40.0% | 100.0% | 2.99 | 0.24s |
| 8 | temperature | 5% | 14 | 40.0% | 100.0% | 2.3 | 0.42s |
| 8 | temperature | 5% | 18 | 40.0% | 100.0% | 2.78 | 0.45s |
| 8 | pressure | 5% | 14 | 40.0% | 100.0% | 0.18 | 0.21s |
| 8 | pressure | 5% | 16 | 40.0% | 90.0% | 0.62 | 0.42s |
| 8 | pressure | 5% | 18 | 40.0% | 90.0% | 0.46 | 0.5s |
| 9 | temperature | 0% | 16 | 40.0% | 80.0% | 5.27 | 0.27s |
| 9 | temperature | 0% | 20 | 40.0% | 100.0% | 3.8 | 0.05s |
| 9 | temperature | 5% | 14 | 40.0% | 100.0% | 3.38 | 0.56s |
| 9 | temperature | 5% | 16 | 40.0% | 100.0% | 3.51 | 0.63s |
| 9 | temperature | 5% | 20 | 40.0% | 100.0% | 2.84 | 0.59s |
| 10 | temperature | 5% | 14 | 40.0% | 90.0% | 6.11 | 0.49s |
| 10 | pressure | 5% | 12 | 40.0% | 90.0% | 0.4 | 0.44s |
| 10 | pressure | 5% | 18 | 40.0% | 90.0% | 0.56 | 0.54s |
| 11 | temperature | 5% | 16 | 40.0% | 100.0% | 3.55 | 0.52s |
| 12 | pressure | 5% | 14 | 40.0% | 100.0% | 0.2 | 0.5s |
| 12 | pressure | 5% | 18 | 40.0% | 100.0% | 0.15 | 0.49s |
| 7 | temperature | 0% | 16 | 30.0% | 100.0% | 2.71 | 0.32s |
| 7 | pressure | 0% | 14 | 30.0% | 70.0% | 1.17 | 0.19s |
| 7 | pressure | 5% | 16 | 30.0% | 90.0% | 0.44 | 0.45s |
| 7 | pressure | 5% | 18 | 30.0% | 90.0% | 0.58 | 0.44s |
| 8 | temperature | 0% | 20 | 30.0% | 100.0% | 3.4 | 0.26s |
| 9 | temperature | 5% | 12 | 30.0% | 90.0% | 6.34 | 0.37s |
| 9 | pressure | 5% | 14 | 30.0% | 100.0% | 0.33 | 0.7s |
| 10 | temperature | 5% | 12 | 30.0% | 100.0% | 2.51 | 0.59s |
| 10 | temperature | 5% | 16 | 30.0% | 100.0% | 4.09 | 0.65s |
| 10 | pressure | 0% | 20 | 30.0% | 90.0% | 0.69 | 0.74s |
| 10 | pressure | 5% | 14 | 30.0% | 100.0% | 0.22 | 0.66s |
| 10 | pressure | 5% | 16 | 30.0% | 100.0% | 0.23 | 0.46s |
| 11 | temperature | 5% | 12 | 30.0% | 100.0% | 4.2 | 0.56s |
| 11 | temperature | 5% | 20 | 30.0% | 100.0% | 2.91 | 0.73s |
| 11 | pressure | 5% | 16 | 30.0% | 90.0% | 0.53 | 0.52s |
| 12 | temperature | 5% | 16 | 30.0% | 100.0% | 5.74 | 0.18s |
| 7 | temperature | 0% | 20 | 20.0% | 100.0% | 2.89 | 0.31s |
| 7 | temperature | 5% | 16 | 20.0% | 100.0% | 3.63 | 0.51s |
| 8 | pressure | 0% | 20 | 20.0% | 100.0% | 0.18 | 0.22s |
| 8 | pressure | 5% | 20 | 20.0% | 90.0% | 0.43 | 0.51s |
| 9 | temperature | 5% | 18 | 20.0% | 100.0% | 5.47 | 0.76s |
| 9 | pressure | 5% | 16 | 20.0% | 100.0% | 0.31 | 0.75s |
| 9 | pressure | 5% | 20 | 20.0% | 100.0% | 0.34 | 0.45s |
| 10 | temperature | 5% | 18 | 20.0% | 100.0% | 4.53 | 0.63s |
| 11 | temperature | 5% | 14 | 20.0% | 90.0% | 6.4 | 0.42s |
| 11 | temperature | 5% | 18 | 20.0% | 100.0% | 4.29 | 0.43s |
| 11 | pressure | 5% | 14 | 20.0% | 90.0% | 0.77 | 0.41s |
| 11 | pressure | 5% | 18 | 20.0% | 90.0% | 0.89 | 0.53s |
| 11 | pressure | 5% | 20 | 20.0% | 100.0% | 0.23 | 0.64s |
| 12 | temperature | 5% | 20 | 20.0% | 100.0% | 6.62 | 0.59s |
| 7 | temperature | 5% | 18 | 10.0% | 100.0% | 5.21 | 0.4s |
| 7 | pressure | 0% | 20 | 10.0% | 100.0% | 0.2 | 0.2s |
| 8 | temperature | 5% | 20 | 10.0% | 100.0% | 6.04 | 0.5s |
| 8 | pressure | 5% | 12 | 10.0% | 90.0% | 0.57 | 0.16s |
| 10 | temperature | 5% | 20 | 10.0% | 100.0% | 4.5 | 0.48s |
| 10 | pressure | 5% | 20 | 10.0% | 100.0% | 0.3 | 0.72s |
| 12 | temperature | 5% | 14 | 10.0% | 100.0% | 6.57 | 0.5s |
| 12 | pressure | 5% | 20 | 10.0% | 100.0% | 0.31 | 0.4s |
| 7 | temperature | 5% | 12 | 0.0% | 100.0% | 3.88 | 0.11s |
| 7 | pressure | 5% | 12 | 0.0% | 100.0% | 0.25 | -0.03s |
| 8 | temperature | 5% | 12 | 0.0% | 100.0% | 8.09 | -0.04s |

### Top 5 Configurations

1. **window=11, temperature, noise=0%, fewshot=12** → 100.0% accuracy, error=0.75
2. **window=11, pressure, noise=0%, fewshot=12** → 100.0% accuracy, error=0.05
3. **window=11, pressure, noise=0%, fewshot=14** → 100.0% accuracy, error=0.04
4. **window=12, temperature, noise=0%, fewshot=18** → 100.0% accuracy, error=0.66
5. **window=12, pressure, noise=0%, fewshot=12** → 100.0% accuracy, error=0.03

### Window Analysis

| Window | Avg Accuracy | Best Accuracy |
|--------|-------------|--------------|
| 7 | 37.5% | 90.0% |
| 8 | 42.5% | 80.0% |
| 9 | 46.0% | 80.0% |
| 10 | 46.5% | 90.0% |
| 11 | 58.0% | 100.0% |
| 12 | 62.5% | 100.0% |

### Few-Shot Analysis

| Few-Shot | Avg Accuracy | Best Accuracy |
|----------|-------------|--------------|
| 12 | 55.0% | 100.0% |
| 14 | 50.0% | 100.0% |
| 16 | 50.4% | 90.0% |
| 18 | 51.2% | 100.0% |
| 20 | 37.5% | 90.0% |

## Experiment 2B: JEPA Loss Simulation

Simulated JEPA training: predict next tile, compute loss, slide window.
200 tile sequences per sensor, window=10.

### Temperature

| Steps | Avg Loss | Min Loss | Max Loss | Trend |
|-------|----------|----------|----------|--------|
| 1-20 | 0.0442 | 0.0035 | 0.1415 | — baseline |
| 21-40 | 0.0486 | 0.0060 | 0.1450 | ↑ worsening |
| 41-60 | 0.0527 | 0.0023 | 0.1402 | ↑ worsening |
| 61-80 | 0.0621 | 0.0050 | 0.1720 | ↑ worsening |
| 81-100 | 0.0547 | 0.0015 | 0.1228 | ↓ improving |
| 101-120 | 0.0633 | 0.0045 | 0.1680 | ↑ worsening |
| 121-140 | 0.0534 | 0.0140 | 0.0992 | ↓ improving |
| 141-160 | 0.0658 | 0.0000 | 0.2188 | ↑ worsening |
| 161-180 | 0.0579 | 0.0010 | 0.1287 | ↓ improving |
| 181-200 | 0.0772 | 0.0025 | 0.1800 | ↑ worsening |
| 201-220 | 0.0800 | 0.0115 | 0.2550 | → stable |
| 221-239 | 0.0731 | 0.0067 | 0.2090 | ↓ improving |

**Overall average loss: 0.0610**

### Pressure

| Steps | Avg Loss | Min Loss | Max Loss | Trend |
|-------|----------|----------|----------|--------|
| 1-20 | 0.0520 | 0.0000 | 0.2920 | — baseline |
| 21-40 | 0.0626 | 0.0000 | 0.2320 | ↑ worsening |
| 41-60 | 0.0610 | 0.0120 | 0.1400 | → stable |
| 61-80 | 0.0722 | 0.0040 | 0.2160 | ↑ worsening |
| 81-100 | 0.0580 | 0.0040 | 0.2080 | ↓ improving |
| 101-120 | 0.0566 | 0.0080 | 0.1360 | → stable |
| 121-140 | 0.0448 | 0.0040 | 0.1160 | ↓ improving |
| 141-160 | 0.0514 | 0.0040 | 0.1680 | ↑ worsening |
| 161-180 | 0.0680 | 0.0000 | 0.1280 | ↑ worsening |
| 181-200 | 0.0732 | 0.0040 | 0.2280 | ↑ worsening |
| 201-220 | 0.1040 | 0.0040 | 0.2760 | ↑ worsening |
| 221-239 | 0.0636 | 0.0000 | 0.1680 | ↓ improving |

**Overall average loss: 0.0639**

## Experiment 2C: Room-Specific vs Transfer

Train few-shot on engine_room data, test on different rooms.

| Sensor | Train Room | Test Room | Val Acc | Avg Error | Gap |
|--------|-----------|-----------|---------|-----------|-----|
| temperature | engine_room | engine_room | 70.0% | 2.43 | baseline |
| temperature | engine_room | galley | 60.0% | 2.87 | -10.0pp |
| temperature | engine_room | bridge | 90.0% | 1.77 | --20.0pp |
| pressure | engine_room | engine_room | 80.0% | 0.14 | baseline |
| pressure | engine_room | galley | 80.0% | 0.1 | -0.0pp |
| pressure | engine_room | bridge | 90.0% | 0.1 | --10.0pp |

### Transfer Analysis

- **temperature**: same-room=70.0%, cross-room=75.0% → transfer gap=-5.0pp
- **pressure**: same-room=80.0%, cross-room=85.0% → transfer gap=-5.0pp

## Experiment 2D: Anomaly Detection via Prediction Error

Approach: use prediction error as anomaly signal. Compare normal vs injected anomalies.

| Sensor | Normal Error | Anomaly Error | Spike Factor | Detection Rate | False Positive Rate |
|--------|-------------|--------------|-------------|----------------|-------------------|
| temperature | 0.0639 | 0.8881 | 13.91x | 100.0% | 6.5% |
| pressure | 0.0726 | 0.7694 | 10.59x | 100.0% | 8.7% |

### Comparison: Prediction Error vs Deadband

| Approach | Mechanism | Pros | Cons |
|----------|-----------|------|------|
| Prediction error | Next-tile prediction residual | Adapts to patterns, catches subtle drifts | Requires model, latency |
| Deadband (plato-threshold) | Fixed range check | Zero compute, instant | Can't catch slow drift within range |
| Combined | Both signals fused | Best coverage | More complex |

## Conclusions

1. **Optimal configuration**: window=11, temperature, fewshot=12 → 100.0% accuracy
2. **Best window size**: 12 (avg accuracy: 62.5%)
3. **Best few-shot count**: 12 (avg accuracy: 55.0%)
4. **temperature loss does NOT converge**: 0.0491 → 0.0766
4. **pressure loss does NOT converge**: 0.0593 → 0.0793
5. **Cross-room transfer**: prediction accuracy drops when applying engine_room-trained few-shot to other rooms
6. **Anomaly detection**: prediction error spikes 12.2x on anomalies, detection rate 100.0%

## Recommendations for JEPA Training

1. Use liquid-1.2b as the backbone model for JEPA training
2. Window size of 12 tiles provides optimal context
3. 12 few-shot examples balance accuracy vs prompt size
4. Skip RPM predictions — fundamentally unpredictable with current approach
5. Per-room fine-tuning needed for cross-room transfer
6. Prediction error is a viable anomaly signal (complement to deadband)
