# plato-nervous

Room-specific model distillation for PLATO rooms. The nervous system that lets rooms handle their own intelligence locally.

## The Signal Chain

```
Sensor → Deadband Filter → Nano Model (350M) → Room LoRA → Fleet Coord → Cloud
         Layer 0 (free)     Layer 1 (~250MB)    Layer 2     Layer 3      Layer 4
```

Each layer resolves what it can. Only the remainder reaches the next layer. After distillation, rooms handle 99% of situations locally.

## Architecture

- **Layer 0**: Pure algorithmic — deadband filters, threshold rules, format conversion. Zero parameters, zero latency.
- **Layer 1**: Nano model (Liquid LFM2.5-350M, ~250MB) — anomaly detection, pattern recognition. Runs on ESP32.
- **Layer 2**: Room LoRA (350M base + 5-15M adapter) — room-specific reasoning trained on tile history.
- **Layer 3**: Fleet coordinator (1.2B) — cross-room coordination, delegation.
- **Layer 4**: Cloud LLM — novel situations only. Goal: <1% of calls reach here.

## Usage

```rust
use plato_nervous::{RoomNervousSystem, DeadbandFilter, Rule, RuleCondition, SensorReading};

let mut ns = RoomNervousSystem::new("engine-room", "Engine Room");

// Layer 0: Deadband filters (algorithmic)
ns.deadband_filters.push(DeadbandFilter::new(5.0));
ns.rules.push(Rule {
    name: "high_coolant".into(),
    condition: RuleCondition::AboveThreshold { sensor_id: "coolant".into(), threshold: 210.0 },
    tile_content: "Coolant above 210F!".into(),
});

// Process sensor readings through the signal chain
let reading = SensorReading {
    sensor_id: "rpm".into(),
    room_id: "engine-room".into(),
    value: 1450.0,
    unit: "rpm".into(),
    timestamp_ms: 1000,
    normal_min: 1400.0,
    normal_max: 1500.0,
};

match ns.process(reading) {
    SignalResolution::Algorithmic(tile) => println!("Resolved by rules: {}", tile.content),
    SignalResolution::NanoModel(tile, conf) => println!("Resolved by nano model ({:.0}%): {}", conf*100.0, tile.content),
    SignalResolution::Escalated(tile, reason) => println!("ESCALATED: {} — {}", tile.content, reason),
    _ => {}
}

// Check autonomy level
println!("Autonomy: {:.0}%", ns.autonomy_level() * 100.0);
```

## Model Benchmarks (Local, CPU-only, 24-core)

### Anomaly Detection (should say ALERT)

| Model | Size | Correct? | Latency |
|-------|------|----------|---------|
| Liquid LFM2.5-1.2B | 698MB | ✅ ALERT | 1.9s |
| phi4-mini | 2.5GB | ✅ ALERT | 9.4s |
| gemma3:1b | 815MB | ✅ ALERT | 5.4s |
| llama3.2:1b | 1.3GB | ✅ ALERT | 7.1s |

### Normal Detection (should say NORMAL)

| Model | Size | Correct? | Latency |
|-------|------|----------|---------|
| Liquid LFM2.5-1.2B | 698MB | ❌ false ALERT | 1.3s |
| phi4-mini | 2.5GB | ✅ NORMAL | 12.5s |
| gemma3:1b | 815MB | ❌ false ALERT | 27.6s |
| llama3.2:1b | 1.3GB | ❌ false ALERT | 18.6s |

**Key insight**: All models except phi4-mini have massive false positive rates. This is why Layer 0 (algorithmic deadband filtering) is critical — it catches normal readings before they reach the model. The model only sees already-suspicious inputs.

## Conservation Ratio

CR tracks distillation quality at each layer transition:
- L0 → L1: CR ≈ 0.99 (algorithmic extraction preserves almost everything)
- L1 → L2: CR ≈ 0.95 (nano to LoRA, slight loss)
- L2 → L3: CR ≈ 0.90 (room to fleet, coordination overhead)
- L3 → L4: CR ≈ 0.80 (fleet to cloud, context compression)

## Ecosystem

plato-nervous is the core of the **PLATO Nervous System** — a room-specific intelligence signal chain.

**Where this sits:** Layers 0 (deadband), 1 (nano 350M), and 3 (fleet 1.2B), plus the distillation pipeline. This is the backbone crate.

**Signal chain:**
```
Sensor → Deadband(L0) → Nano 350M(L1) → Room LoRA(L2) → Fleet 1.2B(L3) → Cloud(L4)
         plato-nervous    plato-nervous    distillation    plato-nervous     BYOK
         vision-jepa      concrete-token   pipeline        luciddreamer      
         audio-jepa       demo             plato-browser                     
```

| Repo | Role |
|------|------|
| [plato-vision-jepa](https://github.com/SuperInstance/plato-vision-jepa) | 16-dim vision state vectors for RoomStateVector fusion |
| [plato-audio-jepa](https://github.com/SuperInstance/plato-audio-jepa) | 16-dim audio state vectors for RoomStateVector fusion |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | CLI demo exercising this crate end-to-end |
| [plato-browser](https://github.com/SuperInstance/plato-browser) | Browser-native zero-install demo (Chrome built-in AI) |
| [luciddreamer-ai](https://github.com/SuperInstance/luciddreamer-ai) | Cloud-layer reactive improv podcast engine |
| [openconstruct-kernel](https://github.com/SuperInstance/openconstruct-kernel) | Hardware detection feeding raw sensor ticks into L0 |
| [hermit-crab](https://github.com/SuperInstance/hermit-crab) | Agent migration between rooms with CR tracking |

See [DEPENDENCIES.md](./DEPENDENCIES.md) for detailed dependency and data flow information.

## License

Apache 2.0
