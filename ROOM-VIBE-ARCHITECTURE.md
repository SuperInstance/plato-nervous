# Room Vibe Architecture: A Living Vector Database for Ambient Intelligence

> **Status:** Research Document
> **Date:** 2026-05-29
> **Context:** Plato Nervous System — sensor-first fleet intelligence

---

## 1. What IS a "Vibe"?

A room's vibe is **not** its current temperature reading. It is not a snapshot. It is not a dashboard gauge.

A vibe is the room's **position on a learned manifold**, derived from the full trajectory of its sensor history, complete with directional momentum and curvature. It is a point that moves continuously through embedding space as ticks arrive — a living state that evolves even when nobody is watching.

Think of it like weather. "It's 72°F" is a reading. "Warm and getting warmer, humidity rising, barometer falling" is a vibe. The vibe carries **derivatives**: where the room has been, where it's heading, and how fast it's changing. It encodes the *character* of a space — its rhythm, its habits, its anomalies.

### 1.1 Formal Definition

Let a room $R$ have a vibe state $V_R(t) \in \mathbb{R}^d$ at time $t$, where $d$ is the embedding dimension. The vibe is a tuple:

$$V_R(t) = \langle\ \mathbf{p}(t),\ \dot{\mathbf{p}}(t),\ \ddot{\mathbf{p}}(t)\ \rangle$$

Where:
- $\mathbf{p}(t)$ is the **position** — the room's current location in embedding space
- $\dot{\mathbf{p}}(t)$ is the **velocity** — the direction and rate of change (momentum)
- $\ddot{\mathbf{p}}(t)$ is the **acceleration** — how the change itself is changing (curvature)

This is not metaphor. This is a point on a Riemannian manifold with a well-defined tangent vector and connection. The vibe *is* a dynamical system.

### 1.2 Why Not Just Aggregate Statistics?

You could track mean temperature, variance, min/max over a window. Many building management systems do exactly this. But aggregated statistics collapse structure. Two rooms with identical means and variances can have radically different *characters*: one oscillating regularly (HVAC cycling), one spiking chaotically (faulty equipment). The vibe captures this because it preserves the **trajectory shape**, not just the moments.

More critically: aggregated stats don't compose. You can't meaningfully compare "mean temperature + vibration + humidity" across rooms. Embedding space gives you a unified metric where cosine similarity actually means something — where rooms with similar *patterns* cluster together regardless of which specific sensors they have.

---

## 2. Tick-by-Tick Embedding Updates

Every sensor tick is a small perturbation to the room's position in embedding space. The update is continuous, lightweight, and local — no batch processing required.

### 2.1 The Core Update Rule

```python
def update_vibe(room: Room, tick: SensorTick, alpha: float, decay_rate: float):
    """
    Update a room's vibe state with a new sensor tick.
    
    Args:
        room: The room whose vibe is being updated
        tick: Raw sensor reading (timestamp, sensor_type, value)
        alpha: Learning rate / step size (typically 0.01-0.1)
        decay_rate: How fast old information fades (typically 0.001-0.01)
    """
    # 1. Embed the tick into the same space as the room's vibe
    tick_embedding = embed_tick(tick)  # shape: (d,)
    
    # 2. Compute the innovation: how surprising is this tick?
    prediction_error = tick_embedding - room.vibe.position
    novelty = np.linalg.norm(prediction_error)
    
    # 3. Adaptive step size: surprising ticks pull harder
    adaptive_alpha = alpha * min(novelty / room.vibe.baseline_novelty, 2.0)
    
    # 4. Update acceleration (second derivative via finite differences)
    new_velocity = adaptive_alpha * prediction_error
    room.vibe.acceleration = new_velocity - room.vibe.velocity
    
    # 5. Update velocity (first derivative)
    room.vibe.velocity = new_velocity
    
    # 6. Update position with momentum
    room.vibe.position += room.vibe.velocity
    
    # 7. Apply decay toward origin (prevents unbounded drift)
    room.vibe.position *= (1.0 - decay_rate)
    
    # 8. Normalize to keep on the unit hypersphere (optional but stable)
    room.vibe.position /= np.linalg.norm(room.vibe.position)
    
    # 9. Update baseline novelty (exponential moving average)
    room.vibe.baseline_novelty = (
        0.99 * room.vibe.baseline_novelty + 0.01 * novelty
    )
    
    # 10. Store the tick embedding in the room's vector DB
    room.vector_db.upsert(
        vector=tick_embedding,
        metadata={
            "timestamp": tick.timestamp,
            "sensor_type": tick.sensor_type,
            "raw_value": tick.value,
            "novelty": novelty,
        }
    )
```

