# Garbage Collection & UX for Room-Level Vector Databases

**Research Report — May 2026**
**Project: Plato Nervous System**

---

## Abstract

Room-level vector databases accumulate embeddings continuously — every tick, every observation, every inference. Without maintenance, they bloat, degrade in quality, and eventually collapse under their own weight. This report defines garbage collection (GC) strategies for embedding databases: when to prune, how to merge, how to detect fundamental shifts in room character, and how to trigger downstream effects like LoRA fine-tuning. Crucially, GC is not just a backend optimization — it is a UX surface. The system's self-awareness of its own learning process is a trust-building mechanism that deserves first-class design.

---

## 1. Vector Database Maintenance Strategies

### 1.1 The Accumulation Problem

A room-level vector database receives new embeddings at every tick cycle (e.g., every 30 seconds). Each embedding represents a momentary snapshot of the room's state — audio features, occupancy patterns, environmental readings. Over a single day, a room generates ~2,880 embeddings. Over a month, ~86,400. Without intervention, query latency degrades, similarity search returns stale results weighted by sheer volume, and the database ceases to represent the room's *current* character.

GC is not optional. It is a fundamental lifecycle operation.

### 1.2 Pruning Strategies

**Age-Based Pruning (Temporal Decay)**

The simplest strategy: embeddings older than a threshold are candidates for removal. But naive age-based pruning throws away rare but important patterns. A better approach couples age with reinforcement count (see §2). Pseudocode:

```
function should_prune(embedding, now, threshold_days=30):
    age_days = (now - embedding.last_reinforced) / (24 * 3600)
    if age_days > threshold_days and embedding.reinforcement_count < 2:
        return true
    if age_days > threshold_days * 3:  # hard ceiling
        return true
    return false
```

**Frequency-Based Pruning**

Embeddings that are never retrieved — never match a query, never contribute to a centroid — are dead weight. Track a `query_hit_count` per embedding. After a window of N ticks with zero hits, the embedding is a candidate.

```
function frequency_prune(embeddings, window_ticks=1000):
    return [e for e in embeddings 
            if e.query_hit_count == 0 
            and e.ticks_since_insertion > window_ticks]
```

**Redundancy-Based Pruning**

When two embeddings are extremely similar (cosine similarity > 0.98), one is redundant. This is the most powerful pruning strategy because it directly targets information content rather than metadata. However, it requires O(n²) comparisons in the naive case, so it must be combined with approximate nearest neighbor (ANN) indexing.

```
function redundancy_prune(embeddings, similarity_threshold=0.98):
    pruned = []
    for each embedding e in embeddings (sorted by reinforcement_count desc):
        neighbors = ann_search(e, k=5)
        for n in neighbors:
            if cosine_similarity(e, n) > similarity_threshold:
                if n.reinforcement_count < e.reinforcement_count:
                    pruned.append(n)
    return unique(pruned)
```

### 1.3 Merging Similar Embeddings

Pruning alone loses information. Merging preserves it in compressed form.

**Centroid Merge**

When multiple embeddings cluster tightly, replace them with their centroid weighted by reinforcement count:

```
function centroid_merge(cluster):
    total_weight = sum(e.reinforcement_count for e in cluster)
    merged = zeros(dim)
    for e in cluster:
        weight = e.reinforcement_count / total_weight
        merged += weight * e.vector
    merged = normalize(merged)
    merged.reinforcement_count = total_weight
    merged.label = cluster[0].label  # or majority vote
    merged.created_at = min(e.created_at for e in cluster)
    merged.last_reinforced = max(e.last_reinforced for e in cluster)
    return merged
```

**Weighted Average (Temporal Decay)**

Give newer embeddings more weight, but not exclusively:

```
function temporal_weighted_merge(cluster, decay_rate=0.95):
    weights = [decay_rate ** hours_since(e.last_reinforced) for e in cluster]
    total = sum(weights)
    merged = zeros(dim)
    for e, w in zip(cluster, weights):
        merged += (w / total) * e.vector
    return normalize(merged)
```

**Learned Merge**

Train a small neural network to produce merged embeddings that best preserve retrieval accuracy on a held-out query set. This is expensive but can be amortized — train once per GC cycle, which might be weekly.

### 1.4 Archive vs. Delete

