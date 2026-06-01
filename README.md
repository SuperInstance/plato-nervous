# plato-nervous

> The full PLATO signal chain: Sensor → Deadband → Nano → LoRA → Fleet → Cloud

## What This Does

plato-nervous implements PLATO's tiered intelligence pipeline. Sensor readings come in at the bottom and each layer resolves what it can, escalating only the hard problems upward. Most readings are handled by simple algorithmic filters (deadband). A few need the nano-model. Rarely, a room-specific LoRA adapter. Almost never, the cloud LLM.

This is the nervous system: fast local reflexes, slower centralized reasoning.

## The Key Idea

Your spinal cord pulls your hand from a hot stove before your brain knows what happened. PLATO works the same way: Layer 0 (algorithmic deadband) handles 90%+ of readings in microseconds, Layer 1 (nano-model) catches anomalies in milliseconds, Layer 2 (room LoRA) handles room-specific patterns, Layer 3 (fleet coordinator) sees cross-room effects, and Layer 4 (cloud) handles genuinely novel situations.

Each resolution produces a "tile" — a resolved observation that records what was found and who resolved it. As tiles accumulate per room, they become training data for distilling room-specific LoRA adapters.

## Install

```bash
cargo add plato-nervous
```

## Quick Start

```rust
use plato_nervous::*;

// Create a sensor reading
let reading = SensorReading {
    sensor_id: "temp-001".into(),
    room_id: "kitchen".into(),
    value: 22.5,
    unit: "celsius".into(),
    timestamp_ms: 1700000000000,
    normal_min: 15.0,
    normal_max: 30.0,
};

// Layer 0: Deadband filter (most readings stop here)
let mut deadband = DeadbandFilter { deadband: 0.5, last_value: None };
if deadband.should_pass(reading.value) {
    // Value changed significantly — create a tile
}
```

## API Reference

### Core Types

| Type | Description |
|---|---|
| `SensorReading { sensor_id, room_id, value, unit, timestamp_ms, normal_min, normal_max }` | Raw sensor data with normal range |
| `Tile { id, room_id, tile_type, content, confidence, resolved_by, ... }` | A resolved observation |
| `TileType` | `Status` / `Alert` / `Prediction` / `Anomaly` / `Coordination` / `Escalation` |
| `ResolutionLayer` | `Algorithmic` → `NanoModel` → `RoomLora` → `FleetCoord` → `CloudEscalation` |
| `TileExample { input, output, quality, layer }` | Training example for distillation |

### Resolution Layers

| Layer | What | When |
|---|---|---|
| 0 Algorithmic | Deadband, thresholds | Value changed > deadband |
| 1 NanoModel | 350M parameter anomaly detection | Pattern looks unusual |
| 2 RoomLora | 350M + LoRA room-specific adapter | Room-specific reasoning needed |
| 3 FleetCoord | 1.2B cross-room coordinator | Multi-room correlation |
| 4 CloudEscalation | Full LLM API call | Novel situation |

### DeadbandFilter

```rust
let mut filter = DeadbandFilter { deadband: 0.5, last_value: None };
filter.should_pass(22.5); // true (first value)
filter.should_pass(22.6); // false (within deadband)
filter.should_pass(23.2); // true (exceeds deadband)
```

### Ollama Integration (`ollama` module)

Integration with local Ollama for nano-model and LoRA inference.

## Testing

45 tests covering: sensor reading handling, deadband filtering, tile creation across all resolution layers, escalation logic, and the full signal chain pipeline.

## License

Apache-2.0