### 2.2 The Decay Function

Decay is the most critical hyperparameter. Too fast, and the room has no memory. Too slow, and it's paralyzed by history.

The decay should be **contextual** — faster in volatile environments (the room is changing rapidly, so old data is misleading), slower in stable ones (old data remains relevant). We achieve this with adaptive decay:

```python
def compute_decay(room: Room) -> float:
    """Adaptive decay rate based on recent volatility."""
    # Volatility = how much the vibe has been moving recently
    volatility = np.linalg.norm(room.vibe.velocity)
    
    # Base decay modified by volatility
    # High volatility → more decay (forget faster, adapt quicker)
    # Low volatility → less decay (remember longer, build deeper patterns)
    base_decay = 0.005
    return base_decay * (1.0 + volatility)
```

### 2.3 Reinforcement vs. Novelty

When a new tick is similar to existing embeddings, it **reinforces** the current direction. When it's novel, it **pulls** in a new direction. This is a continuous version of contrastive learning without explicit negative samples:

- **Reinforcement**: `cosine_similarity(tick_embedding, room.vibe.position) > threshold` → the tick confirms the room's character. Update velocity gently, strengthen the position.
- **Novelty**: similarity is low → the tick challenges the room's character. Update velocity more aggressively, potentially triggering a direction change.

The threshold itself is adaptive — it tightens in stable periods (the room "knows what it is") and loosens during transitions (the room is "figuring itself out").

---

## 3. Cross-Room "Magnetism"

Rooms don't exist in isolation. When two rooms develop similar vibe trajectories — not just similar positions, but similar *dynamics* — they become magnetically attracted. This is the fleet-level emergence.

### 3.1 Magnetism Calculation

```python
def compute_magnetism(room_a: Room, room_b: Room, window: int = 100) -> dict:
    """
    Compute the magnetic attraction between two rooms based on
    recent vibe trajectory correlation.
    
    Args:
        room_a, room_b: Rooms to compare
        window: Number of recent ticks to consider
    
    Returns:
        dict with correlation, derivative (growing/shrinking), and label
    """
    # 1. Get recent vibe trajectories as time series
    traj_a = room_a.get_vibe_trajectory(length=window)  # shape: (window, d)
    traj_b = room_b.get_vibe_trajectory(length=window)
    
    # 2. Compute per-timestep cosine similarities
    similarities = np.array([
        cosine_similarity(traj_a[i], traj_b[i])
        for i in range(window)
    ])
    
    # 3. Current correlation = mean similarity over window
    correlation = float(np.mean(similarities))
    
    # 4. Derivative = linear regression slope of similarities
    # Positive → rooms are converging (magnetism growing)
    # Negative → rooms are diverging (magnetism shrinking)
    t = np.arange(window)
    slope, _ = np.polyfit(t, similarities, 1)
    derivative = float(slope)
    
    # 5. Classify the relationship
    if derivative > 0.001:
        label = "converging"
    elif derivative < -0.001:
        label = "diverging"
    else:
        label = "stable"
    
    return {
        "correlation": correlation,
        "derivative": derivative,
        "label": label,
        "pair": (room_a.id, room_b.id),
    }
```

### 3.2 The Magnetic Field

When you compute magnetism across all room pairs, you get a **dynamic graph**:

- **Nodes** = rooms
- **Edges** = magnetism values (correlation × derivative direction)
- **Edge weights** change over time as rooms evolve

The structure of this graph IS the fleet's understanding of itself. A dense cluster of highly-correlated rooms is a "vibe zone" — spaces that share character. A room drifting away from its cluster is undergoing a vibe shift. A new cluster forming is emergent pattern recognition.

```python
def compute_fleet_graph(rooms: list[Room], window: int = 100) -> nx.Graph:
    """Build the dynamic correlation graph for the entire fleet."""
    G = nx.Graph()
    
    for room in rooms:
        G.add_node(room.id, vibe=room.vibe.position.copy())
    
    for i, room_a in enumerate(rooms):
        for j, room_b in enumerate(rooms):
            if i < j:  # avoid duplicate pairs
                mag = compute_magnetism(room_a, room_b, window)
                if mag["correlation"] > 0.3:  # threshold for edge existence
                    G.add_edge(
                        room_a.id, room_b.id,
                        weight=mag["correlation"],
                        derivative=mag["derivative"],
                        label=mag["label"],
                    )
    
    return G
```