Deleted embeddings are gone. Archived embeddings move to cold storage and can be restored. The archive is a safety net:

- **When to archive:** Embeddings that fail pruning thresholds but have *any* reinforcement (count ≥ 1). Never hard-delete reinforced embeddings on the first pass.
- **Archive format:** Parquet or similar columnar format, stored locally or in S3. Include full metadata (reinforcement count, creation date, label, source tick).
- **Restoration trigger:** User-initiated, or automatic when a vibe shift is detected and historical context is needed.

### 1.5 Compaction Guarantees

After GC, the database should be smaller but represent the *same* understanding. This is measured by:

1. **Retrieval consistency:** Run a set of probe queries before and after GC. The top-k results should overlap by ≥ 90%.
2. **Centroid stability:** The centroid of the entire embedding space should shift by less than ε (e.g., cosine distance < 0.02).
3. **Coverage ratio:** The fraction of unique "concept clusters" (at similarity threshold 0.90) should not decrease by more than 5%.

### 1.6 Quantifying Information Loss

Information loss from GC is inevitable but measurable. Define:

```
information_retention = (post_gc_archetype_count / pre_gc_unique_clusters) 
                        * retrieval_consistency_ratio
```

Target: information_retention ≥ 0.85 after any GC cycle. If below threshold, reduce GC aggressiveness and flag for review.

---

## 2. Decay and Reinforcement

### 2.1 The Memory Strength Model

Each embedding has a scalar `strength` value that determines its survival probability during GC. Strength is a function of:

- **Age:** How long since the embedding was created or last reinforced.
- **Reinforcement count:** How many times a new tick produced an embedding similar to this one.
- **Recency of reinforcement:** A recent reinforcement counts more than an old one.

```
function compute_strength(embedding, now):
    age_hours = (now - embedding.created_at) / 3600
    hours_since_reinforcement = (now - embedding.last_reinforced) / 3600
    
    # Base strength decays with age
    base = exp(-embedding.decay_rate * age_hours)
    
    # Reinforcement bonus: each reinforcement adds durability
    reinforcement_bonus = log(1 + embedding.reinforcement_count) / log(1 + MAX_REINFORCEMENT)
    
    # Recency factor: recent reinforcement makes the embedding more salient
    recency = exp(-0.1 * hours_since_reinforcement)
    
    return base * (0.3 + 0.7 * reinforcement_bonus) * (0.5 + 0.5 * recency)
```

### 2.2 Exponential Decay

Embeddings that are never reinforced decay exponentially:

```
strength(t) = strength(0) * e^(-λt)
```

Where λ is the decay rate, tuned per room type. A quiet office might use λ=0.01 (slow decay — rare events are significant). A busy lobby might use λ=0.05 (fast decay — only recent patterns matter).

### 2.3 Reinforcement Mechanics

When a new tick embedding arrives, compute its similarity to existing embeddings:

```
function reinforce_or_create(new_embedding, existing_embeddings, threshold=0.92):
    best_match = ann_search(new_embedding, k=1)
    
    if cosine_similarity(new_embedding, best_match) > threshold:
        # Reinforce existing: nudge vector toward new observation
        alpha = 1 / (2 + best_match.reinforcement_count)  # diminishing update
        best_match.vector = normalize(
            (1 - alpha) * best_match.vector + alpha * new_embedding.vector
        )
        best_match.reinforcement_count += 1
        best_match.last_reinforced = now
        return best_match
    else:
        # Novel pattern: insert new embedding
        new_embedding.reinforcement_count = 1
        new_embedding.last_reinforced = now
        insert(new_embedding)
        return new_embedding
```

The key insight: reinforcement is not just a counter. It nudges the embedding vector toward the new observation, making the archetype more representative over time. This is analogous to how biological memories become more robust and refined with each recall.

### 2.4 The Sleep Analogy

In the brain, memory consolidation happens during sleep. The hippocampus replays recent experiences, and the neocortex integrates them into long-term storage. Weak traces are pruned; strong traces are reinforced.

GC is the system's sleep. It should:

1. **Run on a schedule** (e.g., daily at 3 AM local time) — not continuously.
2. **Batch-process** all embeddings accumulated since the last GC cycle.
3. **Produce a consolidation report** — what was merged, what was archived, what was lost.

