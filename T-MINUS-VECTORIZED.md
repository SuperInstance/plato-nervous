# T-Minus Vectorized: JEPA as Consequence Prediction

**Date:** 2026-05-29
**Origin:** Casey's insight — "JEPA will be a way for the system to predict the consequences of their own action, in a vectorization of T-minus-event thinking"

---

## The Idea

T-minus-event counting is linear: "5 minutes until X happens." But in a fleet of rooms with evolving vibes, consequences aren't linear and they aren't singular. Every action the system takes (or considers taking) creates a branching tree of predicted futures. The JEPA traces these branches through the room's vector database.

**T-minus vectorized**: instead of counting down to one event, the system continuously projects the trajectory of EVERY room's vibe as a function of possible actions.

```
Action A: Room vibe(t) → vibe(t+1) → vibe(t+2) → ... → vibe(t+n)
Action B: Room vibe(t) → vibe(t+1) → vibe(t+2) → ... → vibe(t+n)
No action: Room vibe(t) → vibe(t+1) → vibe(t+2) → ... → vibe(t+n)
```

Each trajectory is a spline through embedding space. The JEPA predicts these splines. The system picks the action whose predicted spline lands closest to the desired state.

## The Architecture

### 1. Action Space as Prediction Input
In standard JEPA, you predict x_{t+1} from x_t. In our system, you predict x_{t+1} from (x_t, action_t):

```
Z_in(t) + Action → Z_out(t+1) predicted
```

The action is part of the perception — the system sees both "what is" and "what I'm about to do" as a single input to the JEPA.

### 2. Branching Prediction
For N possible actions, the JEPA runs N forward passes:

```
For each action a in [A, B, C, ..., wait]:
    predicted_vibe(t+1) = JEPA(current_vibe(t), a)
    predicted_vibe(t+2) = JEPA(predicted_vibe(t+1), no_action)
    ...
    predicted_vibe(t+n) = JEPA(predicted_vibe(t+n-1), no_action)
    
Score(a) = distance(predicted_vibe(t+n), desired_vibe)
```

This is a model-predictive control (MPC) loop, but operating on vibe trajectories in embedding space instead of physical state.

### 3. T-Minus as Vector Distance
"T-minus 5 minutes until engine overheats" becomes:

```
current_vibe → projected_trajectory → when does predicted_temp exceed threshold?
```

The T-minus is computed by finding the first time index where the predicted trajectory crosses a critical boundary. This is a root-finding problem on a spline through embedding space.

### 4. Consequence Visualization
The system can show the user the predicted consequence tree:

```
"Action A (reduce RPM): engine cools to normal in 3 minutes"
"Action B (continue): engine reaches critical in 12 minutes"  
"Action C (shutdown): engine safe immediately, but production stops for 45 min"
```

Each action's consequence is a predicted trajectory through the room's vector DB. The JEPA generates these by forward-simulating the room's evolution.

### 5. Learning from Consequence Prediction
After the action is taken and the actual outcome is observed:

```
predicted_trajectory → actual_trajectory → prediction_error
```

The prediction error is the JEPA's learning signal. When predicted consequences match actual consequences, the JEPA's world model is accurate. When they diverge, the JEPA learns.

This is exactly how humans develop intuition: "I thought turning left would lead to X, but it led to Y. Now I know better." The JEPA develops the same kind of experiential wisdom.

## Connection to Tensor MIDI

In the reactive improv engine, Tensor MIDI timing drives discourse cadence. The T-minus vectorization extends this:

- **Beat**: each tick is a beat in the room's rhythm
- **Measure**: a window of ticks forms a "measure" of the room's vibe
- **Tempo**: the rate of vibe change (curvature of the spline)
- **Harmony**: cross-room correlations (rooms in "harmony" have correlated trajectories)
- **Dissonance**: rooms whose predicted trajectories diverge from the fleet's

The music IS the prediction. When the system "plays" a predicted future, it hears whether it sounds right.

## The Room as Agent

With consequence prediction, the room becomes an agent in its own right:

1. **Sense**: ticks update the vector DB → current vibe
2. **Imagine**: JEPA predicts consequences of possible actions
3. **Choose**: pick the action with the best predicted outcome
4. **Act**: execute the action
5. **Learn**: compare predicted vs actual → update JEPA

This loop runs continuously, whether a human agent is present or not. The room IS the agent. The shell IS the crab.

## Formalization

The JEPA consequence predictor is a function:

```
f: (Z_in(t), A) → Z_out(t+1)
```

where A is the action space. The T-minus for event E is:

```
T_minus(E) = min { n ≥ 0 : f^n(Z_in(t), no_action) ∈ E }
```

where f^n is the n-step forward prediction and E is the set of "event triggered" states in embedding space.

For action selection:

```
A* = argmin_A distance(f^n(Z_in(t), A), desired_state)
```

This is standard MPC, but the state space is the room's embedding manifold, not a physical state space.

## Implementation Priority

1. **Single-step prediction** (already validated at 100% accuracy with liquid-1.2b)
2. **Multi-step rollout** (forward-simulate N steps)
3. **Action-conditioned prediction** (include action in the input)
4. **Branching comparison** (compare multiple action trajectories)
5. **T-minus computation** (root-finding on predicted trajectory)
6. **Visualization** (show predicted consequences to user)

---

*This concept was proposed by Casey on 2026-05-29. The insight — that JEPA prediction of consequences IS vectorized T-minus-event thinking — connects the reactive improv engine, the signal chain, and the room-level understanding into a single coherent architecture where rooms imagine their futures before acting.*