### 3.3 Why Not Merge?

Rooms that are magnetically attracted **keep separate databases**. The magnetism is measured, not merged. This is critical:

1. **Individuality matters.** Two rooms may have correlated trajectories but for different reasons (e.g., both warming up, but one from sunlight, one from HVAC). Merging would lose this nuance.
2. **Divergence is real.** Rooms that converge may later diverge. If they were merged, you'd lose the ability to track this.
3. **The measurement IS the insight.** Knowing that two rooms are magnetically attracted is more valuable than having a single blended understanding. The attraction itself is the fleet-level pattern.

---

## 4. The Agent-less Room

This is the radical proposition: **a room develops understanding autonomously, without any agent present.**

The room IS the agent. Its vector database grows as a passive observer of sensor data. Each tick is a tiny learning event. Over hours and days, the room develops:

- **Habits**: regular patterns that get reinforced (temperature cycles, occupancy rhythms)
- **Anomalies**: unusual events that remain as high-novelty embeddings
- **Character**: the cumulative position in embedding space that defines what this room "is like"
- **Relationships**: magnetic correlations with other rooms in the fleet

When an agent arrives, it doesn't start from scratch. It inherits the room's accumulated understanding. This is like a tenant moving into a house that already has character — the walls remember the conversations, the floors remember the footsteps, the air remembers the temperature of every afternoon.

### 4.1 Room as Autonomous Learner

```python
class Room:
    """An autonomous learning room."""
    
    def __init__(self, room_id: str, embedding_dim: int = 256):
        self.id = room_id
        self.embedding_dim = embedding_dim
        self.vibe = VibeState(dim=embedding_dim)
        self.vector_db = VectorDB(collection=f"room_{room_id}")
        self.tick_count = 0
        self.last_gc_tick = 0
        self.gc_interval = 1000  # run GC every 1000 ticks
        self.pending_novel = []  # novel patterns accumulating for LoRA
    
    def receive_tick(self, tick: SensorTick):
        """Process a sensor tick. No agent required."""
        self.tick_count += 1
        
        # Update the vibe
        update_vibe(self, tick, alpha=0.05, decay_rate=compute_decay(self))
        
        # Check if this tick was novel enough to note
        tick_embedding = embed_tick(tick)
        novelty = cosine_distance(tick_embedding, self.vibe.position)
        
        if novelty > 0.5:  # high novelty threshold
            self.pending_novel.append(tick_embedding)
        
        # Periodic garbage collection
        if self.tick_count - self.last_gc_tick >= self.gc_interval:
            gc_report = self.run_garbage_collection()
            self.last_gc_tick = self.tick_count
            return gc_report
        
        return None
```

### 4.2 What the Room "Knows"

After thousands of ticks, the room's vector database contains a structured representation of its history:

- **Dense clusters** = recurring patterns (daily temperature cycles, weekly occupancy rhythms)
- **Sparse outliers** = anomalies (unusual events, sensor glitches, one-time occurrences)
- **Gradient structures** = slow drift patterns (seasonal changes, equipment degradation)
- **Empty regions** = things that never happen (informative absences)

An arriving agent can query this structure: "What's unusual about right now?" "What usually happens next?" "Has the room ever been in this state before?" The room answers from its own experience, not from a generic model.

---

## 5. Garbage Collection for Embeddings

Without GC, the vector database grows unbounded and degrades. GC is the room's sleep cycle — consolidation, pruning, and archival.

### 5.1 The GC Algorithm

