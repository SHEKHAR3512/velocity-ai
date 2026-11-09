# VELOCITY AI

**Native Offline AI Chatbot Engineered for Maximum Inference Speed on Limited Hardware.**

VELOCITY AI is a lightweight, high-performance desktop AI application written in Rust. Specifically tailored for consumer laptops such as the Acer Predator PT315-51 (Intel Core i5-9300H, NVIDIA GTX 1650 4GB VRAM, 16GB RAM), VELOCITY AI focuses on real-world response latency, zero bloat, and offline autonomy.

---

## Key Characteristics

- **Performance-First Architecture:** Built from the ground up in Rust (`egui`/`eframe`), consuming minimal memory (~20 MB UI RAM) with zero Electron or web-wrapper overhead.
- **Hardware-Aware Inference:** Automatically inspects CPU instruction extensions (AVX2), dedicated VRAM, and driver capabilities to select the optimal acceleration runtime (CUDA 12.4, Vulkan, or CPU AVX2).
- **100% Offline & Private:** All inference runs locally via `llama.cpp` using the GGUF model format. Chat history is stored locally in an embedded SQLite database. No data ever leaves your device.
- **Sub-Second Latency on 4GB GPUs:** Optimized for modern small language models (SLMs) such as Qwen2.5 (0.5B / 1.5B) and Llama 3.2 (1B / 3B) that fit entirely within 4GB VRAM.
- **Empirical Benchmarking Engine:** Built-in multi-trial benchmark harness comparing CPU thread scaling, GPU acceleration backends, and quantization schemes without guesswork.

---

## Workspace Crates

- [`crates/velocity-core`](crates/velocity-core): Domain types, error definitions, and logging.
- [`crates/velocity-hardware`](crates/velocity-hardware): Runtime CPU, RAM, GPU, and acceleration backend detection.
- [`crates/velocity-config`](crates/velocity-config): Persistent configuration and system directory resolution.
- [`crates/velocity-storage`](crates/velocity-storage): Local SQLite database for conversations, messages, and benchmarks.
- [`crates/velocity-models`](crates/velocity-models): GGUF header inspector, catalog manager, and resumable downloader.
- [`crates/velocity-engine`](crates/velocity-engine): External `llama-server` process supervisor and asynchronous SSE streaming client.
- [`crates/velocity-benchmark`](crates/velocity-benchmark): Automated multi-trial performance measurement harness.
- [`apps/desktop`](apps/desktop): Native egui/eframe desktop graphical interface.

---

## Documentation

- [Project Blueprint & Plan](docs/PROJECT_PLAN.md)
- [System Architecture](docs/ARCHITECTURE.md)
- [Hardware Audit Report](docs/HARDWARE_REPORT.md)
- [Model Research & Strategy](docs/MODEL_RESEARCH.md)
- [Phase Tracker](PHASE_TRACKER.md)

---

## License

MIT License. See [LICENSE](LICENSE) for details.

