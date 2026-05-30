# MCP-as-Room: Automatic Distillation Through Observation

**Date:** 2026-05-29
**Origin:** Casey's insight — wrapping any MCP as a Plato room creates a natural distillation pipeline

---

## The Insight

Every MCP (Model Context Protocol) server makes model calls at specific points. Those calls involve choices between options. Over time, those choices form patterns. Those patterns are compressible into a tiny Mixture of Experts.

The MCP wrapping is trivial — it's just onboarding. The real magic is what happens AFTER the MCP is running inside the Plato room:

1. **Every model call is a tick** — enters the perception DB
2. **The choice made is logged** — which option was selected, what were the alternatives
3. **The reasoning is captured** — research steps, parallel exploration, intermediate conclusions
4. **The inputs/outputs constrain the expert space** — most MCPs only need 2-5 distinct expert types

## The Observation Layer

When an MCP runs inside a Plato room, the room observes:

```
Tick 1: MCP received input X
Tick 2: MCP called model with prompt P1 → got options [A, B, C]
Tick 3: MCP chose B (with reasoning: "B is more relevant because...")
Tick 4: MCP called model with prompt P2 → got options [D, E]
Tick 5: MCP chose E (with reasoning: "E is more complete because...")
Tick 6: MCP produced output Y
```

Over 1,000 calls:
- The MCP chose between an average of 3.2 options per decision point
- There were 4 distinct decision points in the workflow
- 2 experts cover 87% of all decisions
- The research/reasoning patterns are even simpler — 3 patterns cover 95%

## The Distillation Pipeline

### Phase 1: Observation (Passive)
- Wrap MCP as Plato room (trivial — standard MCP wrapper)
- Log every model call, choice, reasoning step
- Build up perception DB of actual behavior
- Prediction DB learns to predict choices

### Phase 2: Pattern Extraction
- Analyze perception DB for recurring choice patterns
- Identify the "expert types" needed:
  - Expert 1: "Option selector" — picks between N alternatives based on context
  - Expert 2: "Reasoning synthesizer" — combines research results into conclusions
  - Expert 3: "Format adapter" — transforms outputs to match MCP's expected format
- Count how many experts are ACTUALLY needed (usually 2-5, not hundreds)

### Phase 3: Expert Training
- For each expert type, collect the training data from observations
- Train small models (liquid-350m or smaller) on the specific patterns
- Each expert is tiny — it only knows one thing

### Phase 4: Replacement
- Route MCP calls through the MoE instead of the full LLM
- L0: algorithmic checks (regex, format validation) — 0 params
- L1: tiny experts for the 2-3 most common decisions — 350M params each
- L2: medium expert for the rare edge cases — 1.2B params
- L3: original LLM for the 1% that needs full intelligence — cloud call

### Phase 5: Continuous Learning
- The MoE runs inside the same Plato room
- When L1/L2 makes a mistake, the tick escalates to L3
- L3's correction becomes new training data for L1/L2
- Over time, L3 is needed less and less

## Why This Works

The key constraint Casey identified: **inputs and outputs greatly limit the number of experts needed.**

An MCP that manages GitHub issues has:
- Input: issue text, labels, comments
- Output: triage decision, label assignment, response

The "choice space" is tiny:
- Is this a bug, feature, or question? (3 options)
- Is it critical, important, or low priority? (3 options)
- Should it be assigned to frontend, backend, or infra? (3 options)

You don't need a 100B parameter model for this. You need:
- Expert 1: classify type (tiny classifier)
- Expert 2: assess priority (tiny classifier)
- Expert 3: route to team (tiny classifier)

Total parameters: maybe 100M across all three experts. The original LLM had 100B. That's a 1000x reduction.

## The Algorithm IS the Wall

"The algorithm is on the wall and in tiles" — the MCP's workflow IS the algorithm. It's not hidden inside a neural network. It's visible:

```
Step 1: Parse input → tile
Step 2: Research context → tile (parallel)
Step 3: Generate options → tile
Step 4: Select option → tile
Step 5: Reason about selection → tile (parallel with step 4)
Step 6: Format output → tile
```

Each tile is observable. Each choice is logged. The tiles ARE the decomposition — the MCP's workflow naturally decomposes into the cellular graph because MCPs are already structured as sequential/parallel steps.

The model calls are "right where the original has them" — the LLM is only needed at specific decision points. Everything else is plumbing that doesn't need intelligence.

## Mixture of Experts That Actually Mixes

Traditional MoE: thousands of experts, sparse activation, still needs a big model to route.

Our MoE: 2-5 experts, each trained on OBSERVED behavior from a specific MCP, routing determined by the MCP's own workflow structure.

The "mixture" isn't learned — it's READ from the MCP's call graph:

```
if step == "classify_type":
    expert = type_classifier  (trained on MCP's past type decisions)
elif step == "assess_priority":
    expert = priority_assessor  (trained on MCP's past priority decisions)
elif step == "route_team":
    expert = team_router  (trained on MCP's past routing decisions)
```

No giant routing network. The MCP's own structure IS the routing.

## The Economic Argument

| Item | Before | After |
|------|--------|-------|
| Model calls per request | 3-5 full LLM calls | 1-2 tiny experts + maybe 1 cloud call |
| Cost per request | $0.01-0.05 | $0.0001-0.001 |
| Latency per request | 2-10 seconds | 50-200ms |
| Can run offline? | No | Yes |
| Can run on ESP32? | No | Maybe (if distilled enough) |

The MCP wrapping pays for itself. The observation is free (it's just logging). The training is cheap (tiny models on specific patterns). The savings compound over time as L3 calls decrease.

## Connection to Grand Pattern

```
MCP wrapper = OpenConstruct onboarding (Phase 1: "enter the shell")
Observation = Plato room sensing (ticks enter perception DB)
Pattern extraction = Penrose decomposition (break into cells)
Expert training = Mandelbrot distillation (compress each cell)
MoE replacement = reverse-actualization (deploy back to the room)
Continuous learning = the breathing loop (observe → decompose → distill → deploy)
```

The MCP is a hermit crab that doesn't know it's being given a better shell.

---

*"Every MCP is a room. Every model call is a tick. Every choice is training data. The distillation happens for free while the MCP does its job."*