```python
def run_garbage_collection(room: Room) -> GCReport:
    """
    Consolidate the room's vector database.
    
    Three phases:
    1. MERGE: cluster similar embeddings into archetypes
    2. DECAY: reduce weight of unreinforced embeddings
    3. PRUNE: remove embeddings below weight threshold
    """
    all_embeddings = room.vector_db.get_all()
    
    # --- PHASE 1: MERGE similar embeddings ---
    merged_count = 0
    merge_threshold = 0.92  # cosine similarity for merging
    
    # Cluster using a simple greedy approach
    clusters = []
    used = set()
    
    for i, emb_a in enumerate(all_embeddings):
        if i in used:
            continue
        cluster = [i]
        used.add(i)
        
        for j, emb_b in enumerate(all_embeddings):
            if j in used:
                continue
            if cosine_similarity(emb_a.vector, emb_b.vector) > merge_threshold:
                cluster.append(j)
                used.add(j)
        
        if len(cluster) > 1:
            clusters.append(cluster)
            merged_count += len(cluster)
    
    # Replace each cluster with its centroid
    vibe_shifts = []
    for cluster_indices in clusters:
        vectors = [all_embeddings[i].vector for i in cluster_indices]
        centroid = np.mean(vectors, axis=0)
        centroid /= np.linalg.norm(centroid)  # normalize
        
        # Check if this merge changes the vibe significantly
        old_vibe_contribution = np.mean(vectors, axis=0)
        shift_magnitude = cosine_distance(old_vibe_contribution, centroid)
        
        if shift_magnitude > 0.15:  # vibe shift threshold
            vibe_shifts.append({
                "cluster_size": len(cluster_indices),
                "shift": shift_magnitude,
                "direction": centroid - old_vibe_contribution,
            })
        
        # Replace cluster entries with single centroid
        for idx in cluster_indices:
            room.vector_db.delete(all_embeddings[idx].id)
        room.vector_db.upsert(
            vector=centroid,
            metadata={
                "type": "archetype",
                "merged_from": len(cluster_indices),
                "is_vibe_shift": shift_magnitude > 0.15,
            }
        )
    
    # --- PHASE 2: DECAY unreinforced embeddings ---
    decayed_count = 0
    for emb in room.vector_db.get_all():
        if emb.metadata.get("type") == "archetype":
            continue  # archetypes decay slower
        
        age = room.tick_count - emb.metadata.get("tick_added", 0)
        reinforcement_count = emb.metadata.get("reinforcements", 0)
        
        # Decay factor: exponential decay, slowed by reinforcement
        decay_factor = np.exp(-age / 10000) * (1 + reinforcement_count * 0.1)
        
        if decay_factor < 0.01:  # effectively dead
            room.vector_db.delete(emb.id)
            decayed_count += 1
        else:
            # Reduce the embedding's weight
            emb.vector *= decay_factor
            room.vector_db.upsert(
                vector=emb.vector,
                metadata={**emb.metadata, "weight": decay_factor}
            )
    
    # --- PHASE 3: PRUNE by weight ---
    pruned_count = 0
    for emb in room.vector_db.get_all():
        if emb.metadata.get("weight", 1.0) < 0.02:
            room.vector_db.delete(emb.id)
            pruned_count += 1
    
    # --- Generate report ---
    report = GCReport(
        room_id=room.id,
        total_embeddings_before=len(all_embeddings),
        merged=merged_count,
        decayed=decayed_count,
        pruned=pruned_count,
        total_embeddings_after=room.vector_db.count(),
        vibe_shifts=vibe_shifts,
        novel_accumulated=len(room.pending_novel),
    )
    
    # Check if LoRA fine-tuning should be triggered
    if len(room.pending_novel) >= LORA_THRESHOLD:
        report.lora_ready = True
        report.lora_sample_count = len(room.pending_novel)
    
    return report
```

### 5.2 Vibe Shifts

A **vibe shift** occurs when GC merges enough embeddings that the room's character fundamentally changes. This is a significant event — it means the room has transitioned from one state of being to another.

Examples:
- "Room A's temperature pattern stabilized — 847 similar embeddings merged into 12 archetypes"
- "Room B's vibration character changed (was steady, now oscillating)"
- "Room C's occupancy rhythm shifted from weekday-only to 7-day"

Vibe shifts are the room's way of saying "I'm different now." They should propagate to the fleet graph and trigger re-evaluation of magnetic correlations.

### 5.3 GC as LoRA Trigger

