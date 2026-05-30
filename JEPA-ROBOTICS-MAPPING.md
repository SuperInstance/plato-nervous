# JEPA for Physical Systems → Room-Level Understanding

## Research Report
**Date:** 2026-05-29  
**Context:** Mapping the state of Joint-Embedding Predictive Architectures (JEPA) in robotics and physical systems to the Plato Nervous room-level understanding architecture.

---

## 1. V-JEPA and Visual Prediction

### 1.1 Background: What V-JEPA Is

V-JEPA (Video Joint-Embedding Predictive Architecture), developed by Meta FAIR (Bardes, Garrido, Ponce, Chen, Rabbat, LeCun, Assran, Ballas — 2024), is a self-supervised method that learns visual representations from video *without pixel-level reconstruction*. Instead of generating pixels, it predicts in latent space: an encoder maps video frames to embeddings, and a predictor network forecasts the embedding of future frames given the current ones.

The key insight is that V-JEPA discards the decoder entirely. It doesn't reconstruct pixels — it predicts *representations*. This forces the model to learn abstract features (object categories, motion patterns, physical interactions) because low-level pixel details are irrelevant to the prediction task.

### 1.2 Physical Understanding Without Supervision

V-JEPA has demonstrated emergent physical understanding from passive video observation alone. When decoded (using a frozen encoder + separate diffusion decoder), its latent predictions exhibit:

- **Spatio-temporal consistency**: Predicted future states respect physical continuity — objects don't teleport, shadows move appropriately.
- **Object permanence**: Masked regions that contain objects are predicted to maintain coherent object shapes.
- **Motion extrapolation**: Moving objects continue along plausible trajectories (gravity, momentum, collision responses).

The paper "Probing the Latent World: Emergent Discrete Symbols and Physical Structure in Latent Representations" (arXiv, March 2026) directly investigates this — finding that V-JEPA's latent space spontaneously organizes into discrete, interpretable symbols corresponding to physical entities and their relationships.

### 1.3 V-JEPA 2 and V-JEPA 2.1

V-JEPA 2 (Meta FAIR, 2025) scaled the architecture dramatically, showing that larger models trained on more video data produce substantially better physical world models. The key advancement is that V-JEPA 2 can be used as a *world model* for planning — given an action, it can predict the visual consequence.

V-JEPA 2.1 (Mur-Labadia, Muckley, Bar, Assran, Sinha, Rabbat, LeCun, Ballas, Bardes — March 2026) specifically addresses dense feature extraction, unlocking fine-grained spatial understanding that earlier V-JEPA models lacked.

### 1.4 Limitations of Video-Only JEPA

- **No action conditioning**: Standard V-JEPA watches passively. It cannot distinguish "what happens next" from "what I make happen next."
- **No multi-modal grounding**: Pure vision gives no access to physics parameters (mass, friction, force), only visual correlates.
- **Temporal resolution is fixed**: Video operates at fixed frame rates. Real-world sensory data arrives asynchronously.
- **No relational reasoning**: V-JEPA predicts individual frame embeddings, not the *relationships between entities* across time.

---

## 2. Action-Conditioned JEPA for Robotics

### 2.1 The Core Idea: From Observation to Interaction

This is the most directly relevant strand of JEPA research to our room-level architecture. The question is: **given a current state and an action, predict the next state in latent space.**

This is precisely LeCun's vision for JEPA as a world model: the architecture should predict z_{t+1} from z_t and a_t, where z is the latent representation and a is the action. This is the foundation of model-based reinforcement learning without the brittleness of pixel-space reconstruction.

### 2.2 Key Papers and Implementations

**UWM-JEPA: Predictive World Models That Imagine in Belief Space** (arXiv, May 2026)  
This is the most directly relevant recent work. UWM-JEPA extends JEPA to handle uncertainty in world modeling — the "belief space" formulation predicts distributions over future states rather than point estimates. This maps cleanly to our room-vibe architecture: each room's state isn't a single deterministic vector but a distribution capturing uncertainty about the room's current and future conditions.

**JEPA-VLA: Video Predictive Embedding is Needed for VLA Models** (Miao, Feng, Wu, Lin, He, Li, Long — February 2026)  
This paper demonstrates that Vision-Language-Action (VLA) models for robotics perform significantly better when their visual backbone is pretrained with JEPA-style predictive objectives. The key finding: "predictive embeddings pretrained on video... provide anticipatory knowledge of how the environment evolves under successful task execution." This is exactly the "vibe prediction" we need — the model learns not just what the room looks like, but how it tends to evolve.

