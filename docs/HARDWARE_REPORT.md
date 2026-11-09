# VELOCITY AI — Hardware Audit & Capability Report

**Date:** 2026-11-09  
**Machine:** Acer Predator PT315-51  
**Operating System:** Windows 11 Home Single Language (Build 10.0.26300, 64-bit)  
**Verification Method:** Direct hardware query via PowerShell CIM/WMI, `nvidia-smi`, and system driver interrogation.

---

## 1. System Hardware Specifications

| Component | Detected Specification | Verification Status |
|---|---|---|
| **CPU** | Intel Core i5-9300H @ 2.40GHz (Coffee Lake-H, 14nm) | **Verified** |
| **Physical Cores** | 4 physical cores | **Verified** |
| **Logical Threads** | 8 logical threads (Hyper-Threading enabled) | **Verified** |
| **L3 Cache** | 8,192 KB (8 MB SmartCache shared) | **Verified** |
| **Instruction Sets** | AVX, AVX2, FMA3, SSE4.1, SSE4.2, BMI1, BMI2 | **Verified** |
| **System RAM** | 16,610,356 KB visible (~16 GB DDR4) | **Verified** |
| **Available RAM** | ~6.57 GB currently free (dependent on running apps) | **Verified** |
| **Primary GPU (Discrete)** | NVIDIA GeForce GTX 1650 Mobile (Turing TU117) | **Verified** |
| **VRAM Capacity** | 4,096 MiB (4 GB) dedicated video memory | **Verified** |
| **Compute Capability**| 7.5 (Turing architecture) | **Verified** |
| **NVIDIA Driver** | 617.42 (KMD 617.42, Driver Version 32.0.16.1742) | **Verified** |
| **CUDA Driver Support**| CUDA UMD 13.4 supported via `nvcuda.dll` | **Verified** |
| **Vulkan Support** | Present (`C:\Windows\System32\vulkan-1.dll`) | **Verified** |
| **Secondary GPU** | Intel UHD Graphics 630 (Driver 31.0.101.2145) | **Verified** |
| **Primary Storage (C:)**| 254.9 GB SSD (~59.27 GB free) | **Verified** |
| **Secondary Storage (D:)**| 1,000.2 GB HDD (~800.97 GB free) | **Verified** |

---

## 2. Hardware Acceleration Capability Matrix

| Acceleration Backend | Availability | Compatibility Notes |
|---|---|---|
| **CPU AVX2** | **Available & Usable** | Supported directly by the i5-9300H instruction pipeline. Delivers predictable, reliable inference with zero VRAM limits. |
| **Vulkan Compute** | **Available & Usable** | Supported through `vulkan-1.dll`. Bypasses CUDA driver stack dependencies, allows cross-vendor execution, and provides low-overhead GPU compute. |
| **CUDA (12.4 Runtime)** | **Available & Usable** | Supported via NVIDIA driver 617.42 and Compute Capability 7.5. Compatible with prebuilt `llama-server` CUDA 12.4 runtime. |
| **DirectML** | **Available (via Windows)** | Can be utilized for fallback or cross-GPU execution if needed. |

---

## 3. Hardware Constraints & Critical Engineering Trade-Offs

### A. The 4 GB VRAM Hard Boundary
The NVIDIA GeForce GTX 1650 has 4,096 MiB of dedicated VRAM.
- Operating system display compositing and background Windows desktop processes consume ~160–250 MiB of VRAM.
- Realistically usable VRAM for AI models is **~3,700–3,850 MiB**.
- Any model whose weights + KV cache exceed ~3.7 GB will either fail allocation or trigger layer offloading spillover to system RAM.

### B. PCIe Bus Bandwidth & Partial Offloading Penalty
The Acer Predator PT315-51 connects the mobile GTX 1650 via PCIe 3.0.
- When a model is **partially offloaded** (e.g. 15 layers on GPU, 10 layers on CPU), hidden layer tensor activations must travel back and forth across the PCIe 3.0 bus for every generated token.
- At 16 lanes of PCIe 3.0 (~15.75 GB/s theoretical, ~12 GB/s practical), this transfer overhead can negate GPU compute benefits, causing generation speed to drop below pure CPU inference.
- **Rule of Thumb for VELOCITY AI:** 
  - Prefer **100% full GPU offload** for models $\le$ 2.5 GB file size.
  - Benchmark pure CPU vs partial offload before deciding any split configuration.

### C. CPU Thread Scaling & Hyper-Threading Contention
The Intel Core i5-9300H features 4 physical cores and 8 threads.
- Hyper-Threading pairs two logical execution threads on a single physical execution pipeline with shared L1/L2 caches and an 8 MB L3 cache.
- In dense GEMM (matrix multiplication) loops typical of LLM inference, logical sibling threads contend for execution units and cache lines.
- Empirical testing on similar mobile architectures consistently shows that **4 threads (matching physical core count)** or **6 threads** yields higher tokens-per-second and lower jitter than 8 threads.
- Phase 1 must benchmark 2, 4, 6, and 8 threads to confirm the sweet spot empirically.

### D. Memory Bandwidth Comparison
- **DDR4 System RAM:** Dual-channel bandwidth is approximately **~38–42 GB/s**.
- **GTX 1650 GDDR5/GDDR6 VRAM:** Dedicated bandwidth is approximately **~128 GB/s** (GDDR5) / **~192 GB/s** (GDDR6).
- For models small enough to live entirely within VRAM (such as 1B to 1.5B parameter models quantized at Q4_K_M or Q5_K_M, taking 0.8–1.2 GB), the GPU will execute memory-bandwidth-bound token generation up to **3x to 4x faster** than CPU RAM.

---

## 4. Hardware-Informed Architecture Directives

1. **Prioritize 1B–2B Parameter Models:** Models in this bracket consume 0.8–1.8 GB VRAM, fitting 100% inside the 4 GB GTX 1650 with generous headroom for a 2048–4096 context KV cache.
2. **Never Blindly Enable CUDA:** Test whether Vulkan or CPU AVX2 is faster for specific model architectures and prompt lengths.
3. **Keep Storage on Drive D:** The project workspace and model repository are stationed on Drive D (800 GB free HDD), while frequently executed binaries can be cached or loaded into memory buffers.
4. **Active Memory Guard:** The application must inspect available system RAM and VRAM before spawning the inference engine to prevent out-of-memory OS thrashing.

---

## 5. OS Security & Code Integrity Diagnostic (Smart App Control)

During Phase 1 execution of external inference runtimes, the Windows kernel raised:
- **NTSTATUS:** `0xC0E90002` (`STATUS_SYSTEM_INTEGRITY_POLICY_VIOLATION`)
- **Win32 Error:** `os error 4551` (*An Application Control policy has blocked this file*)
- **Code Integrity Event ID 3077:** `llama-cli.exe attempted to load llama.dll that did not meet the Enterprise signing level requirements or violated code integrity policy {0283ac0f-fff1-49ae-ada1-8a933130cad6}`
- **Registry Setting:** `HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy\VerifiedAndReputablePolicyState = 1` (Smart App Control: Enforced).

**Resolution:**
To develop and run native, high-performance offline AI runtimes (`llama.cpp`, Cargo build scripts, and local executables), Windows 11 Smart App Control must be set to **Off** in **Windows Security > App & browser control > Smart App Control settings**.


