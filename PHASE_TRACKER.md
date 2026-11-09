# VELOCITY AI — Phase Tracker & Milestone Status

This document tracks the phased development of **VELOCITY AI**, a performance-first native Rust desktop chatbot engineered specifically for the Acer Predator PT315-51 laptop.

---

## Progress Overview

| Phase | Description | Status | Verification Gate |
|---|---|---|---|
| **Phase 0** | Research, Planning & Hardware Audit | **Completed** | Hardware & Architecture Docs Approved |
| **Phase 1** | Inference Benchmarking (CPU vs Vulkan vs CUDA) | **In Progress** | Empirically Measured Benchmarks Recorded |
| **Phase 2** | Rust Core & Terminal Chatbot | **Pending** | Working Async Terminal Streaming Client |
| **Phase 3** | Local Model Manager | **Pending** | Offline Model Import & Download Resumption |
| **Phase 4** | Native egui Desktop Application | **Pending** | Responsive 60 FPS egui UI with Streaming & Markdown |
| **Phase 5** | Advanced Optimization (KV cache, Threads, Speculative) | **Pending** | Fastest Verified Hardware Config Identified |
| **Phase 6** | Packaging, Hardening & Offline Verification | **Pending** | Clean Offline Release & Production Build |

---

## Detailed Milestone Log

### Phase 0: Research & Planning
- [x] Development environment ground-truth inspection (Windows 11, rustc 1.99.1, cargo 1.99.1, git, Python 3.12).
- [x] Install Gravitas and AAS engineering skills (`gravitas`, `performance-engineer`, `systematic-debugging`, `rust-pro`, `rust-async-patterns`, etc.).
- [x] Hardware capability detection (Intel Core i5-9300H AVX2, 16 GB RAM, NVIDIA GTX 1650 4GB VRAM, Intel UHD 630, Vulkan DLL, CUDA UMD 13.4).
- [x] External llama.cpp release verification (portable prebuilt runtime builds for CPU, Vulkan, and CUDA 12.4).
- [x] Create `docs/HARDWARE_REPORT.md`.
- [x] Create `docs/MODEL_RESEARCH.md`.
- [x] Create `docs/ARCHITECTURE.md`.
- [x] Create `docs/PROJECT_PLAN.md`.

### Phase 1: Inference Benchmark
- [ ] Acquire llama.cpp portable binaries (CPU AVX2, Vulkan, CUDA 12.4).
- [ ] Acquire small candidate model GGUF (e.g. `Qwen2.5-1.5B-Instruct-Q4_K_M.gguf` / `Llama-3.2-1B-Instruct-Q4_K_M.gguf`).
- [ ] Benchmark CPU thread scaling (2, 4, 6, 8 threads).
- [ ] Benchmark Vulkan backend vs CUDA backend vs CPU AVX2.
- [ ] Benchmark Quantizations (Q4_K_M vs Q5_K_M vs Q8_0).
- [ ] Record empirical results in `benchmarks/results/latest.md` and `benchmarks/results/latest.json`.
- [ ] Select recommended default configuration based on real data (not assumptions).

### Phase 2: Rust Core Workspace
- [ ] Initialize modular Cargo workspace:
  - `crates/velocity-core`
  - `crates/velocity-hardware`
  - `crates/velocity-config`
  - `crates/velocity-storage`
  - `crates/velocity-engine`
  - `crates/velocity-models`
  - `crates/velocity-benchmark`
- [ ] Implement runtime hardware detector in `velocity-hardware`.
- [ ] Implement `llama-server` process manager with health check, auto-restart, and graceful shutdown in `velocity-engine`.
- [ ] Implement asynchronous SSE token streaming client in `velocity-engine`.
- [ ] Implement SQLite chat and settings storage in `velocity-storage`.
- [ ] Build interactive terminal chat binary for end-to-end verification.

### Phase 3: Model Manager
- [ ] Configurable model directory (`%LOCALAPPDATA%/VelocityAI/models/` and custom paths).
- [ ] Local GGUF file importer with integrity and metadata verification.
- [ ] Resumable GGUF downloader with file size and SHA256 checks.
- [ ] Model selector, loader, and unloader.
- [ ] Unit & integration tests for model management.

### Phase 4: Native egui Desktop Application
- [ ] `apps/desktop` setup using `egui` and `eframe`.
- [ ] Clean, minimalist dark theme (macOS / Raycast / Linear design aesthetic).
- [ ] Left sidebar: New Chat, History, Models, Benchmarks, Settings.
- [ ] Main chat panel: Streamed token rendering, markdown formatting, syntax highlighted code blocks, copy button.
- [ ] Bottom status bar: Active model, backend (CPU/Vulkan/CUDA), threads, tokens/sec, TTFT, RAM/VRAM usage.
- [ ] Conversation persistence and restoration from SQLite.

### Phase 5: Advanced Optimization
- [ ] Benchmarking harness integration into the UI.
- [ ] Auto-configuration engine (`AUTO MODE`, `MAXIMUM SPEED`, `BALANCED`, `QUALITY`).
- [ ] Flash attention, KV cache quantization, and context-size tuning.
- [ ] Experimental speculative decoding validation (evaluating draft model overhead vs gain).

### Phase 6: Packaging & Finalization
- [ ] Production error handling and edge-case resilience tests.
- [ ] Standalone Windows release package.
- [ ] 100% offline flight check (disconnecting network and testing all features).
- [ ] Complete documentation, user guide, and architecture diagrams.
