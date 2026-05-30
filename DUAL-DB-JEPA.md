# Dual-Database JEPA: The Architecture Where Intelligence Lives Between Spaces

**Date:** 2026-05-29
**Status:** Novel Architecture Proposal
**Origin:** Casey's insight — "what if inputs and outputs had SEPARATE vector databases?"

---

## The Problem with Standard JEPA

Standard JEPA (Joint Embedding Predictive Architecture) works like this:

```
Input x → Encoder → z_in ─┐
                            ├→ Predict in SHARED latent space z
Output y → Encoder → z_out ┘
```

Both encoder outputs land in the **same** vector space. The prediction is made there. This creates several problems:

1. **Collapse**: Both encoders can learn to output the same vector regardless of input (the "collapse problem"). You need explicit variance/covariance regularization to prevent it.

2. **Dimensional conflict**: The input space (rich, high-dimensional sensor data) and output space (compact predictions) have different natural dimensionalities. Forcing them into the same space is a lossy compression on one side or wasteful dimensionality on the other.

3. **Asymmetry is lost**: "What I observed" and "what I should predict" are fundamentally different kinds of information. Forcing them into the same geometry destroys this asymmetry.

4. **No reverse queries**: In standard JEPA, you can only predict forward (input → output). Asking "what input would produce this output?" requires inverting the encoder, which is ill-posed.

## Dual-Database JEPA

The insight: **keep the spaces separate, and learn the mapping between them.**

```
Input x → Encoder → Z_in (Perception DB)  ─┐
                                             ├→ Cross-DB Mapping (THE INTELLIGENCE)
Output y → Encoder → Z_out (Prediction DB) ─┘
```

The Perception Database (Z_in) and Prediction Database (Z_out) are **separate vector databases** with potentially different:
- Dimensions
- Distance metrics
- Internal clustering
- Update rates

The intelligence of the system lives in the **cross-database mapping function**, not in the embeddings themselves.

### Why This Works

**1. No collapse problem.** The two spaces can't collapse because they're structurally separate. A perception vector can't "become" a prediction vector — they're in different databases. The mapping function is what relates them, and it's explicitly learned.

**2. Different dimensions are fine.** Perception space can be 64-dimensional (rich sensory encoding). Prediction space can be 8-dimensional (compact action/prediction). The mapping learns to project between them. No information is wasted compressing one to match the other.

**3. Bidirectional queries are free.** 
- Forward: "Given this perception, what should I predict?" → query Z_in, map to Z_out
- Reverse: "What perceptions led to this prediction?" → query Z_out, map back to Z_in
- The mapping function works in both directions because it's a relational model, not a one-way transformation.

**4. Multiple output databases.** Nothing says there's only one Z_out. You can have:
- Z_predict: "what value will this sensor read next?"
- Z_classify: "is this reading normal or anomalous?"
- Z_act: "what action should I take?"
- Z_explain: "why is this happening?"
Each output database has its own dimensions and mapping from perception space. The perception database is shared — the outputs are specialized.

**5. Database-level operations.** Because both sides are proper vector databases, you get:
- Clustering: "what kinds of perceptions exist?" vs "what kinds of predictions exist?"
- Nearest-neighbor: "what similar situations have I seen?"
- Anomaly detection: "this perception doesn't match any in my database"
- Versioning: the databases grow over time as the room accumulates experience

### The Cross-Database Mapping

The mapping function `f: Z_in × Z_out → R` (relevance score) can be implemented as:

| Method | Complexity | Bidirectional? | Learns? |
|--------|-----------|---------------|---------|
| Cosine + projection matrix | O(d_in × d_out) | Yes | Projection weights |
| Weighted Euclidean | O(d_in + d_out) | Yes | Weight vector |
| Cross-attention | O(d_in × d_out × heads) | Yes | Q/K matrices |
| KNN lookup | O(N) | Yes | Implicit (data-driven) |

The simplest that works: project Z_in into Z_out space using a learned matrix, then cosine similarity. The projection matrix IS the room's accumulated intelligence.

### Training

Training is simple:
1. Observe a tile → encode to Z_in
2. Make a prediction → encode to Z_out  
3. Observe the actual outcome
4. Loss = prediction error
5. Update: adjust the mapping function to make this perception-prediction pair score higher
6. The databases themselves grow (new entries), the mapping function learns (updated weights)

This is JEPA-style learning but with the critical difference: **the databases accumulate experience permanently, while the mapping function adapts continuously.**

### Connection to PLATO

In the PLATO signal chain:
- **L0 (deadband)** populates Z_in — "what I noticed"
- **L1 (nano model)** queries Z_in → Z_out — "what does this mean?"
- **L2 (LoRA model)** refines the mapping — "I've seen this pattern before"
- **L3 (fleet model)** cross-references other rooms' databases — "Room B had similar perceptions"
- **L4 (cloud)** trains new mapping functions — "here's a better Z_in→Z_out projection"

Each room has its own pair of databases. Fleet coordination = sharing mapping functions between rooms.

### The Neural Analog

This architecture mirrors how the brain actually works:

- **Sensory cortex** = Z_in (Perception DB) — encodes what was sensed
- **Motor cortex** = Z_out (Prediction/Action DB) — encodes what to do
- **Basal ganglia / cerebellum** = Cross-DB mapping — learns the relationship
- **Hippocampus** = Database management — stores and retrieves experience
- **Prefrontal cortex** = Multiple output DBs — different action plans

The brain doesn't collapse sensory and motor into one space. They remain separate, and the connections between them IS the learned behavior.

## What This Enables

1. **Explainable predictions**: "I predicted X because perception Y matched with prediction X via mapping Z" — full traceability through both databases.

2. **Room-to-room transfer**: Share mapping functions between rooms with similar perception databases but different prediction needs.

3. **Progressive distillation**: Start with cloud-sized databases, distill to room-sized, then to edge-sized — the mapping function shrinks, not the databases.

4. **Active learning**: The room can identify perception vectors with no close matches in Z_in and request cloud attention for novel situations.

5. **Conservation tracking**: Count tiles in Z_in, count predictions in Z_out, verify the mapping preserves information (no tiles lost).

## Crate: plato-jepa-dual

Implemented as a zero-dependency Rust crate with:
- `PerceptionDB` and `PredictionDB` as separate vector stores
- 4 cross-database comparison methods
- Forward and reverse queries
- Training loop with convergence detection
- Different dimensions supported
- Full serialization

---

*This architecture was proposed by Casey on 2026-05-29 during the JEPA + Tile sprint. The core insight — separate databases for inputs and outputs with intelligence in the mapping — is novel and maps naturally to both the PLATO signal chain and known neuroscience.*