---

## 3. Vibe Shift Detection

### 3.1 What Is a Vibe Shift?

A vibe shift occurs when the room's character fundamentally changes. Not gradual drift (the room slowly getting quieter as a team winds down a project), but a step change (the room was a quiet office, now it's a bustling workshop).

### 3.2 Statistical Detection

**Centroid Shift Method**

Track the centroid of recent embeddings (last N ticks) vs. historical centroid (all embeddings before that). If the distance exceeds a threshold, flag a vibe shift:

```
function detect_vibe_shift(embeddings, recent_window=100, threshold=0.15):
    recent = embeddings[-recent_window:]
    historical = embeddings[:-recent_window]
    
    centroid_recent = mean(e.vector for e in recent)
    centroid_historical = mean(e.vector for e in historical)
    
    shift = cosine_distance(centroid_recent, centroid_historical)
    
    if shift > threshold:
        return {
            "shift": true,
            "magnitude": shift,
            "direction": centroid_recent - centroid_historical,
            "confidence": compute_confidence(recent, historical, shift)
        }
    return {"shift": false}
```

**Distribution-Based Methods**

More robust than centroid comparison:

- **Kolmogorov-Smirnov test:** Compare the distribution of pairwise distances within recent vs. historical embeddings. A significant difference indicates structural change.
- **Wasserstein distance:** Measures the "cost" of transforming one distribution into another. More sensitive to shifts in spread or multimodality than KS.
- **Practical approach:** Use a sliding window of the last 500 ticks. Compute the mean and variance of the distance distribution. Compare to the previous 500-tick window using a simple z-test on distribution moments. This is cheap and catches most shifts.

```
function detect_vibe_shift_robust(embeddings, window=500):
    recent = embeddings[-window:]
    previous = embeddings[-2*window:-window]
    
    recent_dists = pairwise_cosine_distances(recent, sample=100)
    previous_dists = pairwise_cosine_distances(previous, sample=100)
    
    # Compare distributions
    ks_stat, ks_p = ks_test(recent_dists, previous_dists)
    w_dist = wasserstein_distance(recent_dists, previous_dists)
    
    return {
        "shift": ks_p < 0.01 or w_dist > 0.1,
        "ks_statistic": ks_stat,
        "wasserstein": w_dist,
        "recent_mean_dist": mean(recent_dists),
        "previous_mean_dist": mean(previous_dists)
    }
```

### 3.3 Gradual Drift vs. Sudden Shift

The distinction matters because the response differs:

| Feature | Gradual Drift | Sudden Shift |
|---------|--------------|--------------|
| Rate of change | Slow, continuous | Step function |
| Detection | Rolling average diverges over days | Single window comparison triggers |
| Response | Update archetypes incrementally | Create new archetype branch |
| User alert | Monthly digest | Immediate notification |

Detection strategy: maintain a drift accumulator that tracks cumulative centroid shift over time. If the shift happens within 1-2 GC cycles, it's sudden. If it accumulates over 10+ cycles, it's gradual.

```
function classify_shift(drift_history, window=5):
    recent_drift_rate = mean(drift_history[-window:])
    baseline_drift_rate = mean(drift_history)
    
    if recent_drift_rate > 3 * baseline_drift_rate:
        return "sudden"
    elif recent_drift_rate > 1.5 * baseline_drift_rate:
        return "accelerating"
    else:
        return "gradual"
```

---

## 4. LoRA Training Triggers

### 4.1 When to Retrain?

LoRA fine-tuning is the system's way of internalizing patterns that emerge from embedding GC. The trigger should balance cost (compute, time) against benefit (prediction accuracy).

### 4.2 Novel Pattern Accumulation

Track "novel" embeddings — those that don't match any existing archetype:

```
function count_novel_patterns(new_embeddings, archetypes, threshold=0.85):
    novel = 0
    novel_accumulator = []
    
    for e in new_embeddings:
        best_match = ann_search(e, archetypes, k=1)
        if cosine_similarity(e, best_match) < threshold:
            novel += 1
            novel_accumulator.append(e)
    
    return novel, novel_accumulator
```

### 4.3 Trigger Conditions

Retraining is triggered when ALL of the following are true:

