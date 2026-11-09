pub mod backend;
pub mod client;
pub mod supervisor;

use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use velocity_core::{BackendType, InferenceConfig, Message, StreamChunk, VelocityError, VelocityResult};

pub use backend::InferenceBackend;
pub use client::LlamaHttpClient;
pub use supervisor::LlamaServerSupervisor;

pub struct LlamaEngine {
    supervisor: LlamaServerSupervisor,
    client: LlamaHttpClient,
}

impl LlamaEngine {
    pub fn new() -> Self {
        Self {
            supervisor: LlamaServerSupervisor::new(),
            client: LlamaHttpClient::new(),
        }
    }

    pub fn port(&self) -> u16 {
        self.supervisor.port()
    }

    pub async fn warmup(&self, config: &InferenceConfig) -> VelocityResult<()> {
        let warmup_messages = [Message::new(
            uuid::Uuid::nil(),
            velocity_core::Role::User,
            "hi",
        )];
        let mut warmup_cfg = config.clone();
        warmup_cfg.max_tokens = 1;
        if let Ok(mut rx) = self
            .client
            .stream_chat(self.supervisor.port(), &warmup_messages, &warmup_cfg)
            .await
        {
            while let Some(chunk) = rx.recv().await {
                if chunk.is_finished {
                    break;
                }
            }
            tracing::info!("Model warm-up completed successfully");
        }
        Ok(())
    }
}

impl Default for LlamaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceBackend for LlamaEngine {
    async fn start(&mut self, config: &InferenceConfig) -> VelocityResult<()> {
        self.supervisor.start(config).await?;
        let _ = self.warmup(config).await;
        Ok(())
    }

    async fn stop(&mut self) -> VelocityResult<()> {
        self.supervisor.stop().await
    }

    fn is_running(&self) -> bool {
        self.supervisor.is_running()
    }

    async fn stream_chat(
        &self,
        messages: &[Message],
        config: &InferenceConfig,
    ) -> VelocityResult<Receiver<StreamChunk>> {
        if !self.is_running() {
            return Err(VelocityError::Engine(
                "Cannot stream: inference engine is not running".to_string(),
            ));
        }

        self.client
            .stream_chat(self.supervisor.port(), messages, config)
            .await
    }

    fn backend_type(&self) -> BackendType {
        self.supervisor.backend()
    }
}