**Demo-JEPA: Joint-Embedding Predictive Architecture for One-shot Cross-Embodiment Imitation** (He, Li, Zhang, Hou, Che, Zhang — May 2026)  
Demo-JEPA tackles cross-embodiment learning: transferring skills from one robot (or human demonstration) to another. The insight is that JEPA's latent space is *embodiment-agnostic* — it captures the "what" of the task (the trajectory of objects in the world) rather than the "how" (joint angles of a specific robot). This maps to our cross-room correlation: the latent representation should capture the "vibe" independent of which specific sensors produced the observations.

**ThinkJEPA: Empowering Latent World Models with Large Vision-Language Reasoning Model** (Zhang, Li, He, Nagarajan, Chen, Lu, Li, Fu — March 2026)  
ThinkJEPA combines V-JEPA2's latent world model with a vision-language reasoning model. The result is a system that can not only predict future visual states but also *reason* about them linguistically. This is a step toward our goal of rooms that can explain their own vibes.

**LeWorldModel: Stable End-to-End Joint-Embedding Predictive Architecture from Pixels** (Maes, Le Lidec, Scieur, LeCun, Balestriero — March 2026)  
Directly from LeCun's group, this paper addresses the stability issues in training JEPA world models end-to-end from pixels to actions. The "stable" part is critical — training dynamics for JEPA can be unstable when the predictor and encoder are jointly trained.

### 2.3 The Mapping to Our Architecture

| Standard Robotic JEPA | Our Room-Level JEPA |
|---|---|
| Current state = camera image | Current state = room vibe (tick embedding) |
| Action = motor command | Action = environmental event (light change, person entering) |
| Next state = predicted camera image embedding | Next state = predicted room vibe |
| Single agent, single viewpoint | Multiple rooms, multiple sensors |
| Goal: plan motor actions | Goal: predict room correlations |

---

## 3. Multi-Agent and Cross-Entity JEPA

### 3.1 Multi-Agent World Models

While no published paper explicitly applies JEPA to multi-agent systems (as of this research), the conceptual foundations are clear from adjacent work:

**Cross-Embodiment Learning (Demo-JEPA)** already touches on this — different embodiments observing the same environment produce different "views" of the same underlying reality. The JEPA framework naturally handles this by learning shared latent representations across viewpoints.

**Multi-Camera World Models** in autonomous driving (Infrastructure-Centric World Models — Meng, Ai, April 2026) demonstrate the concept of fusing predictions from multiple spatial viewpoints into a coherent world model. This is essentially what our cross-room correlation does — each room is a "camera" on the building's state.

### 3.2 Cross-Agent Correlation Learning

The key insight from multi-agent reinforcement learning that applies here:

- **Decentralized execution, centralized training**: Each agent (room) maintains its own JEPA encoder, but the training signal comes from predicting other agents' states.
- **Communication via latent space**: Rather than sharing raw observations, agents share latent embeddings — exactly what our "vibe broadcast" does.
- **Emergent coordination**: When agents must predict each other's states, they naturally learn to model the causal relationships between their actions.

### 3.3 The Graph Prediction Perspective

Our system goes beyond standard multi-agent JEPA by predicting the *correlation graph* between rooms, not just individual room states. This maps to work on **graph neural network world models** (e.g., the grounded world models literature), where the prediction target is the evolution of a graph structure rather than node features.

The "Grounded World Model for Semantically Generalizable Planning" (Li, Feng, Zhang, Li, Wang, Alahi, Soh — April 2026) addresses exactly this: learning world models that capture semantic relationships between entities, enabling generalization to novel scenarios.

---

## 4. From Pixels to Rooms: The Generalization

### 4.1 The Abstraction Ladder

Standard JEPA operates on a clear abstraction hierarchy:

```
Pixels → Patches → Features → Latent Representations → Predictions
```

Our room-level JEPA introduces a fundamentally different hierarchy:

```
Sensor Ticks → Tick Embeddings → Room Vibes → Cross-Room Correlations → Building State
```

The critical difference is that our "pixels" (ticks) are already semantic — they carry labeled, typed information from specific sensors. We don't need to discover that "this patch of pixels is a thermostat" — the tick already knows it came from a thermostat.

