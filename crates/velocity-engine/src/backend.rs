use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use velocity_core::{BackendType, InferenceConfig, Message, StreamChunk, VelocityResult};

#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn start(&mut self, config: &InferenceConfig) -> VelocityResult<()>;
    async fn stop(&mut self) -> VelocityResult<()>;
    fn is_running(&self) -> bool;
    async fn stream_chat(
        &self,
        messages: &[Message],
        config: &InferenceConfig,
    ) -> VelocityResult<Receiver<StreamChunk>>;
    fn backend_type(&self) -> BackendType;
}

