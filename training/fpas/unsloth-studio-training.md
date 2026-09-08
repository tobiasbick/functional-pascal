# Unsloth Studio training settings

This guide describes the recommended starting configuration for fine-tuning
`unsloth/Qwen3.5-2B` on the Functional Pascal conversational dataset in this
directory. It is an experiment baseline, not a claim that one parameter set is
optimal for every machine or future dataset revision.

Use `data/train.jsonl` for training and `data/validation.jsonl` for evaluation.
Keep `data/test.jsonl` separate for final compiler-backed evaluation.

## Recommended baseline

| Setting | Value |
| --- | --- |
| Model | `unsloth/Qwen3.5-2B` |
| Training method | 16-bit LoRA |
| Context length | `4096` |
| Epochs | `3` |
| Maximum steps | disabled / `0` |
| Per-device batch size | `2` |
| Gradient accumulation steps | `4` |
| Effective batch size | `8` |
| Learning rate | `2e-4` |
| Warmup ratio | `0.1` |
| Weight decay | `0.001` |
| Optimizer | `adamw_8bit` |
| Learning-rate scheduler | `linear` |
| LoRA rank | `16` |
| LoRA alpha | `16` |
| LoRA dropout | `0` |
| Gradient checkpointing | `unsloth` |
| Sequence packing | disabled initially |
| Train on completions only | enabled |
| Evaluation interval | `0.1` of total steps |
| Random seed | `3407` |

Use epochs or maximum steps, not both. For this dataset, select epoch-based
training and leave maximum steps disabled.

## Context length

Do not use the model's maximum advertised context length as the training
context. Training memory and computation increase with the actual sequence
length, while unused capacity does not improve the result.

The current 850 records were measured with the Qwen3.5 tokenizer and chat
template:

| Distribution | Tokens |
| --- | ---: |
| Median | 262 |
| 90th percentile | 1,188 |
| 95th percentile | 1,792 |
| 99th percentile | 2,779 |
| Maximum | 3,178 |

At a context length of 2,048, 31 records would be truncated. No record exceeds
4,096 tokens, so 4,096 preserves the complete dataset without paying for an
irrelevant 262,144-token training context. If memory remains a problem, 2,048
is a reasonable fallback, with the truncation accepted explicitly.

The official Qwen3.5 guide also recommends starting at 2,048 and increasing the
context only after the basic configuration works:

- <https://unsloth.ai/docs/models/qwen3.5/fine-tune.md>

## LoRA and text-only training

Keep 16-bit LoRA for Qwen3.5. Unsloth warns that QLoRA has unusually large
quantization differences for this model family and does not recommend 4-bit
training.

Apply LoRA to both attention and MLP projections:

```text
q_proj, k_proj, v_proj, o_proj, gate_proj, up_proj, down_proj
```

Qwen3.5 is technically a unified vision-language model, but this dataset is
text-only. If Studio exposes the layer switches, use:

| Layer group | Setting |
| --- | --- |
| Fine-tune vision layers | disabled |
| Fine-tune language layers | enabled |
| Fine-tune attention modules | enabled |
| Fine-tune MLP modules | enabled |

No image examples or new vocabulary tokens are introduced, so the vision
encoder, token embeddings, and language-model head do not need separate
training for this run.

## Completion-only loss

Enable **Train on completions only**. Each record contains a system instruction,
a user request, and the desired assistant answer. The model should learn to
produce the answer, not spend most of the loss learning to reproduce prompts.

Current Unsloth Studio versions recognize Qwen3.5's ChatML boundaries as:

```text
<|im_start|>user
<|im_start|>assistant
```

At startup, inspect the log and confirm that completion masking was applied and
that rows were not dropped. Stop the run if masking removes all or a large part
of the dataset. The implementation contains a safety check, but the log remains
the easiest way to verify the effective configuration.

Unsloth's current default configuration and Qwen3.5 template mapping are
available here:

- <https://raw.githubusercontent.com/unslothai/unsloth/main/studio/backend/assets/configs/model_defaults/default.yaml>
- <https://raw.githubusercontent.com/unslothai/unsloth/main/studio/backend/utils/datasets/model_mappings.py>

## Evaluation and checkpoints

Keep the separate validation file attached and evaluate approximately every
10% of the run. Compare validation loss rather than training loss alone. A low
loss does not prove that generated FPAS compiles.

After training, evaluate at least these properties on held-out prompts:

1. The response identifies the language as Functional Pascal or FPAS.
2. Code uses `.fpas` conventions and the implemented `Std.*` units.
3. Reassigned locals use `mutable var`.
4. `for-in` variables include their types.
5. Source-only requests contain no Markdown or explanation.
6. The result passes `fpas check` and, when applicable, produces the expected
   output.

Use the same Qwen3.5 chat template during inference and after GGUF export. A
template mismatch can make an otherwise valid adapter appear substantially
worse:

- <https://unsloth.ai/docs/basics/saving-and-using-models/troubleshooting>

## Reasoning policy

The current dataset is intentionally direct-answer oriented. It contains
explanations when the user asks for an explanation, but it does not contain
hidden chain-of-thought traces. With the Qwen3.5 chat template, an assistant
message without separate reasoning content is rendered with an empty thinking
section followed by the answer.

Do not add automatically generated, free-form reasoning traces to the main
dataset yet. Incorrect rationales are especially dangerous for FPAS because a
general model can confidently reinterpret the language as Free Pascal, Delphi,
or generic Pascal. Training such a rationale would reinforce the exact failure
the dataset is intended to correct.

The recommended sequence is:

1. Train the direct-answer baseline with the settings above.
2. Measure compiler pass rate, expected output, format compliance, and FPAS
   terminology on the held-out test prompts.
3. Only if reasoning-enabled inference is an explicit product goal, create a
   separate experiment with curated, compiler-grounded reasoning summaries.
4. Compare that experiment against the same held-out test set before merging
   reasoning examples into the primary dataset.

A reasoning example should expose a short, verifiable explanation rather than
a long internal monologue. Suitable tasks include diagnosing one invalid FPAS
construct, citing the relevant FPAS rule, presenting corrected source, and
recording the compiler result. Each explanation must be reviewed against the
language documentation and its final program must pass the compiler.

Unsloth states that preserving Qwen3.5's reasoning behavior requires a dataset
dominated by reasoning-style examples, with at least 75% reasoning examples.
That would substantially change this dataset and should therefore be treated as
a separate measured experiment, not as a small toggle or a handful of added
records:

- <https://unsloth.ai/docs/models/qwen3.5/fine-tune.md>