This means our JEPA can skip the low-level representation learning that V-JEPA spends most of its capacity on, and focus entirely on the *temporal and relational* aspects of prediction.

### 4.2 The Dual-DB Architecture as JEPA

Our dual-DB architecture maps cleanly to JEPA's components:

| JEPA Component | Our Component | Function |
|---|---|---|
| Encoder (frozen/context) | Perception DB + vibe encoder | Maps observations to latent space |
| Predictor | Prediction DB + cross-room model | Predicts future latent states |
| Target encoder (EMA of encoder) | Slow-updating room memory | Provides stable prediction targets |
| Loss function | Vibe divergence + correlation error | Drives learning |

The perception DB accumulates observations and continuously updates the room's current vibe — this is the encoder running in inference mode. The prediction DB maintains predictions about other rooms and future states — this is the predictor.

### 4.3 Why This Is Better Than Pixel-Level JEPA for Our Use Case

1. **Efficiency**: We don't waste capacity learning to reconstruct sensor readings.
2. **Grounded semantics**: Every tick already has type information — we know it's a temperature reading, not just a number.
3. **Variable modality**: Different rooms have different sensor mixes. JEPA naturally handles this via the joint embedding — the encoder maps heterogeneous inputs to a shared latent space.
4. **Asynchronous**: Ticks arrive at arbitrary intervals. Our architecture handles this natively; standard JEPA requires fixed frame rates.

---

## 5. Continuous-Time JEPA

### 5.1 The Problem with Discrete Frames

Standard JEPA operates on discrete time steps: t, t+1, t+2, etc. Our system operates in continuous time — ticks arrive at arbitrary intervals, and room states evolve between observations.

### 5.2 Mathematical Frameworks

**Neural ODEs (Chen et al., 2018)** provide the natural framework for continuous-time JEPA. Instead of predicting z_{t+1} from z_t, we define a differential equation:

```
dz/dt = f_θ(z, t)
```

The predictor is no longer a discrete mapping but a vector field that specifies how the latent state evolves at every point in time. Given a current vibe z(t₀) and a future time t₁, we integrate the ODE to get z(t₁).

**Continuous Normalizing Flows** extend this to handle distributions over latent states, which is essential for capturing uncertainty.

**Time-Aware Embeddings** encode the temporal gap between observations directly into the representation. This allows the model to distinguish between "nothing happened in 5 minutes" and "nothing happened in 5 hours."

### 5.3 Variable Frame Rate JEPA

The UWM-JEPA paper (May 2026) partially addresses this through its "belief space" formulation — rather than predicting a single next state, it maintains a distribution that naturally handles temporal uncertainty. But full continuous-time JEPA remains an open research problem.

For our architecture, the solution is pragmatic: we use a time-aware transformer that takes the time interval as an explicit input alongside the tick embedding. The attention mechanism learns to weight observations differently based on recency and the time gaps between them.

### 5.4 The "Vibe as ODE State" Formulation

Each room's vibe is the state of a neural ODE:

```
d(vibe)/dt = f(room_dynamics, recent_ticks, cross_room_signals)
```

When a new tick arrives, it's a discontinuity in the ODE — the vibe jumps. Between ticks, the vibe evolves smoothly according to learned room dynamics. This gives us:

- **Continuous prediction**: We can query the room's state at any time, not just at tick arrivals.
- **Temporal smoothing**: The ODE naturally smooths over noisy sensor readings.
- **Extrapolation**: We can predict beyond the last observation using learned dynamics.

---

## 6. The "Vibe" as JEPA Latent State

### 6.1 Beyond Single-Frame Embeddings

Standard JEPA encoders produce one embedding per input. Our "vibe" is richer — it's a *trajectory embedding* that captures:

1. **Current state**: The room's sensory snapshot right now.
2. **Recent trajectory**: How the room has been evolving (is temperature rising? falling? stable?).
3. **Accumulated patterns**: What this room typically does at this time of day, in this season, under these conditions.
4. **Cross-room context**: What other rooms are doing and how it correlates.

### 6.2 Formalization: Sequence Encoding for Vibes

The vibe can be formalized as the hidden state of a temporal model (LSTM, transformer, or state-space model) that has been processing the room's tick stream:

```
vibe(t) = TemporalModel(tick₁, tick₂, ..., tickₙ, Δt₁, Δt₂, ..., Δtₙ)
```

