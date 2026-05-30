# JEPA-Style Tile Prediction Experiments — Round 2

_Generated: 2026-05-29T20:12:47.745808_
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
| 11 | temperature | 0% | 12 | 100.0% | 100.0% | 0.73 | 0.31s |
| 11 | temperature | 0% | 20 | 100.0% | 100.0% | 0.42 | 0.52s |
| 11 | pressure | 0% | 12 | 100.0% | 100.0% | 0.05 | 0.44s |
| 11 | pressure | 0% | 14 | 100.0% | 100.0% | 0.04 | 0.27s |
| 12 | temperature | 0% | 20 | 100.0% | 100.0% | 0.42 | 0.62s |
| 12 | pressure | 0% | 12 | 100.0% | 100.0% | 0.02 | 0.62s |
| 12 | pressure | 0% | 14 | 100.0% | 100.0% | 0.02 | 0.58s |
| 12 | pressure | 0% | 18 | 100.0% | 100.0% | 0.05 | 0.73s |
| 7 | pressure | 0% | 16 | 90.0% | 100.0% | 0.07 | 0.44s |
| 8 | pressure | 0% | 12 | 90.0% | 90.0% | 0.64 | 0.45s |
| 8 | pressure | 0% | 14 | 90.0% | 100.0% | 0.07 | 0.51s |
| 10 | pressure | 0% | 14 | 90.0% | 100.0% | 0.05 | 0.52s |
| 11 | pressure | 0% | 18 | 90.0% | 100.0% | 0.07 | 0.4s |
| 12 | temperature | 0% | 18 | 90.0% | 100.0% | 0.76 | 0.61s |
| 12 | pressure | 0% | 16 | 90.0% | 90.0% | 0.44 | 0.3s |
| 7 | pressure | 0% | 18 | 80.0% | 90.0% | 0.22 | 0.61s |
| 8 | pressure | 0% | 16 | 80.0% | 90.0% | 0.44 | 0.58s |
| 9 | temperature | 0% | 12 | 80.0% | 100.0% | 1.19 | 0.54s |
| 9 | pressure | 0% | 12 | 80.0% | 90.0% | 0.25 | 0.51s |
| 10 | temperature | 0% | 12 | 80.0% | 100.0% | 0.97 | 0.12s |
| 10 | pressure | 0% | 12 | 80.0% | 90.0% | 0.7 | 0.44s |
| 11 | temperature | 0% | 14 | 80.0% | 100.0% | 1.05 | 0.44s |
| 11 | temperature | 0% | 18 | 80.0% | 100.0% | 1.07 | 0.46s |
| 11 | pressure | 0% | 20 | 80.0% | 90.0% | 0.64 | 0.61s |
| 12 | temperature | 0% | 16 | 80.0% | 100.0% | 4.12 | 0.37s |
| 7 | temperature | 0% | 18 | 70.0% | 100.0% | 1.39 | 0.39s |
| 8 | temperature | 0% | 12 | 70.0% | 100.0% | 1.47 | 0.45s |
| 8 | pressure | 0% | 18 | 70.0% | 100.0% | 0.09 | 0.62s |
| 10 | temperature | 0% | 20 | 70.0% | 100.0% | 1.11 | 0.54s |
| 10 | pressure | 0% | 16 | 70.0% | 100.0% | 0.11 | 0.3s |
| 10 | pressure | 0% | 18 | 70.0% | 100.0% | 0.15 | 0.62s |
| 11 | temperature | 0% | 16 | 70.0% | 100.0% | 1.23 | 0.47s |
| 12 | temperature | 0% | 12 | 70.0% | 100.0% | 1.18 | 0.57s |
| 12 | temperature | 5% | 12 | 70.0% | 100.0% | 3.67 | 0.43s |
| 7 | pressure | 0% | 12 | 60.0% | 100.0% | 0.1 | 0.6s |
| 8 | temperature | 5% | 16 | 60.0% | 100.0% | 2.84 | 0.7s |
| 9 | pressure | 0% | 14 | 60.0% | 90.0% | 0.55 | 0.54s |
| 9 | pressure | 0% | 20 | 60.0% | 90.0% | 0.63 | 0.51s |
| 9 | pressure | 5% | 12 | 60.0% | 100.0% | 0.16 | 0.23s |
| 9 | pressure | 5% | 18 | 60.0% | 100.0% | 0.2 | 0.73s |
| 10 | temperature | 0% | 14 | 60.0% | 100.0% | 1.87 | 0.49s |
| 11 | pressure | 0% | 16 | 60.0% | 100.0% | 0.1 | 0.53s |
| 12 | temperature | 0% | 14 | 60.0% | 100.0% | 6.03 | 0.59s |
| 12 | pressure | 0% | 20 | 60.0% | 100.0% | 0.11 | 0.64s |
| 7 | temperature | 0% | 12 | 50.0% | 100.0% | 2.87 | 0.58s |
| 7 | pressure | 5% | 14 | 50.0% | 100.0% | 0.17 | 0.66s |
| 8 | temperature | 5% | 18 | 50.0% | 100.0% | 2.53 | 0.46s |
| 9 | temperature | 0% | 14 | 50.0% | 100.0% | 2.61 | 0.43s |
| 9 | temperature | 0% | 18 | 50.0% | 100.0% | 2.0 | 0.64s |
| 9 | temperature | 0% | 20 | 50.0% | 100.0% | 3.42 | 0.6s |
| 9 | temperature | 5% | 16 | 50.0% | 100.0% | 3.33 | 0.79s |
| 9 | temperature | 5% | 20 | 50.0% | 100.0% | 2.17 | 0.54s |
| 9 | pressure | 0% | 16 | 50.0% | 100.0% | 0.12 | 0.54s |
| 9 | pressure | 0% | 18 | 50.0% | 100.0% | 0.13 | 0.56s |
| 10 | temperature | 0% | 16 | 50.0% | 90.0% | 5.52 | 0.64s |
| 10 | temperature | 0% | 18 | 50.0% | 100.0% | 2.66 | 0.52s |
| 10 | pressure | 5% | 18 | 50.0% | 90.0% | 0.57 | 0.62s |
| 11 | pressure | 5% | 12 | 50.0% | 90.0% | 0.43 | 0.53s |
| 12 | temperature | 5% | 18 | 50.0% | 100.0% | 3.45 | 0.79s |
| 12 | pressure | 5% | 16 | 50.0% | 100.0% | 0.2 | 0.33s |
| 7 | temperature | 0% | 14 | 40.0% | 100.0% | 2.35 | 0.69s |
| 7 | temperature | 5% | 14 | 40.0% | 100.0% | 2.83 | 0.69s |
| 7 | temperature | 5% | 20 | 40.0% | 100.0% | 3.26 | 0.83s |
| 7 | pressure | 5% | 18 | 40.0% | 90.0% | 0.55 | 0.37s |
| 8 | temperature | 0% | 14 | 40.0% | 100.0% | 2.27 | 0.66s |
| 8 | temperature | 0% | 16 | 40.0% | 100.0% | 2.42 | 0.81s |
| 8 | temperature | 0% | 18 | 40.0% | 100.0% | 3.0 | 0.25s |
| 8 | temperature | 0% | 20 | 40.0% | 100.0% | 3.18 | 0.56s |
| 8 | temperature | 5% | 14 | 40.0% | 100.0% | 2.22 | 0.6s |
| 8 | pressure | 5% | 16 | 40.0% | 90.0% | 0.62 | 0.65s |
| 8 | pressure | 5% | 18 | 40.0% | 90.0% | 0.46 | 0.61s |
| 9 | temperature | 0% | 16 | 40.0% | 80.0% | 5.36 | 0.57s |
| 9 | temperature | 5% | 14 | 40.0% | 100.0% | 3.86 | 0.79s |
| 10 | temperature | 5% | 14 | 40.0% | 90.0% | 5.76 | 0.41s |
| 10 | pressure | 5% | 12 | 40.0% | 90.0% | 0.41 | 0.51s |
| 10 | pressure | 5% | 14 | 40.0% | 100.0% | 0.19 | 0.49s |
| 10 | pressure | 5% | 16 | 40.0% | 100.0% | 0.21 | 0.41s |
| 11 | temperature | 5% | 16 | 40.0% | 100.0% | 3.4 | 0.69s |
| 11 | temperature | 5% | 20 | 40.0% | 100.0% | 2.86 | 0.86s |
| 12 | pressure | 5% | 12 | 40.0% | 100.0% | 0.16 | 0.6s |
| 12 | pressure | 5% | 18 | 40.0% | 100.0% | 0.13 | 0.62s |
| 7 | temperature | 0% | 16 | 30.0% | 100.0% | 2.7 | 0.85s |
| 7 | pressure | 0% | 14 | 30.0% | 70.0% | 1.17 | 0.67s |
| 7 | pressure | 5% | 20 | 30.0% | 100.0% | 0.21 | 0.65s |
| 9 | temperature | 5% | 12 | 30.0% | 90.0% | 6.36 | 0.19s |
| 9 | pressure | 5% | 14 | 30.0% | 100.0% | 0.31 | 0.63s |
| 10 | temperature | 5% | 12 | 30.0% | 100.0% | 2.53 | 0.35s |
| 10 | temperature | 5% | 16 | 30.0% | 100.0% | 3.69 | 0.38s |
| 10 | pressure | 0% | 20 | 30.0% | 90.0% | 0.69 | 0.53s |
| 11 | temperature | 5% | 12 | 30.0% | 100.0% | 4.12 | 0.19s |
| 11 | temperature | 5% | 14 | 30.0% | 90.0% | 6.92 | 0.44s |
| 11 | temperature | 5% | 18 | 30.0% | 100.0% | 4.19 | 0.55s |
| 11 | pressure | 5% | 16 | 30.0% | 90.0% | 0.54 | 0.33s |
| 11 | pressure | 5% | 18 | 30.0% | 90.0% | 0.88 | 0.67s |
| 11 | pressure | 5% | 20 | 30.0% | 100.0% | 0.2 | 0.79s |
| 12 | pressure | 5% | 14 | 30.0% | 100.0% | 0.22 | 0.64s |
| 7 | temperature | 0% | 20 | 20.0% | 100.0% | 2.89 | 0.62s |
| 7 | temperature | 5% | 16 | 20.0% | 100.0% | 3.75 | 0.76s |
| 7 | pressure | 5% | 16 | 20.0% | 90.0% | 0.43 | 0.75s |
| 8 | pressure | 0% | 20 | 20.0% | 100.0% | 0.18 | 0.11s |
| 8 | pressure | 5% | 14 | 20.0% | 100.0% | 0.21 | 0.44s |
| 8 | pressure | 5% | 20 | 20.0% | 90.0% | 0.42 | 0.47s |
| 9 | pressure | 5% | 16 | 20.0% | 100.0% | 0.31 | 0.74s |
| 10 | temperature | 5% | 18 | 20.0% | 100.0% | 4.45 | 0.61s |
| 10 | pressure | 5% | 20 | 20.0% | 100.0% | 0.3 | 0.46s |
| 11 | pressure | 5% | 14 | 20.0% | 90.0% | 0.8 | 0.62s |
| 12 | temperature | 5% | 16 | 20.0% | 100.0% | 5.97 | 0.38s |
| 12 | temperature | 5% | 20 | 20.0% | 100.0% | 6.38 | 0.75s |
| 7 | temperature | 5% | 12 | 10.0% | 100.0% | 3.41 | 0.5s |
| 7 | temperature | 5% | 18 | 10.0% | 100.0% | 5.55 | 0.5s |
| 7 | pressure | 0% | 20 | 10.0% | 100.0% | 0.2 | 0.4s |
| 8 | temperature | 5% | 20 | 10.0% | 100.0% | 6.04 | 0.8s |
| 8 | pressure | 5% | 12 | 10.0% | 90.0% | 0.54 | 0.59s |
| 9 | temperature | 5% | 18 | 10.0% | 100.0% | 6.06 | 0.77s |
| 9 | pressure | 5% | 20 | 10.0% | 100.0% | 0.36 | 0.65s |
| 10 | temperature | 5% | 20 | 10.0% | 100.0% | 4.3 | 0.67s |
| 12 | temperature | 5% | 14 | 10.0% | 100.0% | 6.3 | 0.51s |
| 12 | pressure | 5% | 20 | 10.0% | 100.0% | 0.25 | 0.74s |
| 7 | pressure | 5% | 12 | 0.0% | 100.0% | 0.25 | 0.45s |
| 8 | temperature | 5% | 12 | 0.0% | 100.0% | 7.86 | 0.44s |

