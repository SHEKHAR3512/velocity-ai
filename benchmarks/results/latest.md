# VELOCITY AI — Hardware Inference Benchmark Report

**Machine:** Acer Predator PT315-51  
**CPU:** Intel Core i5-9300H (4 Cores / 8 Threads, AVX2)  
**GPU 1 (dGPU):** NVIDIA GeForce GTX 1650 (4 GB VRAM, Turing TU117, Compute Capability 7.5)  
**GPU 2 (iGPU):** Intel UHD Graphics 630  
**RAM:** 16 GB DDR4  
**Model Evaluated:** `qwen2.5-0.5b-instruct-q4_k_m.gguf` (Q4_K_M, ~491 MB)  
**Prompt / Gen Window:** 512 tokens prompt eval / 128 tokens generation  
**Timestamp:** 2026-11-09 20:47:40  
**Methodology:** 3 repeated trials per configuration, median with standard deviation recorded.

---

## 1. Empirical Performance Matrix

| Configuration | Backend | Execution Device | Prompt Eval (pp512 tok/s) | Generation (tg128 tok/s) | Approx. TTFT (512 tokens) |
|---|---|---|---|---|---|
| **CPU-AVX2 (2 threads)** | CPU | Intel Core i5-9300H | 56.88 ± 1.90 | 18.93 ± 2.43 | ~9,001 ms |
| **CPU-AVX2 (4 threads)** | **CPU** | **Intel Core i5-9300H** | **109.48 ± 3.37** | **31.37 ± 3.15** | **~4,676 ms** |
| **CPU-AVX2 (6 threads)** | CPU | Intel Core i5-9300H | 124.50 ± 4.21 | 28.99 ± 8.76 | ~4,112 ms |
| **CPU-AVX2 (8 threads)** | CPU | Intel Core i5-9300H | 111.26 ± 6.77 | 22.80 ± 1.08 | ~4,601 ms |
| **Vulkan (GTX 1650, 100% offload)** | **Vulkan** | **NVIDIA GeForce GTX 1650** | **3,694.53 ± 147.35** | **131.70 ± 5.44** | **~138.6 ms** |
| **Vulkan (Intel UHD 630, 100% offload)** | Vulkan | Intel UHD Graphics 630 | 97.42 ± 2.90 | 17.63 ± 1.02 | ~5,255 ms |
| **CUDA 12.4 (GTX 1650, 100% offload)** | **CUDA** | **NVIDIA GeForce GTX 1650** | **1,495.93 ± 14.62** | **162.05 ± 10.59** | **~342.3 ms** |
| **CUDA 12.4 (GTX 1650, 50% / 12 layers)**| CUDA | GTX 1650 + i5-9300H | 1,288.19 ± 57.21 | 57.60 ± 3.08 | ~397.5 ms |

---

## 2. Key Empirical Findings

1. **Vulkan Delivers 2.47x Faster TTFT on the GTX 1650:**
   - Vulkan compute achieved **3,694.5 tokens/sec** prompt prefill versus CUDA's **1,495.9 tokens/sec**. For a 512-token prompt, Vulkan produces the first token in just **138 ms** (sub-second interactive conversational feel).
2. **CUDA 12.4 Delivers the Highest Steady-State Generation Throughput:**
   - CUDA achieved **162.05 tokens/sec** during active generation versus Vulkan's **131.70 tokens/sec** (a 23% advantage in raw stream speed).
3. **CPU Physical Core Limit (4 Threads Sweet Spot):**
   - On the Intel Core i5-9300H (4 physical cores, 8 logical threads), **4 threads is the optimal CPU configuration** (31.37 tok/s). Scaling to 8 threads causes hyperthreading cache thrashing on the 8MB L3 cache, causing generation speed to drop by **27.3% down to 22.80 tok/s**.
4. **The Partial Offloading Penalty (64.5% Speed Loss):**
   - Offloading only 50% of layers to the GPU causes generation speed to plummet from **162.05 tok/s down to 57.60 tok/s** due to PCIe 3.0 bus synchronization stalls between CPU and GPU.
   - **Architectural Directive:** Always target 100% GPU offload for models $\le$ 3.5 GB on this hardware.
5. **Integrated Intel UHD 630 Bottleneck:**
   - Generating at only 17.63 tok/s, the integrated GPU is slower than pure CPU AVX2 (31.37 tok/s). Discrete GPU selection is mandatory.

---

## 3. Recommended Default Runtime Configuration

| Operating Mode | Backend | Thread Count | GPU Layer Offload | Expected Throughput | Best For |
|---|---|---|---|---|---|
| **AUTO / ULTRA-RESPONSIVE (Default)** | **Vulkan (`-dev Vulkan1`)** | 4 | 100% (`-ngl 99`) | **~132 tok/s (138 ms TTFT)** | Interactive conversational speed, instant latency |
| **MAXIMUM THROUGHPUT** | **CUDA 12.4 (`CUDA_VISIBLE_DEVICES=0`)** | 4 | 100% (`-ngl 99`) | **~162 tok/s (342 ms TTFT)** | Long text generation, summarization, coding tasks |
| **CPU FALLBACK / STABILITY** | **CPU AVX2** | **4** | 0% (`-ngl 0`) | **~31 tok/s** | Zero VRAM requirement, memory pressure fallback |
