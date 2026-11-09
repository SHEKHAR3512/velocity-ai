use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: Role,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<InferenceMetrics>,
}

impl Message {
    pub fn new(conversation_id: Uuid, role: Role, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            role,
            content: content.into(),
            created_at: Utc::now(),
            metrics: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub title: String,
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(title: impl Into<String>, model_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            model_id: model_id.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    Cpu,
    Vulkan,
    Cuda,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Cpu => write!(f, "CPU (AVX2)"),
            BackendType::Vulkan => write!(f, "Vulkan Compute"),
            BackendType::Cuda => write!(f, "CUDA 12.4"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_path: String,
    pub backend: BackendType,
    pub threads: usize,
    pub gpu_layers: usize,
    pub context_size: usize,
    pub batch_size: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            backend: BackendType::Vulkan,
            threads: 4,
            gpu_layers: 99,
            context_size: 2048,
            batch_size: 512,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InferenceMetrics {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub time_to_first_token_ms: f64,
    pub tokens_per_second: f64,
    pub total_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub is_finished: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<InferenceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub total_vram_mb: usize,
    pub free_vram_mb: usize,
    pub compute_capability: Option<String>,
    pub is_discrete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu_name: String,
    pub physical_cores: usize,
    pub logical_threads: usize,
    pub has_avx2: bool,
    pub total_ram_mb: usize,
    pub free_ram_mb: usize,
    pub gpus: Vec<GpuInfo>,
    pub has_vulkan: bool,
    pub has_cuda: bool,
    pub recommended_backend: BackendType,
    pub recommended_threads: usize,
    pub recommended_gpu_layers: usize,
}