### Top 5 Configurations

1. **window=11, temperature, noise=0%, fewshot=12** → 100.0% accuracy, error=0.73
2. **window=11, temperature, noise=0%, fewshot=20** → 100.0% accuracy, error=0.42
3. **window=11, pressure, noise=0%, fewshot=12** → 100.0% accuracy, error=0.05
4. **window=11, pressure, noise=0%, fewshot=14** → 100.0% accuracy, error=0.04
5. **window=12, temperature, noise=0%, fewshot=20** → 100.0% accuracy, error=0.42

### Window Analysis

| Window | Avg Accuracy | Best Accuracy |
|--------|-------------|--------------|
| 7 | 37.0% | 90.0% |
| 8 | 43.5% | 90.0% |
| 9 | 46.5% | 80.0% |
| 10 | 48.5% | 90.0% |
| 11 | 59.5% | 100.0% |
| 12 | 59.5% | 100.0% |

### Few-Shot Analysis

| Few-Shot | Avg Accuracy | Best Accuracy |
|----------|-------------|--------------|
| 12 | 55.4% | 100.0% |
| 14 | 49.6% | 100.0% |
| 16 | 48.8% | 90.0% |
| 18 | 52.9% | 100.0% |
| 20 | 38.8% | 100.0% |

## Experiment 2B: JEPA Loss Simulation

