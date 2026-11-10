use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use uuid::Uuid;
use velocity_core::{BackendType, Conversation, HardwareProfile, InferenceConfig, InferenceMetrics, Message};
use velocity_models::{CuratedModel, DownloadProgress, ModelMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Chat,
    Models,
    Hardware,
    Settings,
}

#[allow(dead_code)]
pub enum EngineCommand {
    StartEngine {
        config: InferenceConfig,
    },
    StopEngine,
    StreamChat {
        messages: Vec<Message>,
        config: InferenceConfig,
    },
    AbortInference,
    StartDownload {
        url: String,
        filename: String,
    },
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    EngineReady {
        backend: BackendType,
        port: u16,
    },
    EngineStopped,
    EngineError(String),
    StreamToken(String),
    StreamFinished(InferenceMetrics),
    StreamError(String),
    DownloadUpdate(DownloadProgress),
    DownloadDone(ModelMetadata),
    DownloadError(String),
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub conversations: Vec<Conversation>,
    pub current_conv_id: Option<Uuid>,
    pub messages: Vec<Message>,
    pub streaming_text: String,
    pub is_generating: bool,
    pub abort_flag: Arc<AtomicBool>,
    pub latest_metrics: Option<InferenceMetrics>,
    pub input_text: String,

    // Hardware & Engine state
    pub hardware: Option<HardwareProfile>,
    pub active_backend: BackendType,
    pub active_port: Option<u16>,
    pub is_engine_ready: bool,

    // Models state
    pub local_models: Vec<ModelMetadata>,
    pub curated_models: &'static [CuratedModel],
    pub selected_model: String,
    pub active_download: Option<DownloadProgress>,

    // Inference parameters
    pub threads: usize,
    pub gpu_layers: usize,
    pub context_size: usize,
    pub batch_size: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub system_prompt: String,

    // Status notifications
    pub status_message: Option<(String, bool)>, // (message, is_error)
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_tab: ActiveTab::Chat,
            conversations: Vec::new(),
            current_conv_id: None,
            messages: Vec::new(),
            streaming_text: String::new(),
            is_generating: false,
            abort_flag: Arc::new(AtomicBool::new(false)),
            latest_metrics: None,
            input_text: String::new(),

            hardware: None,
            active_backend: BackendType::Vulkan,
            active_port: None,
            is_engine_ready: false,

            local_models: Vec::new(),
            curated_models: velocity_models::get_curated_models(),
            selected_model: "models/qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string(),
            active_download: None,

            threads: 4,
            gpu_layers: 99,
            context_size: 2048,
            batch_size: 512,
            temperature: 0.7,
            top_p: 0.9,
            system_prompt: "You are VELOCITY AI, an ultra-fast, helpful, and concise local AI assistant.".to_string(),

            status_message: None,
        }
    }
}