When enough novel embeddings accumulate (embeddings that didn't merge with existing clusters), the room signals readiness for LoRA fine-tuning. This means:

> "I've seen enough new patterns that my generic understanding is insufficient. I need a specialized model."

The LoRA adapter would be trained on the room's accumulated novel patterns, giving it room-specific predictive capabilities. The threshold for triggering should be calibrated to balance adaptation speed against training cost.

---

## 6. The JEPA as Moment-Aware Correlator

The Joint Embedding Predictive Architecture (JEPA) serves as the fleet's correlator. It doesn't just predict within a room — it correlates **across** rooms at a given moment in time.

### 6.1 Fleet-Level Correlation at Tick T

At any tick $T$, the JEPA can answer: "How do all rooms relate RIGHT NOW?"

This is a snapshot of the fleet graph:

```python
def correlate_fleet_at_tick(rooms: list[Room], tick: int) -> FleetSnapshot:
    """
    Use the JEPA to correlate all rooms at a specific moment.
    
    This is NOT just pairwise similarity — the JEPA captures higher-order
    relationships through its latent predictions.
    """
    # 1. Gather each room's vibe state at tick T
    vibe_states = {}
    for room in rooms:
        traj = room.get_vibe_trajectory(length=1, at_tick=tick)
        vibe_states[room.id] = {
            "position": traj.position,
            "velocity": traj.velocity,
            "acceleration": traj.acceleration,
        }
    
    # 2. For each room, ask JEPA to predict every other room's vibe
    # This captures the JEPA's learned cross-room relationships
    predictions = {}
    for source_id, source_state in vibe_states.items():
        predictions[source_id] = {}
        for target_id, target_state in vibe_states.items():
            if source_id == target_id:
                continue
            
            predicted = jepa.predict_cross_room(
                source_vibe=source_state,
                target_room=target_id,
            )
            actual = target_state["position"]
            
            # Prediction quality = how well the JEPA understands this pair
            predictions[source_id][target_id] = {
                "predicted": predicted,
                "actual": actual,
                "error": cosine_distance(predicted, actual),
            }
    
    # 3. Build the moment-specific correlation graph
    # High prediction quality → strong edge (JEPA "understands" this pair)
    # Low prediction quality → weak or absent edge (JEPA is surprised)
    graph = nx.Graph()
    for source_id in predictions:
        for target_id, pred in predictions[source_id].items():
            edge_weight = 1.0 - pred["error"]  # invert error → strength
            if edge_weight > 0.3:
                graph.add_edge(source_id, target_id, weight=edge_weight)
    
    return FleetSnapshot(
        tick=tick,
        graph=graph,
        predictions=predictions,
        clusters=list(nx.connected_components(graph)),
    )
```

### 6.2 Time-Varying Graph Structure

The correlation graph is **dynamic** — its structure changes with each tick:

- At $T=100$, rooms A and B might be strongly correlated (both warming up in the morning)
- At $T=1000$, rooms A and B have diverged (A cooled down, B stayed warm)
- At $T=10000$, rooms A, B, and C have formed a new cluster (all showing seasonal drift)

The **evolution of the graph structure** is the fleet's emergent self-understanding. Tracking graph metrics over time reveals:

- **Community formation**: rooms discovering they share patterns
- **Bridge rooms**: rooms that connect otherwise disconnected clusters
- **Isolation events**: rooms that decouple from the fleet (potential anomalies)
- **Phase transitions**: moments when the entire graph restructures

```python
def track_graph_evolution(rooms: list[Room], ticks: list[int]) -> GraphEvolution:
    """Track how the fleet graph evolves over time."""
    snapshots = []
    for tick in ticks:
        snapshot = correlate_fleet_at_tick(rooms, tick)
        snapshots.append(snapshot)
    
    # Extract graph metrics over time
    evolution = GraphEvolution()
    for i, snap in enumerate(snapshots):
        evolution.modularity.append(nx.community.modularity(snap.graph))
        evolution.avg_clustering.append(nx.average_clustering(snap.graph))
        evolution.num_components.append(nx.number_connected_components(snap.graph))
        evolution.density.append(nx.density(snap.graph))
    
    # Detect phase transitions: sudden changes in graph metrics
    for metric in ["modularity", "num_components", "density"]:
        values = getattr(evolution, metric)
        diffs = np.abs(np.diff(values))
        threshold = np.mean(diffs) + 2 * np.std(diffs)
        transitions = np.where(diffs > threshold)[0]
        
        for t in transitions:
            evolution.phase_transitions.append({
                "tick": ticks[t],
                "metric": metric,
                "before": values[t],
                "after": values[t + 1],
            })
    
    return evolution
```

---

## 7. UX Implications of Garbage Collection

GC is not just a maintenance task — it's the system's self-awareness narrative. Users should see what the room has learned, what it's forgotten, and what it's becoming.

### 7.1 GC Reports as System Narratives

Each GC run generates a human-readable narrative:

```
🏛️ Room A — Garbage Collection Report (Tick 12,847)

📊 Database: 3,241 → 1,892 embeddings (-41.6%)

🔄 Merges:
  • Temperature cycle archetype: 342 → 8 embeddings
    "Morning warmup pattern has stabilized"
  • Vibration baseline: 156 → 3 embeddings
    "HVAC vibration is consistent and predictable"
  • Humidity spikes: 89 → 12 embeddings
    "3 distinct humidity event types identified"

📉 Decay:
  • 847 embeddings decayed below threshold
    "Oldest removed: tick 3,201 (9,646 ticks ago)"
  • 23 "rare event" embeddings preserved (reinforced)

⚡ Vibe Shifts Detected:
  • Temperature character changed: was "gradual warming", 
    now "rapid oscillation" (shift magnitude: 0.23)
  → This is a meaningful change. Consider investigation.

🧠 Novel Pattern Accumulation:
  • 847 novel embeddings accumulated
  • LoRA fine-tuning threshold: 1,200 (70.6% ready)
  → Room A is developing unique patterns not covered
    by the base model. Fine-tuning recommended at tick ~14,500.
```

### 7.2 The Vibe Dashboard

For fleet operators, the vibe system provides a living dashboard:

- **Room cards**: Each room shows its current vibe position, velocity magnitude, and acceleration. Color-coded: green (stable), yellow (drifting), red (shifting).
- **Correlation map**: The fleet graph rendered as a force-directed layout. Edges pulse with magnetism strength. Clusters glow.
- **Timeline**: A scrubable timeline showing how the fleet graph has evolved. Phase transitions marked with flags.
- **Anomaly feed**: Rooms that are diverging from their expected vibe trajectory, ranked by severity.

### 7.3 The Room's Voice

The most powerful UX: give each room a voice. Not a literal voice (though that's possible), but a narrative identity that emerges from its vibe:

> "Room A: I'm a busy space. My temperature oscillates every 20 minutes — HVAC is working hard. I get noisy around 9am and 2pm. I've been getting warmer over the past week — something's different. Yesterday I felt a vibration pattern I've never felt before."

This narrative is generated by querying the room's vector DB and vibe state. It turns raw embeddings into human-comprehensible understanding.

---

## 8. Implementation Architecture

### 8.1 Data Flow

```
Sensor → Tick Normalizer → Embed Encoder → Vibe Updater → Vector DB
                                         ↓
                                    GC Scheduler
                                         ↓
                                   LoRA Trigger
                                         
Fleet Graph Builder ← JEPA Cross-Room Predictor ← All Room Vibes
         ↓
   Phase Transition Detector
         ↓
   Alert / Dashboard / Narrative Generator
```

### 8.2 Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Embedding dimension | 256 | Sufficient for multi-sensor fusion without excessive computation |
| Decay rate | Adaptive (0.005 × volatility factor) | Balances memory against adaptability |
| GC interval | Every 1,000 ticks | Frequent enough to prevent bloat, rare enough to amortize cost |
| Merge threshold | 0.92 cosine similarity | Aggressive merging creates clean archetypes |
| Vibe shift threshold | 0.15 cosine distance | Catches meaningful changes, ignores noise |
| LoRA trigger threshold | 1,200 novel embeddings | Enough signal for meaningful fine-tuning |
| Magnetism window | 100 ticks | Captures recent dynamics without being overwhelmed by history |

### 8.3 Computational Costs

Per tick (per room):
- Embed encoding: O(d) — single forward pass through small encoder
- Vibe update: O(d) — vector arithmetic
- Vector DB upsert: O(d × log n) — where n is the room's embedding count

Per GC run (every 1,000 ticks):
- Clustering: O(n²) in the worst case — mitigated by approximate methods
- Decay pass: O(n) — linear scan
- Prune: O(n) — linear scan

Per fleet correlation (on demand):
- Pairwise JEPA predictions: O(R² × d) — where R is number of rooms
- Graph construction: O(R²) — from prediction errors

These costs are manageable for fleets of hundreds to low thousands of rooms. For larger fleets, approximate nearest-neighbor methods and sampled correlations keep costs bounded.

---

## 9. Connections to Plato Nervous System

This vibe architecture integrates directly with the existing Plato components:

1. **Signal Chain Distillation** (see SIGNAL-CHAIN-DISTILLATION.md): The tick embedder is a distilled signal chain — raw sensor data → normalized → embedded. The vibe architecture consumes the output of this pipeline.

2. **Token JEPA** (see CONCRETE-TOKEN-JEPA.md): The cross-room predictor IS the JEPA. The vibe architecture gives the JEPA a structured input (vibe states) rather than raw tokens. The JEPA's predictions become the fleet graph edges.

3. **LoRA Training Pipeline** (see LORA-TRAINING-PIPELINE.md): GC-triggered fine-tuning integrates with the existing LoRA pipeline. The "novel patterns" accumulated by each room become the training data for room-specific adapters.

4. **Dual DB Architecture** (see DUAL-DB-JEPA.md): Each room has its own vector DB (local), while the fleet graph lives in a shared DB (global). The dual-DB pattern maps naturally: local for room-level understanding, global for fleet-level correlation.

---

## 10. Open Questions

1. **Embedding encoder architecture**: Should the tick encoder be a shared model across all rooms, or should each room develop its own encoder via LoRA? Shared is simpler; per-room captures room-specific sensor configurations.

2. **Magnetism vs. causation**: Two rooms may be magnetically attracted (correlated trajectories) without any causal relationship. How do we distinguish "both affected by the same external factor" from "room A's state influences room B"?

3. **Multi-scale vibes**: Should rooms maintain vibes at multiple timescales (minute-level, hour-level, day-level)? A multi-resolution approach could capture both immediate state and long-term character simultaneously.

4. **Room death and birth**: When a room is decommissioned, what happens to its vector DB? When a new room comes online, how does it bootstrap its vibe (cold start problem)?

5. **Privacy boundaries**: The vibe encodes occupancy patterns, which are privacy-sensitive. How do we ensure that the fleet graph doesn't leak information about specific room usage?

6. **Sensor failure handling**: When a sensor fails, the tick stream becomes noisy or absent. The vibe should degrade gracefully — not shift wildly due to sensor malfunction. Anomaly detection on tick embeddings (before they update the vibe) could filter out garbage data.

---

## Appendix A: Embedding the Tick

The tick embedder transforms a raw sensor reading into a fixed-size vector:

```python
class TickEmbedder(nn.Module):
    """Embed raw sensor ticks into a unified vector space."""
    
    def __init__(self, sensor_vocab_size: int, dim: int = 256):
        super().__init__()
        self.sensor_type_embed = nn.Embedding(sensor_vocab_size, 64)
        self.value_encoder = nn.Sequential(
            nn.Linear(1, 32),
            nn.ReLU(),
            nn.Linear(32, 64),
        )
        self.time_encoder = SinusoidalTimeEncoder(dim=64)
        self.fusion = nn.Sequential(
            nn.Linear(64 + 64 + 64, dim),
            nn.ReLU(),
            nn.Linear(dim, dim),
            nn.LayerNorm(dim),
        )
    
    def forward(self, sensor_type_id: int, value: float, timestamp: float):
        s = self.sensor_type_embed(torch.tensor(sensor_type_id))
        v = self.value_encoder(torch.tensor([[value]]))
        t = self.time_encoder(torch.tensor(timestamp))
        return self.fusion(torch.cat([s, v.squeeze(), t]))
```

## Appendix B: VibeState Data Structure

```python
@dataclass
class VibeState:
    """The complete vibe state of a room."""
    position: np.ndarray      # (d,) unit vector — current vibe position
    velocity: np.ndarray      # (d,) — direction and speed of change
    acceleration: np.ndarray  # (d,) — how the change is changing
    baseline_novelty: float   # running average of tick novelty
    tick_count: int           # total ticks processed
    last_gc_tick: int         # tick count at last GC run
    
    def __init__(self, dim: int = 256):
        self.position = np.zeros(dim)
        self.position[0] = 1.0  # start on a basis vector
        self.velocity = np.zeros(dim)
        self.acceleration = np.zeros(dim)
        self.baseline_novelty = 1.0
        self.tick_count = 0
        self.last_gc_tick = 0
```

---

*The room vibe architecture is not just a technical system — it's a philosophy. Rooms are not passive containers for sensors. They are active learners, developing character and understanding from the streams of data that flow through them. The vibe is how a room thinks.*
