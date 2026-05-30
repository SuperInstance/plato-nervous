# DEPENDENCIES — plato-nervous

## Signal Chain Layer

**L0 (Deadband) + L1 (Nano 350M) + L3 (Fleet 1.2B) + Distillation Pipeline**

This is the core crate of the PLATO Nervous System. It owns the signal chain from raw sensor input through distillation to fleet-level token output.

## Ecosystem Dependencies

| Repo | Relationship | Description |
|------|-------------|-------------|
| [plato-state](https://github.com/SuperInstance/plato-state) | **Depends on** | Provides 16-dim room state vectors for signal chain processing |
| [plato-vision-jepa](https://github.com/SuperInstance/plato-vision-jepa) | **Depends on** | Provides 16-dim vision state vectors for RoomStateVector fusion |
| [plato-audio-jepa](https://github.com/SuperInstance/plato-audio-jepa) | **Depends on** | Provides 16-dim audio state vectors for RoomStateVector fusion |
| [plato-signal-chain](https://github.com/SuperInstance/plato-signal-chain) | **Depended on by** | Composable pipeline that routes tiles through the signal chain layers |
| [plato-coordination](https://github.com/SuperInstance/plato-coordination) | **Related** | L3 fleet coordination layer |
| [plato-diffusion](https://github.com/SuperInstance/plato-diffusion) | **Related** | Progressive distillation trains models deployed into this crate |
| [plato-tiles](https://github.com/SuperInstance/plato-tiles) | **Related** | Base tile types flowing through the signal chain |
| [plato-rooms](https://github.com/SuperInstance/plato-rooms) | **Related** | Room definitions that nervous systems are instantiated for |
| [hermit-crab](https://github.com/SuperInstance/hermit-crab) | **Depended on by** | Uses plato-nervous room/signal concepts for agent migration tracking |
| [concrete-token-demo](https://github.com/SuperInstance/concrete-token-demo) | **Depended on by** | CLI demo that exercises the plato-nervous distillation pipeline end-to-end |
| [plato-browser](https://github.com/SuperInstance/plato-browser) | **Related** | Browser-native parallel demo; mirrors plato-nervous concepts without Rust |
| [luciddreamer-ai](https://github.com/SuperInstance/luciddreamer-ai) | **Related** | Applies plato-nervous reactive concepts to podcast improv |

## Data Flow

```
IN:
  - Raw sensor ticks (from openconstruct-kernel)
  - 16-dim vision state (from plato-vision-jepa)
  - 16-dim audio state (from plato-audio-jepa)
  - Real ollama model responses

OUT:
  - Distilled ConcreteTokens (L3 fleet-level)
  - RoomStateVector (fused 32-dim from vision + audio)
  - JEPA nano-model predictions
  - Compression ratio (CR) metrics per layer
```