Simulated JEPA training: predict next tile, compute loss, slide window.
200 tile sequences per sensor, window=10.

### Temperature

| Steps | Avg Loss | Min Loss | Max Loss | Trend |
|-------|----------|----------|----------|--------|
| 1-20 | 0.0409 | 0.0030 | 0.1415 | — baseline |
| 21-40 | 0.0484 | 0.0060 | 0.1720 | ↑ worsening |
| 41-60 | 0.0520 | 0.0008 | 0.1265 | ↑ worsening |
| 61-80 | 0.0651 | 0.0050 | 0.1720 | ↑ worsening |
| 81-100 | 0.0525 | 0.0017 | 0.1180 | ↓ improving |
| 101-120 | 0.0645 | 0.0063 | 0.1680 | ↑ worsening |
| 121-140 | 0.0529 | 0.0075 | 0.0927 | ↓ improving |
| 141-160 | 0.0567 | 0.0000 | 0.2188 | ↑ worsening |
| 161-180 | 0.0595 | 0.0010 | 0.1270 | → stable |
| 181-200 | 0.0738 | 0.0050 | 0.1800 | ↑ worsening |
| 201-220 | 0.0757 | 0.0140 | 0.2550 | → stable |
| 221-239 | 0.0690 | 0.0067 | 0.2090 | ↓ improving |

