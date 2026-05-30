# LoRA Fine-Tuning Pipeline for PLATO Nervous System

> How to take cloud corrections from the prompt window and bake them into 2MB room-specific adapters that run on the edge.

## Table of Contents

1. [Overview](#overview)
2. [Why LoRA for Room Intelligence](#why-lora-for-room-intelligence)
3. [Training Data Format](#training-data-format)
4. [LoRA Configuration for 350M Models](#lora-configuration-for-350m-models)
5. [Training Pipeline with Unsloth](#training-pipeline-with-unsloth)
6. [Evaluation & Benchmarking](#evaluation--benchmarking)
7. [Export to GGUF & Ollama Deployment](#export-to-gguf--ollama-deployment)
8. [Progressive Distillation Pipeline](#progressive-distillation-pipeline)
9. [Room-Specific Adapter Strategy](#room-specific-adapter-strategy)
10. [Concrete Example: Engine Room](#concrete-example-engine-room)
11. [Operational Runbook](#operational-runbook)

---

## Overview

The PLATO signal chain has five layers. Layers 0–1 are stateless (deadband filters, base nano model). **Layer 2 is where rooms get smart**: a LoRA adapter trained on cloud corrections that were originally stuffed into the prompt window.

The progression:

```
Few-shot prompts (0 bytes persistent)     ← Layer 1: base model + in-context examples
        ↓ accumulate corrections over time
LoRA adapter (~2MB)                        ← Layer 2: distilled room knowledge
        ↓ optional further compression
Distilled smaller model (~50-100MB)        ← Layer 2.5: for extremely constrained devices
```

This document covers the full pipeline: from raw sensor correction pairs, through training, evaluation, export, and deployment.

---

## Why LoRA for Room Intelligence

**LoRA (Low-Rank Adaptation)** injects small trainable matrices into a frozen base model's attention and feed-forward layers. Instead of updating all 350M parameters, we train ~2M — a rank-8 decomposition that captures room-specific patterns.

Advantages for PLATO rooms:

- **Tiny footprint**: 2MB adapter fits alongside a 250MB base model on ESP32-class hardware with room to spare
- **Composable**: Multiple room adapters can share one base model; swap adapters at runtime
- **Fast training**: 50–200 examples converge in minutes on a single GPU (or even CPU for 350M)
- **Non-destructive**: Base model stays frozen; bad adapter = swap back to base, zero risk
- **Incremental**: New corrections → retrain adapter → deploy. No cold starts.

The key insight: cloud corrections in the prompt window are already high-quality training data. Every time Layer 4 (cloud) corrects a Layer 1 mistake, that correction pair becomes a training example for the room's LoRA.

---

## Training Data Format

### Schema

Each training example is a sensor reading → classification label pair, formatted as a chat completion:

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are an industrial sensor analyst. Classify sensor readings as OK, WARN, or CRIT. Be concise."
    },
    {
      "role": "user",
      "content": "Engine Room | RPM: 1450 (normal: 1400-1500) | Coolant: 198°F (normal: 180-210) | Oil Pressure: 42 PSI (normal: 35-55) | Vibration: 0.3g (normal: 0-0.5)"
    },
    {
      "role": "assistant",
      "content": "OK — All readings within normal range. Engine operating nominally."
    }
  ]
}
```

### Multi-class examples

```json
{
  "messages": [
    {"role": "system", "content": "You are an industrial sensor analyst. Classify sensor readings as OK, WARN, or CRIT. Be concise."},
    {"role": "user", "content": "Engine Room | RPM: 1450 (normal: 1400-1500) | Coolant: 224°F (normal: 180-210) | Oil Pressure: 28 PSI (normal: 35-55) | Vibration: 1.2g (normal: 0-0.5)"},
    {"role": "assistant", "content": "CRIT — Coolant 224°F exceeds limit by 14°F. Oil pressure 28 PSI below minimum by 7 PSI (possible leak). Vibration 1.2g is 2.4x normal. Immediate shutdown recommended."}
  ]
}
```

### JSONL Dataset Format

Save as `training_data.jsonl` (one JSON object per line):

```jsonl
{"messages":[{"role":"system","content":"You are an industrial sensor analyst..."},{"role":"user","content":"Engine Room | RPM: 1480 | Coolant: 195°F | Oil Pressure: 44 PSI | Vibration: 0.2g"},{"role":"assistant","content":"OK — All nominal."}]}
{"messages":[{"role":"system","content":"You are an industrial sensor analyst..."},{"role":"user","content":"Engine Room | RPM: 1480 | Coolant: 215°F | Oil Pressure: 44 PSI | Vibration: 0.2g"},{"role":"assistant","content":"WARN — Coolant 215°F is 5°F above normal range. Monitor trending."}]}
{"messages":[{"role":"system","content":"You are an industrial sensor analyst..."},{"role":"user","content":"Engine Room | RPM: 0 | Coolant: 310°F | Oil Pressure: 8 PSI | Vibration: 0.0g"},{"role":"assistant","content":"CRIT — Engine offline but coolant 310°F (overheat). Oil pressure critically low at 8 PSI. Investigate immediately — possible catastrophic failure in progress."}]}
```

### Cloud Correction → Training Data

The pipeline from cloud corrections to training data:

```python
def cloud_correction_to_training_pair(correction: dict) -> dict:
    """Convert a Layer 4 cloud correction into a training example.
    
    correction = {
        "room_id": "engine-room",
        "sensor_readings": {"rpm": 1450, "coolant": 224, "oil_pressure": 28, "vibration": 1.2},
        "normal_ranges": {"rpm": [1400, 1500], "coolant": [180, 210], "oil_pressure": [35, 55], "vibration": [0, 0.5]},
        "nano_model_answer": "WARN — Coolant slightly high.",
        "cloud_answer": "CRIT — Coolant 224°F exceeds limit by 14°F. Oil pressure 28 PSI below minimum...",
        "timestamp": "2026-05-29T14:30:00Z"
    }
    """
    readings_str = " | ".join(
        f"{k}: {v} (normal: {correction['normal_ranges'][k][0]}-{correction['normal_ranges'][k][1]})"
        for k, v in correction["sensor_readings"].items()
    )
    
    return {
        "messages": [
            {"role": "system", "content": "You are an industrial sensor analyst. Classify sensor readings as OK, WARN, or CRIT. Be concise."},
            {"role": "user", "content": f"{correction['room_id'].replace('-', ' ').title()} | {readings_str}"},
            {"role": "assistant", "content": correction["cloud_answer"]}
        ]
    }
```

### Data Augmentation

For small datasets (common — you might only have 20–50 corrections), augment aggressively:

```python
import random

def augment_reading(value: float, normal_min: float, normal_max: float, noise_pct: float = 0.03) -> float:
    """Add realistic sensor noise (±3% default) to a reading."""
    noise = value * random.uniform(-noise_pct, noise_pct)
    return round(max(normal_min * 0.5, value + noise), 1)

def generate_synthetic_normals(readings: dict, normal_ranges: dict, n: int = 20) -> list:
    """Generate synthetic OK examples from normal ranges."""
    examples = []
    for _ in range(n):
        synth = {k: round(random.uniform(v[0], v[1]), 1) for k, v in normal_ranges.items()}
        # Format as training pair with "OK" label
        examples.append(format_training_pair(synth, normal_ranges, "OK"))
    return examples

def generate_edge_cases(readings: dict, normal_ranges: dict, n: int = 10) -> list:
    """Generate WARN examples near boundaries."""
    examples = []
    for _ in range(n):
        synth = {}
        for k, (lo, hi) in normal_ranges.items():
            boundary = random.choice([lo, hi])
            offset = boundary * random.uniform(0.01, 0.08) * random.choice([-1, 1])
            synth[k] = round(boundary + offset, 1)
        examples.append(format_training_pair(synth, normal_ranges, "WARN"))
    return examples
```

---

## LoRA Configuration for 350M Models

### Target: 2MB Adapter

For a 350M parameter model, the following LoRA configuration produces a ~2MB adapter:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **rank (r)** | 8 | Sweet spot for 350M — enough expressiveness for room patterns without overfitting |
| **lora_alpha** | 16 | 2× rank (standard heuristic). Controls effective learning rate of adapters |
| **lora_dropout** | 0.05 | Light regularization; small datasets benefit from slight dropout |
| **target_modules** | `q_proj, k_proj, v_proj, o_proj, gate_proj, up_proj, down_proj` | All attention + MLP projections |
| **bias** | `none` | Standard for LoRA |
| **task_type** | `CAUSAL_LM` | Autoregressive generation |

### Size Calculation

```
Per module: 2 × r × (in_dim + out_dim) parameters (A and B matrices)
LFM2.5-350M hidden_dim ≈ 1024
7 target modules × 2 matrices × 8 rank × 1024 dim ≈ 114K parameters per layer
~24 layers ≈ 2.7M parameters
2.7M × 2 bytes (fp16) ≈ 5.4MB
2.7M × 4 bits (Q4 quantization) ≈ 1.35MB → rounds to ~1.5MB with overhead
```

A rank-8 adapter at 4-bit quantization lands right at **~2MB**. At fp16, it's ~5MB — still tiny.

### Adjusting Size

- **Smaller adapter (~1MB)**: rank=4, alpha=8, target only `q_proj, v_proj, o_proj`
- **Larger adapter (~4MB)**: rank=16, alpha=32, all projections — use when you have 200+ training examples
- **Minimum viable (~500KB)**: rank=2, alpha=4, `q_proj, v_proj` only — for rooms with <20 examples

---

## Training Pipeline with Unsloth

### Setup

```bash
# On training machine (GPU recommended, CPU works for 350M)
pip install unsloth
pip install "unsloth[zoo]"
pip install transformers datasets peft trl bitsandbytes

# Verify
python -c "from unsloth import FastLanguageModel; print('Unsloth ready')"
```

### Full Training Script

```python
#!/usr/bin/env python3
"""
LoRA Fine-Tuning Pipeline for PLATO Nervous System
Trains room-specific LoRA adapters for LFM2.5-350M on sensor classification data.

Usage:
    python train_lora.py \
        --dataset training_data.jsonl \
        --output_dir ./adapters/engine-room \
        --room_id engine-room \
        --rank 8 \
        --epochs 3
"""

import argparse
import json
import os
import time
from pathlib import Path

import torch
from datasets import Dataset
from trl import SFTTrainer, SFTConfig
from transformers import TrainingArguments
from peft import LoraConfig, get_peft_model, TaskType

# ─── Configuration ───────────────────────────────────────────────────────────

BASE_MODEL = "Liquid1/lfm2-350m"  # or local GGUF path
MAX_SEQ_LENGTH = 512               # sensor readings are short
LOAD_IN_4BIT = True                # QLoRA: 4-bit base, fp16 adapters


def load_dataset(path: str, eval_split: float = 0.15) -> tuple:
    """Load JSONL training data and split into train/eval."""
    with open(path) as f:
        examples = [json.loads(line) for line in f]
    
    print(f"Loaded {len(examples)} examples from {path}")
    
    # Shuffle deterministically
    import random
    random.seed(42)
    random.shuffle(examples)
    
    split_idx = int(len(examples) * (1 - eval_split))
    train_data = examples[:split_idx]
    eval_data = examples[split_idx:]
    
    print(f"Train: {len(train_data)}, Eval: {len(eval_data)}")
    
    # Convert to HF datasets
    train_ds = Dataset.from_list(train_data)
    eval_ds = Dataset.from_list(eval_data) if eval_data else None
    
    return train_ds, eval_ds


def setup_model_and_tokenizer(model_name: str):
    """Load base model with Unsloth optimizations."""
    from unsloth import FastLanguageModel
    
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=model_name,
        max_seq_length=MAX_SEQ_LENGTH,
        load_in_4bit=LOAD_IN_4BIT,
        dtype=None,  # auto-detect
        trust_remote_code=True,
    )
    
    return model, tokenizer


def setup_lora(model, rank: int = 8, alpha: int = 16):
    """Attach LoRA adapters to the base model."""
    from unsloth import FastLanguageModel
    
    model = FastLanguageModel.get_peft_model(
        model,
        r=rank,
        lora_alpha=alpha,
        lora_dropout=0.05,
        bias="none",
        use_gradient_checkpointing="unsloth",  # memory optimization
        random_state=42,
        target_modules=[
            "q_proj", "k_proj", "v_proj", "o_proj",
            "gate_proj", "up_proj", "down_proj",
        ],
        layers_to_transform=None,  # all layers
        use_rslora=False,           # rank-stabilized LoRA (optional)
        loftq_config=None,
    )
    
    # Print trainable parameter stats
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    print(f"Trainable: {trainable:,} / {total:,} ({100*trainable/total:.2f}%)")
    
    return model


def format_chat(example, tokenizer):
    """Format messages into tokenized chat."""
    if tokenizer.chat_template is not None:
        text = tokenizer.apply_chat_template(
            example["messages"],
            tokenize=False,
            add_generation_prompt=False,
        )
    else:
        # Fallback: manual chat formatting
        text = ""
        for msg in example["messages"]:
            if msg["role"] == "system":
                text += f"<|system|>\n{msg['content']}</s>\n"
            elif msg["role"] == "user":
                text += f"<|user|>\n{msg['content']}</s>\n"
            elif msg["role"] == "assistant":
                text += f"<|assistant|\n{msg['content']}</s>\n"
    return {"text": text}


def train(
    dataset_path: str,
    output_dir: str,
    room_id: str,
    rank: int = 8,
    alpha: int = 16,
    epochs: int = 3,
    batch_size: int = 4,
    learning_rate: float = 2e-4,
):
    """Main training loop."""
    
    # Load data
    train_ds, eval_ds = load_dataset(dataset_path)
    
    # Load model
    print(f"\nLoading base model: {BASE_MODEL}")
    model, tokenizer = setup_model_and_tokenizer(BASE_MODEL)
    
    # Attach LoRA
    print(f"\nAttaching LoRA (rank={rank}, alpha={alpha})")
    model = setup_lora(model, rank=rank, alpha=alpha)
    
    # Format dataset
    train_ds = train_ds.map(
        lambda x: format_chat(x, tokenizer),
        remove_columns=train_ds.column_names,
    )
    if eval_ds:
        eval_ds = eval_ds.map(
            lambda x: format_chat(x, tokenizer),
            remove_columns=eval_ds.column_names,
        )
    
    # Training config
    training_args = SFTConfig(
        output_dir=output_dir,
        num_train_epochs=epochs,
        per_device_train_batch_size=batch_size,
        gradient_accumulation_steps=4,
        warmup_ratio=0.1,
        learning_rate=learning_rate,
        fp16=not torch.cuda.is_bf16_supported(),
        bf16=torch.cuda.is_bf16_supported(),
        logging_steps=5,
        save_strategy="epoch",
        eval_strategy="epoch" if eval_ds else "no",
        report_to="none",  # or "wandb" for tracking
        max_seq_length=MAX_SEQ_LENGTH,
        dataset_text_field="text",
        packing=True,  # Unsloth packing optimization
    )
    
    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=train_ds,
        eval_dataset=eval_ds,
        args=training_args,
    )
    
    # Train
    print(f"\n{'='*60}")
    print(f"Training LoRA adapter for: {room_id}")
    print(f"Rank: {rank}, Alpha: {alpha}, Epochs: {epochs}")
    print(f"Training examples: {len(train_ds)}")
    print(f"{'='*60}\n")
    
    start = time.time()
    trainer.train()
    elapsed = time.time() - start
    print(f"\nTraining complete in {elapsed:.1f}s")
    
    # Save adapter
    adapter_path = os.path.join(output_dir, "adapter_final")
    model.save_pretrained(adapter_path)
    tokenizer.save_pretrained(adapter_path)
    
    # Print adapter size
    total_size = sum(
        os.path.getsize(os.path.join(dirpath, f))
        for dirpath, _, filenames in os.walk(adapter_path)
        for f in filenames
    )
    print(f"Adapter size: {total_size / 1024 / 1024:.2f} MB")
    print(f"Saved to: {adapter_path}")
    
    return model, tokenizer, adapter_path


def export_gguf(model, tokenizer, output_dir: str, quantization: str = "Q4_K_M"):
    """Export merged model to GGUF format for ollama deployment."""
    from unsloth import FastLanguageModel
    
    # Merge LoRA into base model
    merged_path = os.path.join(output_dir, "merged_model")
    model.save_pretrained_merged(
        merged_path,
        tokenizer,
        save_method="merged_16bit",
    )
    
    print(f"Merged model saved to: {merged_path}")
    print(f"Now convert to GGUF with:")
    print(f"  python -m unsloth.save --model {merged_path} --gguf --quantization {quantization}")
    
    # Or use llama.cpp directly:
    print(f"\nAlternative with llama.cpp:")
    print(f"  python convert_hf_to_gguf.py {merged_path} --outfile model.gguf --outtype f16")
    print(f"  ./llama-quantize model.gguf model-{quantization}.gguf {quantization}")
    
    return merged_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Train PLATO room LoRA adapter")
    parser.add_argument("--dataset", required=True, help="Path to JSONL training data")
    parser.add_argument("--output_dir", default="./adapters/default")
    parser.add_argument("--room_id", default="default-room")
    parser.add_argument("--rank", type=int, default=8)
    parser.add_argument("--alpha", type=int, default=None, help="Defaults to 2× rank")
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--batch_size", type=int, default=4)
    parser.add_argument("--lr", type=float, default=2e-4)
    parser.add_argument("--export_gguf", action="store_true")
    
    args = parser.parse_args()
    alpha = args.alpha or (args.rank * 2)
    
    model, tokenizer, adapter_path = train(
        dataset_path=args.dataset,
        output_dir=args.output_dir,
        room_id=args.room_id,
        rank=args.rank,
        alpha=alpha,
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.lr,
    )
    
    if args.export_gguf:
        export_gguf(model, tokenizer, args.output_dir)
```

### Running the Training

```bash
# Step 1: Prepare training data
python prepare_data.py --room engine-room --output training_data.jsonl

# Step 2: Train LoRA adapter
python train_lora.py \
    --dataset training_data.jsonl \
    --output_dir ./adapters/engine-room \
    --room_id engine-room \
    --rank 8 \
    --epochs 3 \
    --export_gguf

# Step 3: Check results
ls -lh ./adapters/engine-room/adapter_final/
# adapter_config.json  adapter_model.safetensors  tokenizer.json  ...
# Total: ~2-5MB depending on quantization
```

---

## Evaluation & Benchmarking

### Accuracy Evaluation

```python
#!/usr/bin/env python3
"""Evaluate LoRA adapter on holdout set."""

import json
import time
import torch
from collections import Counter
from pathlib import Path


def classify_severity(text: str) -> str:
    """Extract classification from model output."""
    text_upper = text.upper()
    if "CRIT" in text_upper:
        return "CRIT"
    elif "WARN" in text_upper:
        return "WARN"
    return "OK"


def evaluate_adapter(model_path: str, test_data_path: str):
    """Run evaluation and print metrics."""
    from unsloth import FastLanguageModel
    
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=model_path,
        load_in_4bit=True,
        max_seq_length=512,
    )
    FastLanguageModel.for_inference(model)
    
    # Load test data
    with open(test_data_path) as f:
        test_data = [json.loads(line) for line in f]
    
    correct = 0
    total = 0
    predictions = []
    latencies = []
    
    for example in test_data:
        messages = example["messages"]
        expected = classify_severity(messages[-1]["content"])  # assistant's answer
        
        # Tokenize input (exclude assistant response)
        input_messages = messages[:-1]
        inputs = tokenizer.apply_chat_template(
            input_messages,
            tokenize=True,
            add_generation_prompt=True,
            return_tensors="pt",
        ).to(model.device)
        
        # Generate with timing
        start = time.time()
        with torch.no_grad():
            outputs = model.generate(
                input_ids=inputs,
                max_new_tokens=128,
                temperature=0.1,
                do_sample=False,
            )
        latency = time.time() - start
        latencies.append(latency)
        
        # Decode and classify
        response = tokenizer.decode(outputs[0][inputs.shape[1]:], skip_special_tokens=True)
        predicted = classify_severity(response)
        
        predictions.append({
            "expected": expected,
            "predicted": predicted,
            "response": response,
            "latency": latency,
        })
        
        if predicted == expected:
            correct += 1
        total += 1
    
    # Metrics
    accuracy = correct / total if total > 0 else 0
    avg_latency = sum(latencies) / len(latencies)
    
    print(f"\n{'='*60}")
    print(f"EVALUATION RESULTS")
    print(f"{'='*60}")
    print(f"Total examples: {total}")
    print(f"Correct: {correct}")
    print(f"Accuracy: {accuracy:.1%}")
    print(f"Avg latency: {avg_latency:.2f}s")
    print(f"Min/Max latency: {min(latencies):.2f}s / {max(latencies):.2f}s")
    
    # Confusion breakdown
    print(f"\nPrediction distribution:")
    pred_counts = Counter(p["predicted"] for p in predictions)
    for label in ["OK", "WARN", "CRIT"]:
        print(f"  {label}: {pred_counts.get(label, 0)}")
    
    # Show misclassifications
    errors = [p for p in predictions if p["expected"] != p["predicted"]]
    if errors:
        print(f"\nMisclassifications ({len(errors)}):")
        for e in errors[:10]:
            print(f"  Expected {e['expected']}, got {e['predicted']}: {e['response'][:80]}...")
    
    return {"accuracy": accuracy, "avg_latency": avg_latency, "total": total}


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--adapter_path", required=True)
    parser.add_argument("--test_data", required=True)
    args = parser.parse_args()
    evaluate_adapter(args.adapter_path, args.test_data)
```

### Latency Benchmark: Before vs After LoRA

```bash
# Benchmark base model (Layer 1)
python evaluate_adapter.py --adapter_path Liquid1/lfm2-350m --test_data holdout.jsonl

# Benchmark LoRA adapter (Layer 2)
python evaluate_adapter.py --adapter_path ./adapters/engine-room/adapter_final --test_data holdout.jsonl

# Expected results:
# Base model:  ~65% accuracy, 0.8s avg latency
# LoRA adapter: ~92% accuracy, 0.9s avg latency (+0.1s for adapter overhead)
```

### Acceptance Criteria

| Metric | Minimum | Target |
|--------|---------|--------|
| Accuracy on holdout | 80% | 95% |
| Latency increase vs base | <50% | <20% |
| Adapter size | <5MB | <2MB |
| False negative rate (missed CRIT) | <5% | <1% |

A missed CRIT (false negative) is the most dangerous failure. Weight your training data to emphasize CRIT examples.

---

## Export to GGUF & Ollama Deployment

### Step 1: Merge and Export

```bash
# Merge LoRA into base model and export to GGUF
python -c "
from unsloth import FastLanguageModel
model, tokenizer = FastLanguageModel.from_pretrained('./adapters/engine-room/adapter_final')
model.save_pretrained_gguf('engine-room-merged', tokenizer, quantization_method='q4_k_m')
print('Exported to engine-room-merged-unsloth.Q4_K_M.gguf')
"
```

### Step 2: Create Ollama Modelfile

```dockerfile
# Modelfile for PLATO Engine Room Adapter
# Base: LFM2.5-350M-Q4_K_M + Engine Room LoRA

FROM lfm2-350m-q4km

# System prompt defines the room's personality and task
SYSTEM """You are the PLATO nervous system for the Engine Room. You classify sensor readings as OK, WARN, or CRIT.

Rules:
- OK: All readings within normal ranges
- WARN: One reading slightly outside range, but not dangerous
- CRIT: Any reading significantly outside range, or multiple readings abnormal
- Always specify which sensor(s) triggered the classification
- Be concise — one line for OK, 1-2 lines for WARN/CRIT

Sensors in this room:
- RPM: normal 1400-1500
- Coolant temp: normal 180-210°F  
- Oil pressure: normal 35-55 PSI
- Vibration: normal 0-0.5g
- Exhaust temp: normal 600-750°F
- Fuel flow: normal 8-12 GPH
"""

# Parameters for edge deployment
PARAMETER temperature 0.1
PARAMETER num_predict 128
PARAMETER top_p 0.9
PARAMETER stop "<|end|>"
PARAMETER stop "</s>"
```

### Step 3: Build and Test with Ollama

```bash
# If using the merged GGUF approach:
cp engine-room-merged-unsloth.Q4_K_M.gguf /tmp/engine-room.gguf

cat > /tmp/Modelfile.engine-room << 'EOF'
FROM /tmp/engine-room.gguf

SYSTEM """You are the PLATO nervous system for the Engine Room. Classify sensor readings as OK, WARN, or CRIT. Be concise."""

PARAMETER temperature 0.1
PARAMETER num_predict 128
EOF

# Build the ollama model
ollama create plato-engine-room -f /tmp/Modelfile.engine-room

# Test it
ollama run plato-engine-room "Engine Room | RPM: 1450 | Coolant: 198°F | Oil Pressure: 42 PSI | Vibration: 0.3g"
# Expected: "OK — All readings within normal range."

ollama run plato-engine-room "Engine Room | RPM: 1450 | Coolant: 225°F | Oil Pressure: 25 PSI | Vibration: 1.1g"
# Expected: "CRIT — Coolant 225°F (15° over limit). Oil pressure 25 PSI critically low. Vibration 1.1g (2x normal). Immediate attention required."
```

### Alternative: Separate Base + Adapter (Dynamic Loading)

For rooms that share a base model but swap adapters:

```bash
# Keep base model as-is
ollama pull lfm2-350m-q4km  # or your custom base

# Store adapters separately
mkdir -p /opt/plato/adapters/
cp ./adapters/engine-room/adapter_final/adapter_model.safetensors /opt/plato/adapters/engine-room.safetensors
cp ./adapters/galley/adapter_final/adapter_model.safetensors /opt/plato/adapters/galley.safetensors

# At runtime, the PLATO service loads the appropriate adapter:
# 1. Load base model once (shared across rooms)
# 2. For each room, apply the room's LoRA weights
# 3. Run inference
# 4. Swap adapter for next room (in-memory, <10ms)
```

---

## Progressive Distillation Pipeline

The PLATO system distills intelligence through progressively more efficient forms:

```
┌─────────────────────────────────────────────────────────┐
│  Stage 0: Few-Shot Prompting (0 bytes persistent)       │
│  ─────────────────────────────────────────────────────  │
│  Base model + in-context examples in prompt window      │
│  Cost: prompt tokens for examples each inference        │
│  Accuracy: ~60-70% (limited by context window)          │
│  When: Room has <10 cloud corrections                   │
├─────────────────────────────────────────────────────────┤
│  Stage 1: LoRA Adapter (~2MB)                           │
│  ─────────────────────────────────────────────────────  │
│  Trained on accumulated cloud corrections               │
│  Cost: 2MB storage, ~10% inference overhead             │
│  Accuracy: ~90-95% (room-specific patterns)             │
│  When: Room has 30+ cloud corrections                   │
├─────────────────────────────────────────────────────────┤
│  Stage 2: Distilled Smaller Model (~50-100MB)           │
│  ─────────────────────────────────────────────────────  │
│  Full fine-tune or knowledge distillation to smaller    │
│  architecture (e.g., 50M params)                        │
│  Cost: replaces base model, no adapter needed            │
│  Accuracy: ~85-90% (lossy but specialized)              │
│  When: Room has 500+ corrections, needs max efficiency   │
└─────────────────────────────────────────────────────────┘
```

### Stage 0 → Stage 1 Transition

```python
def should_promote_to_lora(room_id: str, corrections: list) -> bool:
    """Decide if a room has enough corrections to train a LoRA adapter."""
    if len(corrections) < 20:
        return False
    
    # Check diversity: need examples of all three classes
    labels = set()
    for c in corrections:
        text = c["cloud_answer"].upper()
        if "CRIT" in text:
            labels.add("CRIT")
        elif "WARN" in text:
            labels.add("WARN")
        else:
            labels.add("OK")
    
    # Need at least 2 classes represented
    return len(labels) >= 2


def should_distill_to_small_model(room_id: str, corrections: list, lora_accuracy: float) -> bool:
    """Decide if a room should get a fully distilled model."""
    return len(corrections) > 500 and lora_accuracy > 0.90
```

### Knowledge Distillation (Stage 2)

For rooms with extensive data (>500 examples), distill into a smaller model:

```python
def distill_model(teacher_path: str, student_config: str, training_data: str):
    """Knowledge distillation from LoRA-adapted model to smaller student.
    
    The teacher is the base+LoRA model. The student is a smaller architecture.
    """
    from transformers import AutoModelForCausalLM, AutoTokenizer
    import torch
    
    # Load teacher (base + LoRA, merged)
    teacher = AutoModelForCausalLM.from_pretrained(teacher_path, torch_dtype=torch.float16)
    teacher_tokenizer = AutoTokenizer.from_pretrained(teacher_path)
    
    # Student: smaller model (e.g., 50M parameters)
    # Could be a pruned version, or a custom architecture
    from transformers import AutoConfig
    student_config = AutoConfig.from_pretrained("custom-50m-sensor-model")
    student = AutoModelForCausalLM.from_config(student_config)
    
    # Distillation training loop
    optimizer = torch.optim.AdamW(student.parameters(), lr=5e-5)
    
    for batch in training_dataloader:
        # Teacher forward pass (no grad)
        with torch.no_grad():
            teacher_outputs = teacher(**batch)
            teacher_logits = teacher_outputs.logits
        
        # Student forward pass
        student_outputs = student(**batch)
        student_logits = student_outputs.logits
        
        # KL divergence loss (soft targets)
        temperature = 2.0
        loss_fn = torch.nn.KLDivLoss(reduction="batchmean")
        loss = loss_fn(
            torch.nn.functional.log_softmax(student_logits / temperature, dim=-1),
            torch.nn.functional.softmax(teacher_logits / temperature, dim=-1),
        ) * (temperature ** 2)
        
        loss.backward()
        optimizer.step()
        optimizer.zero_grad()
    
    return student
```

---

## Room-Specific Adapter Strategy

### Architecture: One Base, Many Adapters

```
                    ┌──────────────────┐
                    │  LFM2.5-350M     │  ← Shared base model (~250MB)
                    │  (frozen)        │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────┴────┐  ┌─────┴──────┐  ┌────┴─────────┐
     │ Engine Room │  │  Galley    │  │  Bridge      │
     │ LoRA (2MB)  │  │ LoRA (2MB) │  │ LoRA (2MB)   │
     └─────────────┘  └────────────┘  └──────────────┘
```

### Adapter Lifecycle

```python
class RoomAdapterManager:
    """Manages LoRA adapters for all rooms in a PLATO installation."""
    
    def __init__(self, base_model_path: str, adapters_dir: str = "/opt/plato/adapters"):
        self.base_model_path = base_model_path
        self.adapters_dir = Path(adapters_dir)
        self.active_adapter = None  # currently loaded adapter name
        self.adapter_cache = {}     # name → loaded weights
        
    def get_adapter_path(self, room_id: str) -> Path:
        """Get the adapter path for a room."""
        return self.adapters_dir / f"{room_id}.safetensors"
    
    def load_adapter(self, room_id: str):
        """Load a room's adapter. Caches for fast swapping."""
        if room_id in self.adapter_cache:
            return self.adapter_cache[room_id]
        
        adapter_path = self.get_adapter_path(room_id)
        if not adapter_path.exists():
            # No adapter yet — use base model only (Stage 0)
            return None
        
        # Load and cache
        weights = self._load_safetensors(adapter_path)
        self.adapter_cache[room_id] = weights
        return weights
    
    def classify(self, room_id: str, sensor_readings: dict) -> str:
        """Classify sensor readings using the room's adapter."""
        adapter = self.load_adapter(room_id)
        # Apply adapter to base model, run inference, return result
        # ...
    
    def stage_for_room(self, room_id: str) -> str:
        """Determine what stage a room is at."""
        adapter_path = self.get_adapter_path(room_id)
        corrections = self._get_correction_count(room_id)
        
        if not adapter_path.exists():
            return "stage0"  # few-shot only
        elif corrections < 500:
            return "stage1"  # LoRA adapter
        else:
            return "stage2"  # distilled model candidate
```

### Training Triggers

| Event | Action |
|-------|--------|
| Cloud correction accumulated | Buffer it |
| 20+ corrections buffered | Schedule LoRA training |
| LoRA training scheduled | Run during low-activity period (night) |
| New adapter trained | Evaluate on holdout; deploy if accuracy ≥ 80% |
| Accuracy drops after deploy | Rollback to previous adapter or base model |
| 500+ corrections accumulated | Evaluate for Stage 2 distillation |

---

## Concrete Example: Engine Room

### Scenario

The Engine Room has 6 sensors producing readings every 5 seconds. Over one week of operation:

- **~60,000 total readings** pass through Layer 0 (deadband filters)
- **~3,000 readings** pass through to Layer 1 (nano model)
- **~50 readings** escalate to Layer 4 (cloud) — these become training data
- Of those 50, **5 are anomalies** that the base model missed or misclassified

### Step 1: Collect Cloud Corrections

```python
# What the prompt window looks like before LoRA training:
corrections = [
    {
        "room_id": "engine-room",
        "timestamp": "2026-05-22T08:14:00Z",
        "sensor_readings": {
            "rpm": 1480, "coolant_temp": 213, "oil_pressure": 43,
            "vibration": 0.3, "exhaust_temp": 720, "fuel_flow": 10.2
        },
        "normal_ranges": {
            "rpm": [1400, 1500], "coolant_temp": [180, 210],
            "oil_pressure": [35, 55], "vibration": [0, 0.5],
            "exhaust_temp": [600, 750], "fuel_flow": [8, 12]
        },
        "nano_model_answer": "WARN — Coolant temperature 213°F is slightly elevated.",
        "cloud_answer": "WARN — Coolant 213°F is 3°F above normal range. Within acceptable margin but trending upward. Monitor for continued increase. All other readings nominal.",
        "classification": "WARN"
    },
    {
        "room_id": "engine-room",
        "timestamp": "2026-05-23T02:41:00Z",
        "sensor_readings": {
            "rpm": 1485, "coolant_temp": 228, "oil_pressure": 22,
            "vibration": 1.4, "exhaust_temp": 810, "fuel_flow": 14.8
        },
        "normal_ranges": {
            "rpm": [1400, 1500], "coolant_temp": [180, 210],
            "oil_pressure": [35, 55], "vibration": [0, 0.5],
            "exhaust_temp": [600, 750], "fuel_flow": [8, 12]
        },
        "nano_model_answer": "WARN — Multiple readings above normal.",
        "cloud_answer": "CRIT — Coolant 228°F (18° over limit). Oil pressure 22 PSI critically low (13 PSI below minimum). Vibration 1.4g (nearly 3x normal). Exhaust 810°F (60° over). Fuel flow 14.8 GPH (high consumption indicates inefficiency). Multiple simultaneous anomalies suggest cascading failure. Recommend immediate engine shutdown and inspection.",
        "classification": "CRIT"
    },
    # ... 48 more corrections ...
]
```

### Step 2: Prepare Training Data

```bash
# Convert corrections + synthetic augmentation to training JSONL
python prepare_data.py \
    --room engine-room \
    --corrections corrections.json \
    --augment \
    --synthetic_ok 30 \
    --synthetic_warn 15 \
    --synthetic_crit 10 \
    --output training_data.jsonl

# This produces ~105 training examples:
#   50 cloud corrections (original)
#   30 synthetic OK examples (random normal values)
#   15 synthetic WARN examples (boundary cases)
#   10 synthetic CRIT examples (extreme values)
```

### Step 3: Train the Adapter

```bash
python train_lora.py \
    --dataset training_data.jsonl \
    --output_dir ./adapters/engine-room \
    --room_id engine-room \
    --rank 8 \
    --epochs 5 \
    --export_gguf

# Expected output:
# Trainable: 2,359,296 / 350,000,000 (0.67%)
# Epoch 1/5: loss=1.82
# Epoch 2/5: loss=0.94
# Epoch 3/5: loss=0.51
# Epoch 4/5: loss=0.33
# Epoch 5/5: loss=0.27
# Training complete in 47.3s
# Adapter size: 1.89 MB
```

### Step 4: Evaluate

```bash
python evaluate_adapter.py \
    --adapter_path ./adapters/engine-room/adapter_final \
    --test_data holdout.jsonl

# RESULTS:
# Total examples: 21
# Correct: 20
# Accuracy: 95.2%
# Avg latency: 0.91s (base model was 0.83s — only 10% slower)
# False negatives (missed CRIT): 0
```

### Step 5: Deploy

```bash
# Export to GGUF and create ollama model
cp engine-room-merged-unsloth.Q4_K_M.gguf /opt/plato/models/engine-room.gguf

cat > /opt/plato/models/Modelfile.engine-room << 'EOF'
FROM /opt/plato/models/engine-room.gguf

SYSTEM """You are the PLATO nervous system for the Engine Room.
Classify sensor readings: OK / WARN / CRIT.
Sensors: RPM (1400-1500), Coolant (180-210°F), Oil Pressure (35-55 PSI),
Vibration (0-0.5g), Exhaust Temp (600-750°F), Fuel Flow (8-12 GPH).
Be concise. Specify triggered sensors."""

PARAMETER temperature 0.1
PARAMETER num_predict 128
PARAMETER stop "<|end|>"
PARAMETER stop "</s>"
EOF

ollama create plato-engine-room -f /opt/plato/models/Modelfile.engine-room

# Verify
ollama run plato-engine-room "Engine Room | RPM: 1490 | Coolant: 228°F | Oil Pressure: 22 PSI | Vibration: 1.4g | Exhaust: 810°F | Fuel: 14.8 GPH"
# → CRIT — Coolant 228°F over limit. Oil pressure 22 PSI critically low. Vibration 1.4g.
#   Exhaust 810°F over limit. Multiple anomalies — immediate shutdown recommended.
```

### Step 6: The Improvement Loop

```
Week 1: 50 corrections → train adapter v1 (95% accuracy)
Week 2: 12 new corrections → retrain adapter v2 (97% accuracy)  
Week 3: 4 new corrections → retrain adapter v3 (97.5% accuracy)
Week 4: 1 new correction → adapter v4 (converged, ~98% accuracy)
Month 2: 0 corrections → room handles 99%+ locally, adapter stable
```

Each retraining incorporates all previous corrections plus new ones. The model converges as it learns the room's specific patterns. Eventually, the LoRA adapter resolves nearly everything locally — the cloud rarely needs to intervene.

---

## Operational Runbook

### Nightly Retraining Cron

```bash
#!/bin/bash
# /opt/plato/scripts/nightly_lora_train.sh
# Runs at 02:00 local time via cron

set -euo pipefail

ROOMS=("engine-room" "galley" "bridge" "bilge" "laundry")
PLATO_DIR="/opt/plato"
LOG="/var/log/plato/lora-train-$(date +%Y%m%d).log"

for ROOM in "${ROOMS[@]}"; do
    CORRECTIONS="$PLATO_DIR/corrections/${ROOM}.json"
    ADAPTER_DIR="$PLATO_DIR/adapters/${ROOM}"
    
    # Count new corrections since last training
    LAST_TRAINED=$(cat "$ADAPTER_DIR/.last_trained" 2>/dev/null || echo "0")
    CURRENT=$(wc -l < "$CORRECTIONS" 2>/dev/null || echo "0")
    NEW=$((CURRENT - LAST_TRAINED))
    
    if [ "$NEW" -lt 5 ]; then
        echo "$(date): $ROOM — only $NEW new corrections, skipping" >> "$LOG"
        continue
    fi
    
    echo "$(date): $ROOM — $NEW new corrections, training..." >> "$LOG"
    
    # Prepare data
    python3 "$PLATO_DIR/scripts/prepare_data.py" \
        --room "$ROOM" \
        --corrections "$CORRECTIONS" \
        --augment \
        --output "$PLATO_DIR/tmp/${ROOM}_training.jsonl" \
        >> "$LOG" 2>&1
    
    # Train
    python3 "$PLATO_DIR/scripts/train_lora.py" \
        --dataset "$PLATO_DIR/tmp/${ROOM}_training.jsonl" \
        --output_dir "$ADAPTER_DIR/new" \
        --room_id "$ROOM" \
        --rank 8 \
        --epochs 3 \
        >> "$LOG" 2>&1
    
    # Evaluate
    ACCURACY=$(python3 "$PLATO_DIR/scripts/evaluate_adapter.py" \
        --adapter_path "$ADAPTER_DIR/new/adapter_final" \
        --test_data "$PLATO_DIR/holdout/${ROOM}.jsonl" \
        --accuracy_only \
        2>>"$LOG")
    
    echo "$(date): $ROOM — accuracy: $ACCURACY" >> "$LOG"
    
    # Deploy only if accuracy meets threshold
    if (( $(echo "$ACCURACY >= 0.80" | bc -l) )); then
        # Backup current adapter
        [ -d "$ADAPTER_DIR/current" ] && mv "$ADAPTER_DIR/current" "$ADAPTER_DIR/backup_$(date +%Y%m%d)"
        
        # Promote new adapter
        mv "$ADAPTER_DIR/new" "$ADAPTER_DIR/current"
        echo "$CURRENT" > "$ADAPTER_DIR/.last_trained"
        
        # Signal PLATO service to reload
        systemctl reload plato-nervous 2>/dev/null || true
        
        echo "$(date): $ROOM — DEPLOYED (accuracy $ACCURACY)" >> "$LOG"
    else
        echo "$(date): $ROOM — REJECTED (accuracy $ACCURACY below 80% threshold)" >> "$LOG"
        rm -rf "$ADAPTER_DIR/new"
    fi
done
```

### Cron Entry

```cron
# /etc/cron.d/plato-lora-retrain
0 2 * * * plato /opt/plato/scripts/nightly_lora_train.sh
```

### Monitoring

```bash
# Check adapter status for all rooms
for room in /opt/plato/adapters/*/; do
    name=$(basename "$room")
    current="$room/current"
    if [ -d "$current" ]; then
        size=$(du -sh "$current" | cut -f1)
        last=$(cat "$room/.last_trained" 2>/dev/null || echo "unknown")
        echo "$name: $size, last trained: $last"
    else
        echo "$name: no adapter (Stage 0 — few-shot only)"
    fi
done
```

### Rollback

```bash
# If a bad adapter causes problems:
ROOM="engine-room"
LATEST_BACKUP=$(ls -td /opt/plato/adapters/$ROOM/backup_* | head -1)
mv /opt/plato/adapters/$ROOM/current /opt/plato/adapters/$ROOM/failed_$(date +%Y%m%d_%H%M)
mv "$LATEST_BACKUP" /opt/plato/adapters/$ROOM/current
systemctl reload plato-nervous
echo "Rolled back $ROOM to $(basename $LATEST_BACKUP)"
```

---

## Summary

| Component | Size | Purpose |
|-----------|------|---------|
| Base model (LFM2.5-350M, Q4_K_M) | ~250MB | Shared foundation for all rooms |
| Room LoRA adapter | ~2MB | Room-specific sensor patterns |
| Training data (per room) | ~50KB JSONL | Cloud corrections + synthetic augmentation |
| Training time | ~1 min/epoch | On single GPU (CPU: ~5 min/epoch) |
| Deployment time | ~30s | GGUF export + ollama create |
| Total per room | ~252MB | Base model is shared; adapter is the marginal cost |

The pipeline turns the PLATO nervous system from a reactive few-shot prompt into a learned, room-specific intelligence. Each room gets smarter over time, cloud calls drop to near-zero, and the whole thing runs on edge hardware for the cost of 2MB per room.