where Δtᵢ are the time intervals between consecutive ticks.

This is more expressive than V-JEPA's approach because:

- **Variable-length context**: The temporal model can attend to arbitrarily long histories.
- **Explicit time encoding**: Time gaps are first-class inputs, not implicit in frame indices.
- **Hierarchical temporal structure**: The model can learn both fast dynamics (motion detection) and slow dynamics (seasonal patterns).

### 6.3 Trajectory Embeddings

Recent work on trajectory embeddings (from the robot learning community) shows that encoding entire motion trajectories as single vectors enables better prediction and generalization. The "Latent Video Prediction Learns Better World Models" paper (Alrasheed, Parast, Azam, Bailey, Akhtar — May 2026) systematically compares latent prediction approaches and finds that trajectory-level predictions significantly outperform frame-level predictions for world modeling.

For our system: the vibe is not a snapshot — it's a trajectory embedding. It captures where the room has been and where it's going, which is far more informative for prediction.

---

## 7. JEPA Beyond Prediction: Correlation as Intelligence

### 7.1 From Node Prediction to Graph Prediction

Standard JEPA predicts: `z_{t+1} = predictor(z_t)` — a single node's future state.

Our system predicts: `G_{t+1} = predictor(G_t)` — the entire correlation graph's future state.

This is a fundamentally harder and more powerful prediction task. The prediction target isn't "what will room A's temperature be?" but "how will the relationship between rooms A, B, and C change when the front door opens?"

### 7.2 Graph Neural Network + JEPA Hybrids

While no published work combines GNNs with JEPA explicitly for graph-level prediction, the components exist:

- **Graph Neural Networks** for relational reasoning (message passing between room nodes).
- **JEPA-style prediction** for learning latent dynamics.
- **Temporal graph networks** for evolving graph structures.

The natural architecture would be:

1. Each room produces a vibe (node embedding) via its JEPA encoder.
2. A GNN performs message passing between rooms, producing edge embeddings that capture pairwise correlations.
3. A JEPA-style predictor forecasts the future graph state (both node and edge features).
4. The loss is the divergence between predicted and actual graph states.

### 7.3 Correlation as the Prediction Target

Why predict correlations rather than states?