1. **Novel pattern threshold:** `novel_count >= min_novel` (e.g., 50 novel embeddings since last training).
2. **Cluster coherence:** The novel patterns form at least `k` coherent clusters (not just noise). Use DBSCAN or similar:
   ```
   clusters = dbscan(novel_accumulator, eps=0.15, min_samples=5)
   if len(clusters) < 3:
       return  # too fragmented, likely noise
   ```
3. **Staleness:** Time since last training exceeds `min_interval` (e.g., 7 days).
4. **Accuracy degradation:** Prediction accuracy on recent ticks has dropped by ≥ 5% from the post-training baseline.

```
function should_trigger_lora(state):
    if state.novel_since_training < MIN_NOVEL_PATTERNS:
        return false
    if state.days_since_training < MIN_TRAINING_INTERVAL:
        return false
    if state.novel_clusters < MIN_NOVEL_CLUSTERS:
        return false
    if state.accuracy_drop < ACCURACY_DROP_THRESHOLD:
        return false
    return true
```

### 4.4 Cost-Benefit Analysis

| Factor | Cost | Benefit |
|--------|------|---------|
| Compute | ~15 min GPU time per LoRA epoch | Better tick predictions |
| Time | GC + training + validation ~1 hour | Reduced false positive/negative rate |
| Risk | Catastrophic forgetting | Better alignment with current room state |
| Data | Need held-out validation set | Archetypes that match real patterns |

Mitigation for catastrophic forgetting: use LoRA with a low rank (r=4-8), which constrains how much the model can change. Merge LoRA weights with the base model only after validation confirms improvement.

### 4.5 Training Data Composition

When retraining, compose the training set from:

- **50% recent archetypes** (post-GC embeddings from the last 2 weeks)
- **25% reinforced historical** (embeddings with high reinforcement count, regardless of age)
- **25% novel patterns** (the clusters that triggered retraining)

This balance prevents the model from forgetting old, well-established patterns while still adapting to new ones.

---

## 5. UX of Garbage Collection

### 5.1 The Self-Aware System

The system should narrate its own learning. This isn't vanity — it's trust-building. Users who see the system thinking are more likely to trust its outputs, correct its mistakes, and engage with its predictions.

### 5.2 GC Reports

After each GC cycle, produce a human-readable report:

```
╔══════════════════════════════════════════════╗
║  🧠 Memory Consolidation Report             ║
║  Room: Studio A • Cycle #47                 ║
║  2026-05-29 03:00 AKDT                      ║
╠══════════════════════════════════════════════╣
║                                              ║
║  Before: 12,847 embeddings                   ║
║  After:  3,412 embeddings (73% reduction)    ║
║                                              ║
║  Operations:                                 ║
║  • Merged 847 similar → 12 archetypes       ║
║  • Archived 2,104 weak embeddings            ║
║  • Pruned 6,480 redundant ticks              ║
║  • Reinforced 891 strong memories            ║
║                                              ║
║  Information retention: 94.2% ✓              ║
║  Retrieval consistency: 96.8% ✓              ║
║                                              ║
║  Top new archetypes:                         ║
║  1. "Morning ambient + coffee machine"       ║
║  2. "Late-night focused work"                ║
║  3. "Group brainstorm (4-6 people)"          ║
║                                              ║
║  Vibe shift: None detected                   ║
║  LoRA retraining: Not needed (next check 6d) ║
╚══════════════════════════════════════════════╝
```

### 5.3 Vibe Shift Alerts

When a vibe shift is detected, the notification should be immediate and contextual:

> ⚡ **Vibe shift detected in Studio A**
> 
> The room's character changed significantly in the last 2 hours. It shifted from "focused solo work" to "collaborative workshop with music." This is the third shift this month.
> 
> Magnitude: 0.23 (moderate)
> Confidence: 94%
> 
> The system is adapting. A new archetype ("workshop with music") has been created.

### 5.4 The Learning Narrative

Beyond reports, the system should surface insights proactively:

- **"I've noticed this room has three distinct modes: morning focus, afternoon collaboration, and evening quiet. I've created archetypes for each."**
- **"The 'morning ambient + coffee machine' pattern has been reinforced 47 times this week — this is a strong memory now."**
- **"I archived 2,000 embeddings from February because they all described a pattern that hasn't recurred. If the room goes back to that mode, I'll retrieve them."**

