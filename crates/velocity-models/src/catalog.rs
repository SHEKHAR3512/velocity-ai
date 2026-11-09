use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedModel {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub param_size: &'static str,
    pub quantization: &'static str,
    pub file_size_mb: f64,
    pub download_url: &'static str,
    pub default_filename: &'static str,
    pub recommended_backend: &'static str,
    pub expected_tps: &'static str,
    pub fits_gtx_1650_vram: bool,
}

pub static CURATED_MODELS: &[CuratedModel] = &[
    CuratedModel {
        id: "qwen2.5-0.5b-q4",
        name: "Qwen 2.5 0.5B Instruct",
        description: "Ultra-fast lightweight model. Instantaneous responses with near-zero VRAM pressure.",
        param_size: "0.5B",
        quantization: "Q4_K_M",
        file_size_mb: 491.0,
        download_url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf",
        default_filename: "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        recommended_backend: "Vulkan / CUDA (100% offload)",
        expected_tps: "130 - 162 tok/s",
        fits_gtx_1650_vram: true,
    },
    CuratedModel {
        id: "qwen2.5-1.5b-q4",
        name: "Qwen 2.5 1.5B Instruct",
        description: "Optimal sweet spot between high instruction accuracy and ultra-fast throughput.",
        param_size: "1.5B",
        quantization: "Q4_K_M",
        file_size_mb: 986.0,
        download_url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf",
        default_filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        recommended_backend: "Vulkan / CUDA (100% offload)",
        expected_tps: "70 - 95 tok/s",
        fits_gtx_1650_vram: true,
    },
    CuratedModel {
        id: "llama-3.2-1b-q4",
        name: "Llama 3.2 1B Instruct",
        description: "Meta's highly optimized lightweight reasoning and conversational model.",
        param_size: "1.2B",
        quantization: "Q4_K_M",
        file_size_mb: 809.0,
        download_url: "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf",
        default_filename: "llama-3.2-1b-instruct-q4_k_m.gguf",
        recommended_backend: "Vulkan / CUDA (100% offload)",
        expected_tps: "80 - 110 tok/s",
        fits_gtx_1650_vram: true,
    },
    CuratedModel {
        id: "smollm2-1.7b-q4",
        name: "SmolLM2 1.7B Instruct",
        description: "Hugging Face's state-of-the-art small reasoning model trained on synthetic data.",
        param_size: "1.7B",
        quantization: "Q4_K_M",
        file_size_mb: 1060.0,
        download_url: "https://huggingface.co/HuggingFaceTB/SmolLM2-1.7B-Instruct-GGUF/resolve/main/smollm2-1.7b-instruct-q4_k_m.gguf",
        default_filename: "smollm2-1.7b-instruct-q4_k_m.gguf",
        recommended_backend: "Vulkan / CUDA (100% offload)",
        expected_tps: "65 - 90 tok/s",
        fits_gtx_1650_vram: true,
    },
];

pub fn get_curated_models() -> &'static [CuratedModel] {
    CURATED_MODELS
}

pub fn find_curated_model(id: &str) -> Option<&'static CuratedModel> {
    CURATED_MODELS.iter().find(|m| m.id == id || m.default_filename == id)
}