1. **Robustness**: Sensor noise averages out across correlated rooms. If room A's temperature sensor drifts, the correlation with room B's humidity can still detect the anomaly.
2. **Transferability**: Correlations are more stable than absolute states. The correlation between "kitchen heat" and "hallway temperature rise" is the same in summer and winter, even though the absolute values differ.
3. **Anomaly detection**: A broken correlation (kitchen is hot but hallway isn't warming) is immediately detectable, whereas absolute thresholds require manual tuning.
4. **Causal discovery**: Over time, the learned correlation graph reveals the building's causal structure — which rooms influence which others, and through what mechanisms.

### 7.4 Emergent Physics of Buildings

Just as V-JEPA learns physics from video, our room-level JEPA learns the "physics of buildings" — thermal dynamics, occupancy patterns, HVAC behavior, lighting usage patterns — purely from observation. The correlation graph is the building's physics engine.

The "Physically Native World Models: A Hamiltonian Perspective on Generative World Modeling" paper (Cui, Ma — May 2026) proposes that world models should be structured around physical conservation laws. For buildings, the "conservation laws" are:

- Energy conservation (heat in = heat out + stored)
- Occupancy conservation (people don't teleport)
- Temporal periodicity (circadian patterns, weekly patterns)

Our JEPA can discover these laws from data, just as V-JEPA discovers gravity and collisions from video.

---

## 8. Synthesis: The Complete Mapping

### 8.1 Architecture Comparison

```
Standard JEPA Pipeline:
  Input x_t → Encoder → z_t → Predictor → ẑ_{t+1} → Loss vs z_{t+1}

Our Room-Level JEPA Pipeline:
  Ticks(t) → TickEncoder → RoomVibe(t) → CrossRoomGNN → GraphState(t)
       ↓                                              ↓
  Perception DB                              Prediction DB
       ↓                                              ↓
  Current Vibe(t) → CorrelationPredictor → PredictedGraph(t+Δt)
                                                         ↓
                                              Loss vs ActualGraph(t+Δt)
```

### 8.2 Key Innovations in Our Approach

1. **Semantic inputs**: We start from labeled sensor data, not raw pixels.
2. **Continuous time**: Variable-interval ticks, not fixed frame rates.
3. **Graph prediction**: We predict relationships, not individual states.
4. **Dual databases**: Perception and prediction are cleanly separated.
5. **Trajectory embeddings**: Vibes capture temporal dynamics, not just snapshots.
6. **Multi-scale temporal reasoning**: From seconds (motion) to seasons (HVAC patterns).

### 8.3 Open Research Questions

1. **How to handle missing rooms?** If a room goes offline, the graph is incomplete. The JEPA predictor should be robust to partial observations — related to V-JEPA's masking strategy.
2. **What is the right loss function?** Standard JEPA uses VICReg or similar. For room-level prediction, we may need physics-informed losses that penalize physically impossible state transitions.
3. **Scalability**: How many rooms can the GNN handle before message passing becomes a bottleneck? Hierarchical graph structures (rooms → floors → buildings) may be needed.
4. **Cold start**: How to bootstrap the JEPA when a new room comes online with no history? Transfer learning from similar rooms?
5. **Explainability**: Can we decode the correlation graph into human-readable rules? ("When the kitchen stove is on for >30 min, the hallway heats up within 10 minutes.")

---

## 9. Referenced Papers

| Paper | Authors | Date | Relevance |
|---|---|---|---|
| V-JEPA (original) | Bardes, Garrido, Ponce, Chen, Rabbat, LeCun, Assran, Ballas | 2024 | Foundation: latent video prediction |
| V-JEPA 2 | Meta FAIR | 2025 | Scaled world model, action-conditioned |
| V-JEPA 2.1 | Mur-Labadia et al. | Mar 2026 | Dense features for fine-grained understanding |
| UWM-JEPA | — | May 2026 | Belief-space prediction with uncertainty |
| JEPA-VLA | Miao, Feng, Wu, Lin, He, Li, Long | Feb 2026 | JEPA embeddings for vision-language-action |
| Demo-JEPA | He, Li, Zhang, Hou, Che, Zhang | May 2026 | Cross-embodiment imitation via JEPA |
| ThinkJEPA | Zhang, Li, He, Nagarajan, Chen, Lu, Li, Fu | Mar 2026 | V-JEPA2 + VLM reasoning |
| LeWorldModel | Maes, Le Lidec, Scieur, LeCun, Balestriero | Mar 2026 | Stable end-to-end JEPA from pixels |
| Latent Video Prediction | Alrasheed, Parast, Azam, Bailey, Akhtar | May 2026 | Systematic comparison of latent world models |
| Probing the Latent World | — | Mar 2026 | Emergent physical structure in JEPA latents |
| Grounded World Model | Li, Feng, Zhang, Li, Wang, Alahi, Soh | Apr 2026 | Semantically generalizable planning |
| Physically Native World Models | Cui, Ma | May 2026 | Hamiltonian perspective on world modeling |
| VL-JEPA | Chen, Shukor, Moutakanni, Chung, Yu, et al. | Dec 2025 | JEPA for vision-language |
| Gaussian Embeddings | Balestriero, Ballas, Rabbat, LeCun | Oct 2025 | JEPA learns data density |
| Reconstruction or Semantics? | Nilaksh, Jha, Zholus, Chandar | May 2026 | What makes latent spaces useful for robot world models |
| Infrastructure-Centric World Models | Meng, Ai | Apr 2026 | Multi-camera fusion for prediction |

---

## 10. Conclusion

The JEPA ecosystem has matured rapidly from LeCun's original proposal to a rich family of architectures spanning video prediction (V-JEPA), robotics (JEPA-VLA, Demo-JEPA), language-vision (VL-JEPA), and world modeling (UWM-JEPA, LeWorldModel). The core principle — predict in latent space, not pixel space — has proven remarkably effective across domains.

Our room-level understanding architecture represents a natural next step in JEPA's evolution. By operating on semantic sensor data rather than pixels, predicting correlation graphs rather than individual states, and working in continuous time rather than discrete frames, we extend JEPA's philosophy into a domain it hasn't explicitly addressed: the physics of inhabited spaces.

The key takeaway: **JEPA's strength is not in what it predicts, but in what the prediction forces the model to learn.** By training to predict room correlations, our system will necessarily learn the building's physics, occupancy patterns, and causal structure — without any explicit supervision. That's the JEPA promise, and the existing literature gives us strong confidence it will work.