This narration serves multiple purposes:
1. **Transparency:** The user understands what the system knows.
2. **Debuggability:** If the system is wrong, the user can see where the mistake came from.
3. **Engagement:** The system feels alive, not like a black box.
4. **Trust calibration:** Users learn what the system is good at and what it isn't.

### 5.5 User Controls

The user should be able to:

- **Request GC:** "Consolidate now" — trigger an immediate GC cycle.
- **Adjust aggressiveness:** Slider from "conservative" (keep more, merge less) to "aggressive" (smaller database, more merging).
- **Protect memories:** Mark certain archetypes as "protected" — they will never be pruned or merged.
- **Restore from archive:** Browse archived embeddings and restore specific ones.
- **Override vibe shift:** Dismiss a detected shift as a one-time anomaly.

---

## 6. GC as Conservation Check

### 6.1 The Plato-Conserve Principle

In the Plato nervous system, conservation means: the total information in the system is never lost without explicit accounting. Every tick that enters the system should either be resolved, escalated, or archived — but never silently dropped.

### 6.2 Conservation Verification

After GC, run a conservation check:

```
function verify_conservation(pre_gc_state, post_gc_state):
    # Tile count conservation
    pre_total = pre_gc_state.resolved + pre_gc_state.escalated + pre_gc_state.archived
    post_total = post_gc_state.resolved + post_gc_state.escalated + post_gc_state.archived
    
    if pre_total != post_total:
        return {
            "conserved": false,
            "violation": "tile_count_mismatch",
            "pre": pre_total,
            "post": post_total,
            "delta": post_total - pre_total
        }
    
    # Archetype coverage: every pre-GC archetype should be representable by post-GC archetypes
    for archetype in pre_gc_state.archetypes:
        nearest = ann_search(archetype, post_gc_state.archetypes, k=1)
        if cosine_distance(archetype, nearest) > COVERAGE_THRESHOLD:
            return {
                "conserved": false,
                "violation": "coverage_gap",
                "lost_archetype": archetype.label,
                "nearest_distance": cosine_distance(archetype, nearest)
            }
    
    return {"conserved": true}
```

### 6.3 Violation Handling

If conservation is violated:

1. **Flag immediately** in the GC report.
2. **Identify the lost information** — which embeddings or archetypes are unaccounted for.
3. **Roll back** to the pre-GC state if the loss exceeds tolerance.
4. **Log the violation** for human review.
5. **Adjust GC parameters** to prevent recurrence.

This is a hard constraint, not a soft target. Conservation violations are bugs, not acceptable losses.

---

## 7. Fleet-Level GC Coordination

### 7.1 Cross-Room Pattern Discovery

When one room's GC reveals a new archetype, check all other rooms for similar patterns:

```
function fleet_gc_coordinate(room_gc_results, fleet_archetypes):
    fleet_wide_patterns = []
    
    for room, results in room_gc_results.items():
        for new_archetype in results.new_archetypes:
            # Check if other rooms have similar patterns
            similar_rooms = []
            for other_room, other_archetypes in fleet_archetypes.items():
                if other_room == room:
                    continue
                best_match = ann_search(new_archetype, other_archetypes, k=1)
                if cosine_similarity(new_archetype, best_match) > 0.88:
                    similar_rooms.append({
                        "room": other_room,
                        "match": best_match,
                        "similarity": cosine_similarity(new_archetype, best_match)
                    })
            
            if len(similar_rooms) >= 2:  # pattern appears in 3+ rooms
                fleet_wide_patterns.append({
                    "pattern": new_archetype,
                    "discovered_in": room,
                    "also_in": similar_rooms,
                    "significance": "fleet_wide"
                })
    
    return fleet_wide_patterns
```

### 7.2 Fleet-Wide Events

When multiple rooms develop similar vibe shifts simultaneously, this may indicate:

- **Environmental event:** Temperature change, power fluctuation, network issue.
- **Behavioral event:** A team moving between rooms, a company-wide meeting.
- **Seasonal pattern:** End-of-quarter crunch, summer schedule change.

Detection:

