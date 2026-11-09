# VELOCITY AI — Implementation Blueprint & Project Plan

**Target System:** Acer Predator PT315-51 (Intel i5-9300H, GTX 1650 4GB VRAM, 16GB RAM)  
**Methodology:** Gravitas-enforced evidence-backed engineering, phased development, empirical benchmarking.

---

## Phase Breakdown

### PHASE 0 — Research, Environment & Hardware Audit
- **Objective:** Gather ground-truth facts on system capabilities, toolchain availability, and model options before writing application code.
- **Tasks:**
  - Verify Rust compiler, Cargo, Git, and Windows environment (`rustc 1.99.1`, `cargo 1.99.1`, GNU toolchain with `rust-lld`).
  - Interrogate hardware: CPU cores, AVX2, RAM, VRAM, NVIDIA driver 617.42, CUDA UMD 13.4, Vulkan loader.
  - Research official llama.cpp release binaries (CPU AVX2, Vulkan, CUDA 12.4).
  - Research candidate GGUF models for 4GB VRAM.
  - Author technical baseline documentation.
- **Deliverables:**
  - `PHASE_TRACKER.md`
  - `docs/HARDWARE_REPORT.md`
  - `docs/MODEL_RESEARCH.md`
  - `docs/ARCHITECTURE.md`
  - `docs/PROJECT_PLAN.md`
- **Verification Gate:**
  - All hardware specs confirmed via runtime commands.
  - Rust compilation verified with a test binary.
  - Gravitas policy record emitted.

---

### PHASE 1 — Inference Benchmarking Runtime
- **Objective:** Establish the empirical performance baseline (CPU vs Vulkan vs CUDA) on this exact Acer Predator machine using real small language models.
- **Tasks:**
  - Download official portable llama.cpp runtimes into `bin/` (`llama-cpu`, `llama-vulkan`, `llama-cuda`).
  - Acquire candidate small model (`Qwen2.5-1.5B-Instruct-Q4_K_M.gguf` or `Llama-3.2-1B-Instruct-Q4_K_M.gguf`).
  - Benchmark CPU thread counts: 2, 4, 6, 8 threads.
  - Benchmark acceleration backends: CPU AVX2 vs Vulkan vs CUDA 12.4.
  - Benchmark quantizations: Q4_K_M vs Q5_K_M vs Q8_0 (if applicable).
  - Measure: cold/warm load time, time to first token (TTFT), tokens/sec, RAM usage, VRAM usage.
- **Deliverables:**
  - Working local runtime binaries.
  - `benchmarks/results/latest.json`
  - `benchmarks/results/latest.md`
  - Recommended default configuration document based on observed numbers.
- **Verification Gate:**
  - Each configuration measured across at least 3 trials.
  - Zero fabricated numbers.
  - VERDICT: PASS on benchmark execution.

---

### PHASE 2 — Rust Core & Headless Inference Engine
- **Objective:** Implement the modular Cargo workspace and a fully functional terminal chatbot demonstrating streaming inference, process management, and local persistence.
- **Tasks:**
  - Initialize Cargo workspace (`velocity-core`, `velocity-hardware`, `velocity-config`, `velocity-storage`, `velocity-engine`, `velocity-models`, `velocity-benchmark`).
  - Implement hardware detection module in `velocity-hardware`.
  - Implement `llama-server` process supervisor with port allocation and `/health` polling.
  - Implement asynchronous SSE client streaming tokens over localhost HTTP.
  - Implement SQLite storage for chats and messages with WAL mode.
  - Build interactive terminal chat CLI binary.
- **Deliverables:**
  - Modular Cargo workspace passing all unit tests.
  - Functional terminal chat binary capable of streaming responses from local GGUF models.
- **Verification Gate:**
  - `cargo test --workspace` passes.
  - Terminal chat streams tokens with live latency and tok/s metrics.
  - Messages persist across restarts in SQLite.

---

### PHASE 3 — Model Management System
- **Objective:** Provide robust, user-friendly, completely offline-compatible model management.
- **Tasks:**
  - Build GGUF header inspector (reads architecture, parameter count, context length, quant type).
  - Implement configurable local model directory scanning (`%LOCALAPPDATA%/VelocityAI/models/` and custom directories).
  - Implement resumable HTTP downloader with chunked streaming, `Range` headers, and SHA256 verification.
  - Support manual local file import without internet access.
  - Add model unloading and switching logic in `velocity-engine`.
