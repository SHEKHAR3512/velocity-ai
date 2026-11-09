use thiserror::Error;

#[derive(Error, Debug)]
pub enum VelocityError {
    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Inference engine error: {0}")]
    Engine(String),

    #[error("Benchmark error: {0}")]
    Benchmark(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type VelocityResult<T> = Result<T, VelocityError>;