```
function detect_fleet_event(room_shifts, time_window_hours=4):
    # Group shifts by time proximity
    clusters = temporal_cluster(room_shifts, window=time_window_hours)
    
    fleet_events = []
    for cluster in clusters:
        if len(cluster.rooms) >= MIN_FLEET_EVENT_ROOMS:
            fleet_events.append({
                "type": "fleet_wide_shift",
                "rooms": [r.name for r in cluster.rooms],
                "time_range": [cluster.earliest, cluster.latest],
                "common_direction": mean(s.direction for s in cluster.shifts),
                "hypothesis": generate_hypothesis(cluster)
            })
    
    return fleet_events
```

### 7.3 Fleet-Level Archetypes

Fleet GC produces archetypes that no single room could discover alone. For example:

- **"Monday morning startup"** — appears in Rooms A, C, and F but not B or E.
- **"Post-lunch energy dip"** — universal across all rooms.
- **"Friday afternoon creative burst"** — only in Rooms B and D (the design team).

These fleet-level archetypes become the basis for:

1. **Room-to-room recommendations:** "Room A's Monday pattern is similar to Room C's — they might benefit from shared scheduling."
2. **Anomaly detection at scale:** "Room E is the only room that didn't show the fleet-wide post-lunch dip today."
3. **Meta-learning:** The fleet-level model learns patterns about patterns — how rooms relate to each other, how the building breathes.

### 7.4 Federated GC

For privacy or bandwidth reasons, individual room databases may not be directly accessible. Federated GC works by:

1. Each room produces a GC summary (new archetypes, pruned embeddings, vibe shifts).
2. Summaries are shared with a fleet coordinator.
3. The coordinator identifies cross-room patterns from summaries alone (not raw embeddings).
4. The coordinator sends back fleet-level archetype suggestions.
5. Each room can accept or reject suggestions based on local context.

This mirrors federated learning — the benefits of scale without centralized data access.

---

## 8. Implementation Roadmap

### Phase 1: Core GC (Week 1-2)
- Implement age-based and frequency-based pruning
- Centroid merge for redundant embeddings
- Archive infrastructure (local Parquet storage)
- Conservation verification

### Phase 2: Decay & Reinforcement (Week 3-4)
- Memory strength model (age + reinforcement + recency)
- Reinforcement-on-insert (nudge existing embeddings)
- GC scheduling (daily at low-traffic hours)
- GC reports (text format to start)

### Phase 3: Vibe Shift Detection (Week 5-6)
- Centroid shift detection
- Distribution-based detection (KS test, Wasserstein)
- Drift vs. shift classification
- User-facing vibe shift alerts

### Phase 4: LoRA Triggers (Week 7-8)
- Novel pattern accumulator
- Training trigger conditions
- Automated training pipeline
- Validation and rollback

### Phase 5: Fleet Coordination (Week 9-12)
- Cross-room archetype matching
- Fleet-wide event detection
- Fleet-level archetype synthesis
- Federated GC protocol

---

## 9. Key Design Principles

1. **Conservation is non-negotiable.** Information loss must be measured, not assumed.
2. **GC is sleep, not death.** It consolidates and strengthens, not just deletes.
3. **The user sees the system thinking.** Transparency builds trust; opacity breeds suspicion.
4. **Decay rates are per-room.** A quiet archive and a busy lobby have different memory needs.
5. **Reinforcement is learning.** Every tick that matches an existing pattern makes that pattern stronger.
6. **Fleet patterns emerge from local GC.** No room needs to know about other rooms to contribute to fleet intelligence.
7. **Archives are safety nets.** Nothing is truly deleted on the first pass.

---

## Appendix A: Pseudocode Summary

### A.1 Complete GC Cycle