- **Deliverables:**
  - Model manager module in `velocity-models`.
  - Test suite covering header parsing, local import, and corrupted file handling.
- **Verification Gate:**
  - Valid GGUF files correctly parsed; corrupted or invalid files rejected with clear errors.
  - Downloads can be paused, interrupted, and resumed cleanly.

---

### PHASE 4 — Native Desktop Application (egui / eframe)
- **Objective:** Build a premium, minimalist, responsive native desktop application.
- **Tasks:**
  - Scaffold `apps/desktop` with `eframe` and `egui`.
  - Implement dark minimalist theme (inspired by macOS, Raycast, and Linear).
  - Design left navigation: New Chat, Chat History, Models, Benchmarks, Settings.
  - Design main chat interface: token streaming, smooth scrolling, markdown parser, syntax-highlighted code blocks, copy button.
  - Implement bottom status bar: active model, backend mode, CPU/GPU, tokens/sec, TTFT, RAM/VRAM.
  - Wire egui event loop with Tokio asynchronous supervisor via `mpsc` channels.
- **Deliverables:**
  - Compilable, launchable desktop executable.
  - Zero UI lag during active token generation.
- **Verification Gate:**
  - Desktop app renders at consistent 60 FPS.
  - Tokens stream in real time into chat bubbles.
  - Markdown and code blocks render correctly.

---

### PHASE 5 — Advanced Performance Optimization
- **Objective:** Optimize inference parameters specifically for the Acer Predator hardware.
- **Tasks:**
  - Integrate Flash Attention (`--flash-attn`).
  - Test quantized KV cache (`--cache-type-k q8_0`, `--cache-type-v q8_0`).
  - Test speculative decoding (`--model-draft`) comparing latency and speed.
  - Implement Auto Mode selector (chooses fastest verified backend and thread count).
  - Implement model warm-up on initial load.
- **Deliverables:**
  - Integrated optimization modes in UI: Auto Mode, Maximum Speed, Balanced, Quality.
  - Empirical report on speculative decoding gains or overhead on this machine.
- **Verification Gate:**
  - Optimization mode transitions verified without crashing.
  - Speculative decoding disabled automatically if slower than baseline.

---

### PHASE 6 — Hardening, Offline Verification & Release
- **Objective:** Finalize product reliability, verify 100% offline flight readiness, and deliver packaging.
- **Tasks:**
  - Comprehensive error recovery tests (missing model, killed runtime process, GPU out-of-memory).
  - Disconnect all network interfaces and verify complete functionality offline.
  - Generate standalone release bundle for Windows.
  - Document user installation, quick start, and architectural overview.
- **Deliverables:**
  - Final production build in `target/release`.
  - `README.md` and user manual.
  - Clean flight check verification log.
- **Verification Gate:**
  - Zero crashes across all recovery scenarios.
  - Complete operation verified in air-gapped / offline state.

---

## Risk Matrix & Mitigations

| Risk | Likelihood | Impact | Mitigation Strategy |
|---|---|---|---|
| **GTX 1650 4GB VRAM Exhaustion** | High | High | Strict pre-allocation memory guard; limit default context to 2,048 tokens; prioritize 1B–2B models; fallback to CPU AVX2 if allocation fails. |
| **PCIe 3.0 Transfer Bottleneck on Partial Offload** | High | Medium | Avoid partial layer offloading unless proven faster in Phase 1 benchmarks; default to 100% GPU offload or pure CPU. |
| **CUDA / Driver Incompatibility** | Low | High | Support Vulkan and CPU AVX2 backends as fully functional, zero-install alternatives. |
| **Orphaned Inference Processes** | Medium | Medium | Implement parent process PID tracking, heartbeat checks, and drop guards that terminate child processes on shutdown. |
| **egui UI Stutter During Inference** | Low | Medium | Strict thread decoupling: inference runs entirely on a background Tokio thread; tokens passed via non-blocking unbounded channels. |
