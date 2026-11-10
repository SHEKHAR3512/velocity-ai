# ⚡ VELOCITY AI

> **A native Rust offline AI desktop chatbot engineered specifically for extreme inference speed, minimal memory footprint, and zero bloat on limited hardware.**

VELOCITY AI was conceived and engineered with a single guiding question:
**"Does this engineering decision make VELOCITY AI faster, lighter, or more efficient?"**

Instead of building another heavy Electron wrapper or generic Python prototype, VELOCITY AI is written from the ground up in **pure Rust** and optimized down to the hardware registers of consumer laptops (specifically tuned for an **Acer Predator PT315-51**: Intel Core i5-9300H, NVIDIA GTX 1650 4GB VRAM, 16GB RAM, Windows 11).

The result: An offline AI desktop application that consumes **only ~78 MB of RAM**, achieves **138 ms Time-To-First-Token (TTFT)**, and streams tokens at **over 160 tokens per second** locally with zero cloud dependencies.

---

## 📑 Table of Contents
1. [The Problem with Modern AI Chatbots](#-the-problem-with-modern-ai-chatbots)
2. [How We Made It So Fast (The 7 Core Optimizations)](#-how-we-made-it-so-fast-the-7-core-optimizations)
3. [Empirical Benchmark Results](#-empirical-benchmark-results)
4. [Technology Stack](#-technology-stack)
5. [System Architecture](#-system-architecture)
6. [Curated Model Strategy](#-curated-model-strategy)
7. [Modular Workspace Structure](#-modular-workspace-structure)
8. [Building & Running](#-building--running)
9. [Verification & Milestone Log](#-verification--milestone-log)

---

## 🚫 The Problem with Modern AI Chatbots

Most desktop AI chatbots today suffer from three critical engineering flaws:

1. **Electron & Web Bloat:** They bundle a full Chromium browser engine and Node.js runtime. An empty chat window eats **800 MB to 1.5 GB of RAM** before a model is even loaded, introducing garbage collector (GC) pauses and sluggish window resizing.
2. **Blind Hardware Assumptions:** Typical runtimes allocate all available CPU threads (e.g. 8 threads on a 4-core/8-thread laptop CPU), causing massive L3 cache contention and severe thermal throttling.
3. **PCIe Synchronization Stalls:** When a model exceeds VRAM, generic software offloads half the layers to CPU and half to GPU, causing severe PCIe 3.0 bus bottlenecks that can degrade token generation speed by over **64%**.

**VELOCITY AI fixes all of this.**

---

## 🚀 How We Made It So Fast (The 7 Core Optimizations)

Here are the concrete systems-engineering decisions that make VELOCITY AI deliver instantaneous responses on limited hardware:

### 1. Eliminating Hyperthreading Cache-Thrashing (The 4-Thread Rule)
* **The Problem:** The Intel Core i5-9300H has 4 physical cores and 8 logical threads sharing an 8 MB L3 cache. Dense matrix multiplications (GEMM) in LLM inference are memory-bandwidth and cache bound.
* **Our Discovery:** In multi-trial empirical benchmarks, running 8 threads degraded token generation throughput to **22.8 tokens/sec** because hyperthreading threads constantly thrash each other's L1/L2 data caches and compete for memory controller channels.
* **The Solution:** VELOCITY AI dynamically detects physical CPU cores and locks execution to **exactly 4 threads**. This single hardware-level optimization boosted generation speed to **31.4 tokens/sec (+37.6% faster!)**.

### 2. Dual GPU Acceleration: Vulkan for Instant TTFT, CUDA for Peak Throughput
Rather than locking into a single backend, VELOCITY AI provides dynamic backend routing based on real benchmark telemetry:
* **Vulkan Compute (`-dev Vulkan1` on GTX 1650):** Achieves an astonishing **3,694.5 tokens/sec prompt evaluation rate**. This cuts Time-To-First-Token (TTFT) for a 512-token context to just **138.6 milliseconds**—2.47× faster than CUDA. The assistant begins responding almost instantaneously after hitting Enter.
* **NVIDIA CUDA 12.4:** Once token streaming begins, CUDA reaches a sustained continuous generation throughput of **162.05 tokens/sec** on the GTX 1650 Turing GPU.
* Users can toggle between Vulkan (instant interactive responsiveness) and CUDA (maximum streaming throughput) with one click in the UI.

### 3. Zero Electron / Zero Web Bloat (~78 MB UI Footprint)
* VELOCITY AI uses **native Rust** with the **`egui` / `eframe`** immediate-mode graphical toolkit running on raw OpenGL (`glow`).
* There is no JavaScript, no HTML DOM, no V8 engine, and no CSS recalculation overhead.
* The entire desktop application idles at only **~78 MB of RAM** and renders at a locked **60 FPS** with zero frame drops during active token generation.

### 4. 100% VRAM Layer Offloading (Eliminating PCIe Bus Stalls)
* **The Benchmark Proof:** In our live hardware tests, offloading 50% of model layers to GPU and 50% to CPU caused token generation to plummet from **162.05 tok/s to 57.60 tok/s (a 64.5% performance penalty)** due to PCIe 3.0 synchronization stalls on laptop architecture.
* **The Rule:** For small language models (< 3.2 GB), VELOCITY AI enforces **100% layer offloading (`-ngl 99`)** directly into the 4 GB GDDR5 VRAM. Every matrix calculation remains on the GPU's high-speed internal memory bus without roundtrips to system RAM.

### 5. Automated Pre-Emptive Model Warm-up
* Cold LLM processes normally suffer from a 1.5 to 3 second delay on the first query while Vulkan shaders compile, memory maps prime, and driver caches fill.
* When VELOCITY AI initializes an inference backend, it automatically fires an asynchronous 1-token warm-up in the background (completed in ~397 ms). By the time the user types their first prompt, the GPU execution pipeline is fully primed, ensuring **sub-250 ms end-to-end response latency**.

### 6. Zero-Copy GGUF Binary Parsing & Memory-Mapped Loading (`mmap`)
* Instead of reading gigabytes of weights into user-space heap buffers, models are mapped directly into virtual memory via the operating system's `mmap` kernel facility.
* Our custom zero-dependency GGUF inspector (`crates/velocity-models`) parses binary header magic bytes (`GGUF`), versions, tensor counts, and metadata in **under 4 milliseconds**, without loading the tensor weight tables.

### 7. Asynchronous Non-Blocking Channel Architecture
* The graphical UI thread never makes HTTP requests, disk I/O, or child process operations.
* A dedicated multi-threaded **Tokio runtime** supervises the inference server and receives Server-Sent Events (SSE) token streams.
* Tokens flow to the UI across lock-free Rust `mpsc` channels, allowing the UI to render markdown and code syntax highlighting fluidly without ever stuttering.

---

## 📊 Empirical Benchmark Results

All benchmarks were measured directly on the target hardware (**Acer Predator PT315-51**, Intel Core i5-9300H, NVIDIA GeForce GTX 1650 4GB VRAM, 16GB RAM, Windows 11) using `Qwen2.5-0.5B-Instruct-Q4_K_M.gguf` across 3 warm trials:

| Hardware Configuration | Prompt Processing (tok/s) | Generation Speed (tok/s) | TTFT (512 ctx) | VRAM / RAM Used | Status / Verdict |
|---|---|---|---|---|---|
| **Vulkan Compute (GTX 1650)** | **3,694.53 ± 147.35** | **131.70 ± 5.44** | **138.6 ms** | ~491 MB VRAM | 🏆 **Fastest TTFT (Optimal for Chat)** |
| **CUDA 12.4 (GTX 1650)** | 1,495.93 ± 14.62 | **162.05 ± 10.59** | 342.3 ms | ~491 MB VRAM | 🚀 **Peak Streaming Throughput** |
| **CPU AVX2 (4 Threads)** | 268.42 ± 1.52 | **31.37 ± 0.31** | 1,907.6 ms | 0 MB VRAM / ~600 MB RAM | 🎯 **Optimal CPU Baseline** |
| **CPU AVX2 (2 Threads)** | 143.90 ± 0.69 | 18.93 ± 0.12 | 3,558.0 ms | 0 MB VRAM / ~600 MB RAM | ⚠️ CPU Underutilized (-39.7%) |
| **CPU AVX2 (6 Threads)** | 338.16 ± 3.42 | 28.99 ± 0.49 | 1,514.0 ms | 0 MB VRAM / ~600 MB RAM | ⚠️ HT Contention (-7.6%) |
| **CPU AVX2 (8 Threads)** | 364.70 ± 2.61 | 22.80 ± 0.33 | 1,403.9 ms | 0 MB VRAM / ~600 MB RAM | ❌ Hyperthreading Thrashing (-27.3%) |
| **CUDA 50% Layers Offloaded** | 761.27 ± 4.21 | 57.60 ± 0.38 | 672.5 ms | ~250 MB VRAM / ~350 MB RAM | ❌ PCIe Bus Penalty (**-64.5% Loss**) |
| **Intel UHD Graphics 630** | 129.56 ± 0.94 | 17.63 ± 0.19 | 3,951.8 ms | Shared System RAM | ❌ Slower than pure CPU |

---

## 🛠️ Technology Stack

```
┌────────────────────────────────────────────────────────┐
│                   VELOCITY AI UI                       │
│    Native Rust • egui / eframe • Minimal Dark Theme   │
└──────────────────────────┬─────────────────────────────┘
                           │ (Lock-free MPSC Channels)
┌──────────────────────────▼─────────────────────────────┐
│                 Inference Supervisor                   │
│     Async Tokio Runtime • HTTP SSE Client • Metrics    │
└──────────────────────────┬─────────────────────────────┘
                           │ (Localhost REST / SSE)
┌──────────────────────────▼─────────────────────────────┐
│                 llama.cpp Runtime                      │
│       Vulkan Compute  •  CUDA 12.4  •  CPU AVX2        │
└──────────────────────────┬─────────────────────────────┘
                           │ (Zero-Copy mmap)
┌──────────────────────────▼─────────────────────────────┐
│             Local GGUF Models & SQLite WAL             │
│   Qwen 2.5 • Llama 3.2 • SmolLM2 • Local Storage DB    │
└────────────────────────────────────────────────────────┘
```

| Layer | Component | Description |
|---|---|---|
| **Core Systems** | [Rust](https://www.rust-lang.org/) | Compiled native language providing memory safety, zero-cost abstractions, and predictable latency. |
| **UI Framework** | [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) | Immediate-mode desktop GUI running via OpenGL (`glow`). 60 FPS, ~78 MB RAM, zero web dependencies. |
| **Async Runtime** | [`tokio`](https://tokio.rs/) | Multi-threaded asynchronous event loop powering non-blocking process management and network streaming. |
| **Inference Engine** | [`llama.cpp`](https://github.com/ggml-org/llama.cpp) | High-performance C/C++ tensor library supporting Vulkan 1.3, CUDA 12.4, and AVX2 CPU SIMD instructions. |
| **Model Format** | [GGUF](https://github.com/ggerganov/ggml/blob/master/docs/gguf.md) | Single-file binary model container format with memory-mapped tensor loading. |
| **Storage Engine** | [SQLite](https://www.sqlite.org/) via [`rusqlite`](https://github.com/rusqlite/rusqlite) | Embedded relational database operating with Write-Ahead Logging (WAL) and `synchronous = NORMAL`. |
| **Integrity & Hash** | [`ring`](https://github.com/briansmith/ring) | SIMD-accelerated cryptographic SHA-256 validation for downloaded models. |
| **Model Networking** | [`reqwest`](https://github.com/seanmonstar/reqwest) | Non-blocking HTTP client supporting HTTP `Range` headers for resumable model downloads. |

---

## 🧠 Curated Model Strategy

To deliver maximum speed on consumer hardware with 4 GB VRAM, VELOCITY AI focuses on modern **Small Language Models (SLMs)**:

1. **Qwen 2.5 0.5B Instruct (Q4_K_M — 491 MB):**
   * *Speed:* 130 – 162 tok/s | *TTFT:* ~138 ms
   * *Role:* Instantaneous answers, quick lookups, summarization, and near-zero VRAM pressure.
2. **Qwen 2.5 1.5B Instruct (Q4_K_M — 986 MB):**
   * *Speed:* 70 – 95 tok/s | *TTFT:* ~220 ms
   * *Role:* The optimal sweet spot between reasoning capability, coding assistance, and high speed.
3. **Llama 3.2 1B Instruct (Q4_K_M — 809 MB):**
   * *Speed:* 80 – 110 tok/s | *TTFT:* ~190 ms
   * *Role:* Meta's compact instruction-tuned model for dialogue and text generation.
4. **SmolLM2 1.7B Instruct (Q4_K_M — 1,060 MB):**
   * *Speed:* 65 – 90 tok/s | *TTFT:* ~240 ms
   * *Role:* High-accuracy lightweight reasoning model trained on synthetic datasets.

---

## 📁 Modular Workspace Structure

```
velocity-ai/
│
├── apps/
│   └── desktop/               # Native egui/eframe desktop application
│       ├── src/
│       │   ├── app.rs         # eframe::App event loop & UI orchestrator
│       │   ├── state.rs       # Shared application state & MPSC command enums
│       │   ├── theme.rs       # Dark macOS/Linear design palette & styling
│       │   ├── views/
│       │   │   ├── chat.rs        # Chat messages, code blocks, copy buttons, input dock
│       │   │   ├── models.rs      # Curated catalog, download progress, local GGUF activation
│       │   │   ├── benchmarks.rs  # Live hardware telemetry & empirical benchmark matrix
│       │   │   └── settings.rs    # Sliders for context size, threads, sampling parameters
│       │   └── main.rs        # Tokio worker runtime & native window launcher
│
├── crates/
│   ├── velocity-core/         # Domain types (Message, Conversation, Role, Metrics, Error)
│   ├── velocity-hardware/     # Runtime AVX2, RAM, and GPU detection with recommendation logic
│   ├── velocity-config/       # AppConfig TOML persistence and path resolution
│   ├── velocity-storage/      # SQLite conversation & message persistence with WAL mode
│   ├── velocity-models/       # Pure Rust GGUF binary parser, catalog, and resumable downloader
│   ├── velocity-engine/       # llama-server supervisor, health checker, and SSE streaming client
│   └── velocity-benchmark/    # Empirical multi-trial benchmark execution harness
│
├── bin/                       # Portable inference runtimes
│   ├── llama-cpu/             # CPU AVX2 prebuilt runtime
│   ├── llama-vulkan/          # Vulkan 1.3 prebuilt runtime
│   └── llama-cuda/            # CUDA 12.4 prebuilt runtime
│
├── models/                    # Downloaded and imported local GGUF models
├── benchmarks/                # Automated PowerShell benchmark harnesses & recorded JSON results
├── docs/                      # Architectural specifications & hardware audit reports
├── Cargo.toml                 # Cargo workspace configuration
└── PHASE_TRACKER.md           # Engineering verification gates and phase status
```

---

## 💻 Building & Running

### Prerequisites
* **Operating System:** Windows 11 (primary) or Linux (secondary).
* **Rust Toolchain:** Rust 1.80+ (`stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`).
* **C/C++ Compiler:** MinGW-w64 (GCC/Clang) for compiling embedded C dependencies (`libsqlite3-sys`, `ring`).

### 1. Launch the Desktop GUI Application
```powershell
cargo run -p velocity-desktop
```
This opens the native dark-themed desktop application, connects to the local inference supervisor, warms up the model, and displays the ready chat window.

### 2. Launch the High-Performance Terminal Chatbot
For direct command-line usage without any GUI overhead:
```powershell
cargo run -p velocity-engine --bin velocity-chat
```

### 3. Run All Workspace Unit Tests
```powershell
cargo test --workspace
```
Runs the automated test suite across all crates, including GGUF binary inspection, SHA-256 integrity checks, SQLite CRUD operations, and hardware detection.

---

## 📈 Verification & Milestone Status

| Phase | Milestone | Status | Key Deliverable |
|---|---|---|---|
| **Phase 0** | Research, Planning & Hardware Audit | **Completed** | Full hardware inspection & architecture documentation. |
| **Phase 1** | Empirical Inference Benchmarking | **Completed** | 3-trial benchmark across 8 hardware configurations recorded. |
| **Phase 2** | Rust Core Workspace & Terminal Chatbot | **Completed** | `velocity-chat` interactive client (67.7 tok/s, 185 ms TTFT). |
| **Phase 3** | Local Model Manager & Resumable Downloader | **Completed** | Zero-dependency GGUF parser + HTTP Range downloader. |
| **Phase 4** | Native `egui`/`eframe` Desktop Application | **Completed** | 60 FPS dark-themed GUI, SQLite WAL persistence, multi-tab layout. |
| **Phase 5** | Advanced Hardware Tuning & Speculative Decoding | **In Progress** | Automated hardware selector & speculative decoding evaluation. |
| **Phase 6** | Packaging & 100% Offline Release | **Pending** | Standalone production Windows installer & air-gapped test. |

---

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.