```
function gc_cycle(room_db, config):
    # 1. Compute strength for all embeddings
    for e in room_db.all_embeddings:
        e.strength = compute_strength(e, now)
    
    # 2. Identify candidates
    prune_candidates = [e for e in room_db if should_prune(e, now)]
    merge_candidates = redundancy_cluster(room_db, threshold=0.95)
    
    # 3. Execute merges
    merged = []
    for cluster in merge_candidates:
        merged.append(centroid_merge(cluster))
    
    # 4. Archive weak embeddings
    archived = []
    for e in prune_candidates:
        if e.reinforcement_count > 0:
            archive(e)
            archived.append(e)
    
    # 5. Hard-delete unreinforced, old embeddings
    deleted = [e for e in prune_candidates if e.reinforcement_count == 0]
    
    # 6. Update database
    room_db.remove(prune_candidates)
    room_db.remove([e for cluster in merge_candidates for e in cluster])
    room_db.insert(merged)
    
    # 7. Verify conservation
    conservation = verify_conservation(pre_state, post_state)
    if not conservation.conserved:
        rollback(room_db, pre_state)
        alert("GC conservation violation — rolled back")
    
    # 8. Check for vibe shift
    shift = detect_vibe_shift_robust(room_db.all_embeddings)
    
    # 9. Check LoRA trigger
    lora_needed = should_trigger_lora(room_db.state)
    
    # 10. Generate report
    return gc_report(pruned=len(prune_candidates), merged=len(merged),
                     archived=len(archived), deleted=len(deleted),
                     conservation=conservation, shift=shift, lora=lora_needed)
```

### A.2 Decay Function

```
function decay_strength(age_hours, reinforcement_count, hours_since_reinforced, 
                        base_decay=0.02):
    base = exp(-base_decay * age_hours)
    reinforcement = log(1 + reinforcement_count) / log(1 + 100)  # normalized to [0,1]
    recency = exp(-0.05 * hours_since_reinforced)
    return base * (0.3 + 0.7 * reinforcement) * (0.5 + 0.5 * recency)
```

### A.3 Vibe Shift Detection (Full)

```
function detect_vibe_shift(embeddings, window=500, ks_threshold=0.01, 
                           wasserstein_threshold=0.1):
    if len(embeddings) < 2 * window:
        return {"shift": false, "reason": "insufficient_data"}
    
    recent = embeddings[-window:]
    historical = embeddings[-2*window:-window]
    
    recent_dists = sample_pairwise_distances(recent, n=200)
    hist_dists = sample_pairwise_distances(historical, n=200)
    
    ks_stat, ks_p = scipy.stats.ks_2samp(recent_dists, hist_dists)
    w_dist = scipy.stats.wasserstein_distance(recent_dists, hist_dists)
    
    centroid_recent = mean(e.vector for e in recent)
    centroid_hist = mean(e.vector for e in historical)
    centroid_shift = cosine_distance(centroid_recent, centroid_hist)
    
    is_shift = (ks_p < ks_threshold) or (w_dist > wasserstein_threshold)
    
    return {
        "shift": is_shift,
        "ks_p_value": ks_p,
        "wasserstein": w_dist,
        "centroid_shift": centroid_shift,
        "recent_spread": std(recent_dists),
        "historical_spread": std(hist_dists),
        "spread_change": std(recent_dists) / std(hist_dists)
    }
```

### A.4 LoRA Trigger Check

```
function check_lora_trigger(room_state):
    # Novel pattern accumulator
    novel_count = room_state.embeddings_since_last_train(
        filter=lambda e: e.best_archetype_similarity < 0.85
    )
    
    # Cluster the novel patterns
    novel_embeddings = room_state.get_novel_embeddings(threshold=0.85)
    clusters = dbscan(novel_embeddings, eps=0.15, min_samples=5)
    
    # Accuracy check
    recent_accuracy = room_state.prediction_accuracy(window=100)
    baseline_accuracy = room_state.post_training_accuracy
    accuracy_drop = baseline_accuracy - recent_accuracy
    
    # Staleness check
    days_since = room_state.days_since_last_training
    
    # Decision
    triggers = {
        "novel_count_met": novel_count >= 50,
        "clusters_formed": len(clusters) >= 3,
        "accuracy_degraded": accuracy_drop >= 0.05,
        "min_interval_met": days_since >= 7
    }
    
    should_train = all([
        triggers["novel_count_met"],
        triggers["clusters_formed"],
        triggers["min_interval_met"],
        triggers["accuracy_degraded"]  # optional, depending on aggressiveness
    ])
    
    return {
        "should_train": should_train,
        "triggers": triggers,
        "novel_count": novel_count,
        "novel_clusters": len(clusters),
        "accuracy_drop": accuracy_drop,
        "days_since_training": days_since
    }
```

---

*End of report. Generated for the Plato Nervous System project, May 2026.*