**Overall average loss: 0.0592**

### Pressure

| Steps | Avg Loss | Min Loss | Max Loss | Trend |
|-------|----------|----------|----------|--------|
| 1-20 | 0.0552 | 0.0000 | 0.2920 | — baseline |
| 21-40 | 0.0680 | 0.0000 | 0.1800 | ↑ worsening |
| 41-60 | 0.0588 | 0.0120 | 0.1400 | ↓ improving |
| 61-80 | 0.0734 | 0.0040 | 0.2400 | ↑ worsening |
| 81-100 | 0.0684 | 0.0080 | 0.2080 | ↓ improving |
| 101-120 | 0.0522 | 0.0120 | 0.1360 | ↓ improving |
| 121-140 | 0.0462 | 0.0040 | 0.1160 | ↓ improving |
| 141-160 | 0.0560 | 0.0040 | 0.1480 | ↑ worsening |
| 161-180 | 0.0692 | 0.0000 | 0.1280 | ↑ worsening |
| 181-200 | 0.0756 | 0.0000 | 0.2280 | ↑ worsening |
| 201-220 | 0.1006 | 0.0160 | 0.2760 | ↑ worsening |
| 221-239 | 0.0743 | 0.0000 | 0.2280 | ↓ improving |

**Overall average loss: 0.0665**

