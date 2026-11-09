# VELOCITY AI — Model Research & Evaluation Strategy

**Target Hardware:** Acer Predator PT315-51 (Intel i5-9300H, NVIDIA GTX 1650 4GB VRAM, 16GB RAM)  
**Primary Engine:** llama.cpp (GGUF format)

---

## 1. Candidate Small Language Models (SLMs)

Our goal is maximum real-world inference speed with solid conversational, reasoning, and coding competence on limited hardware.

| Model | Parameters | Context Window | Architecture / License | GGUF Q4_K_M Size | VRAM Footprint (Weights + 2k KV) | Primary Strength |
|---|---|---|---|---|---|---|
| **Qwen2.5-0.5B-Instruct** | 0.49B | 32,768 | Qwen2 / Apache 2.0 | ~398 MB | ~0.65 GB | Instantaneous inference (>90 tok/s), ideal draft model |
| **Qwen2.5-1.5B-Instruct** | 1.54B | 32,768 | Qwen2 / Apache 2.0 | ~0.99 GB | ~1.30 GB | Best-in-class reasoning & coding for $\le$2B size |
| **Llama-3.2-1B-Instruct** | 1.23B | 131,072 | Llama 3 / Community | ~0.80 GB | ~1.10 GB | High throughput, great instruction following |
| **SmolLM2-1.7B-Instruct** | 1.71B | 8,192 | Llama / Apache 2.0 | ~1.06 GB | ~1.35 GB | Balanced generalist, permissive open license |
| **Llama-3.2-3B-Instruct** | 3.21B | 131,072 | Llama 3 / Community | ~2.02 GB | ~2.65 GB | Richer knowledge, fits in 4GB VRAM at 2k context |
| **Gemma-2-2B-Instruct** | 2.61B | 8,192 | Gemma / Open terms | ~1.62 GB | ~2.20 GB | Strong reasoning, sliding window attention |

---

## 2. Quantization Precision Comparison

GGUF quantizations balance file size, memory footprint, and perplexity (accuracy retention):

| Quantization | Effective Bits/Weight | File Size vs FP16 | Perplexity Impact | Recommended Use Case in VELOCITY AI |
|---|---|---|---|---|
| **Q4_0** | 4.0 bits | -72% | Low to Moderate | Baseline CPU testing; uniform 32-block layout. |
| **Q4_K_M** | **4.5 bits** | **-70%** | **Extremely Low** | **Recommended Default.** Critical layers (attention) retain 5-bit/6-bit precision while feed-forward uses 4-bit. |
| **Q5_K_M** | 5.5 bits | -63% | Negligible | Balanced mode when VRAM permits higher precision. |
| **Q8_0** | 8.5 bits | -46% | Effectively Zero | Quality mode or speculative draft verification. |

### Memory Budget on GTX 1650 (4,096 MiB Total VRAM)
```
Total Physical VRAM:         4,096 MiB
OS / Desktop Compositor:     - 250 MiB
Safety Headroom:             - 150 MiB
--------------------------------------
Maximum Usable Budget:       3,696 MiB

Configuration Examples:
1. Qwen2.5-1.5B Q4_K_M:      1,014 MiB weights + 40 MiB KV = 1,054 MiB (28% of budget) -> 100% GPU offload, zero PCIe bus stalls.
2. Llama-3.2-1B Q4_K_M:        820 MiB weights + 35 MiB KV =   855 MiB (23% of budget) -> 100% GPU offload.
3. Llama-3.2-3B Q4_K_M:      2,070 MiB weights + 80 MiB KV = 2,150 MiB (58% of budget) -> Fits comfortably on GPU.
```

---

## 3. KV Cache Optimization & Context Strategy

Traditional full 16-bit FP16 KV caches can waste memory when scaling context lengths:
- **Grouped Query Attention (GQA):** Modern models like Llama 3.2 and Qwen2.5 use GQA with a small number of KV heads (e.g. 2 or 8 KV heads vs 14 or 32 query heads), drastically shrinking KV cache size compared to multi-head attention models.
- **Quantized KV Cache:** Llama.cpp supports `--cache-type-k q8_0` and `--cache-type-v q8_0`, cutting KV cache memory consumption in half with zero perceptible degradation in chat quality.
- **Context Ceiling Strategy:**
  - Default context: **2,048 tokens** (instantaneous TTFT and optimal memory density).
  - Configurable range: 1,024 to 8,192 tokens.
  - Flash Attention (`--flash-attn` / `-fa`): Enabled on CUDA backend to eliminate $O(N^2)$ memory allocation during long prompts.

---

## 4. Speculative Decoding Pairings

Speculative decoding uses a small, fast "draft" model to propose $K$ tokens in parallel, which a larger "target" model verifies in a single forward pass.

### Viable Pairing for VELOCITY AI:
- **Target Model:** `Qwen2.5-1.5B-Instruct-Q4_K_M.gguf` (~0.99 GB)
- **Draft Model:** `Qwen2.5-0.5B-Instruct-Q4_K_M.gguf` (~0.39 GB)

#### Why This Pairing Works:
1. **Shared Tokenizer & Vocabulary:** Both models use the identical 151,936-token vocabulary and byte-pair encoding. No tokenizer translation layer is needed.
2. **Fits in 4 GB VRAM:** Combined VRAM footprint is only **~1.45 GB**, well within the 4 GB GTX 1650 budget.
3. **Draft Speed:** Qwen2.5-0.5B runs at $\approx 100$ tok/s on the GTX 1650.
4. **Validation Protocol:** Must be benchmarked against standard non-speculative inference. If acceptance rate $< 60\%$ or latency increases, speculative decoding will automatically disable itself.

---

## 5. Verification Checklist Before Downloading Any Model

- [x] Architecture supported by current llama.cpp build.
- [x] GGUF format version (V3 compatible).
- [x] Permissive or clear non-commercial/commercial licensing.
- [x] File size verified against free disk space on Drive D.
- [x] SHA256 integrity checksum available.
- [x] Explicit user confirmation required before initiate download.