## Experiment 2C: Room-Specific vs Transfer

Train few-shot on engine_room data, test on different rooms.

| Sensor | Train Room | Test Room | Val Acc | Avg Error | Gap |
|--------|-----------|-----------|---------|-----------|-----|
| temperature | engine_room | engine_room | 70.0% | 2.34 | baseline |
| temperature | engine_room | galley | 60.0% | 3.04 | -10.0pp |
| temperature | engine_room | bridge | 80.0% | 2.25 | --10.0pp |
| pressure | engine_room | engine_room | 70.0% | 0.15 | baseline |
| pressure | engine_room | galley | 80.0% | 0.11 | --10.0pp |
| pressure | engine_room | bridge | 90.0% | 0.09 | --20.0pp |

### Transfer Analysis

- **temperature**: same-room=70.0%, cross-room=70.0% → transfer gap=0.0pp
- **pressure**: same-room=70.0%, cross-room=85.0% → transfer gap=-15.0pp

## Experiment 2D: Anomaly Detection via Prediction Error

Approach: use prediction error as anomaly signal. Compare normal vs injected anomalies.

| Sensor | Normal Error | Anomaly Error | Spike Factor | Detection Rate | False Positive Rate |
|--------|-------------|--------------|-------------|----------------|-------------------|
| temperature | 0.0647 | 0.8881 | 13.73x | 100.0% | 8.7% |
| pressure | 0.0774 | 0.7744 | 10.0x | 100.0% | 6.5% |

### Comparison: Prediction Error vs Deadband

| Approach | Mechanism | Pros | Cons |
|----------|-----------|------|------|
| Prediction error | Next-tile prediction residual | Adapts to patterns, catches subtle drifts | Requires model, latency |
| Deadband (plato-threshold) | Fixed range check | Zero compute, instant | Can't catch slow drift within range |
| Combined | Both signals fused | Best coverage | More complex |

## Conclusions

1. **Optimal configuration**: window=11, temperature, fewshot=12 → 100.0% accuracy
2. **Best window size**: 11 (avg accuracy: 59.5%)
3. **Best few-shot count**: 12 (avg accuracy: 55.4%)
4. **temperature loss does NOT converge**: 0.0477 → 0.0723
4. **pressure loss does NOT converge**: 0.0615 → 0.0824
5. **Cross-room transfer**: prediction accuracy drops when applying engine_room-trained few-shot to other rooms
6. **Anomaly detection**: prediction error spikes 11.9x on anomalies, detection rate 100.0%

## Recommendations for JEPA Training

1. Use liquid-1.2b as the backbone model for JEPA training
2. Window size of 11 tiles provides optimal context
3. 12 few-shot examples balance accuracy vs prompt size
4. Skip RPM predictions — fundamentally unpredictable with current approach
5. Per-room fine-tuning needed for cross-room transfer
6. Prediction error is a viable anomaly signal (complement to deadband)
