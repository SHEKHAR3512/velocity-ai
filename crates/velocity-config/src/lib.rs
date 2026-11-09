use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use velocity_core::{BackendType, VelocityError, VelocityResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_model: String,
    pub backend: BackendType,
    pub threads: usize,
    pub gpu_layers: usize,
    pub context_size: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub models_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        let (models_dir, data_dir) = default_directories();
        Self {
            active_model: "qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string(),
            backend: BackendType::Vulkan,
            threads: 4,
            gpu_layers: 99,
            context_size: 2048,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 1024,
            models_dir,
            data_dir,
        }
    }
}

pub fn default_directories() -> (PathBuf, PathBuf) {
    // If local models directory exists in working folder, prefer local project folder
    let local_models = PathBuf::from("models");
    let local_data = PathBuf::from("data");

    if local_models.exists() {
        return (local_models, local_data);
    }

    if let Some(proj_dirs) = ProjectDirs::from("ai", "velocity", "VelocityAI") {
        let data_dir = proj_dirs.data_local_dir().to_path_buf();
        let models_dir = data_dir.join("models");
        (models_dir, data_dir)
    } else {
        (PathBuf::from("models"), PathBuf::from("data"))
    }
}

impl AppConfig {
    pub fn config_file_path() -> PathBuf {
        if Path::new("config.toml").exists() {
            return PathBuf::from("config.toml");
        }

        if let Some(proj_dirs) = ProjectDirs::from("ai", "velocity", "VelocityAI") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load() -> VelocityResult<Self> {
        let path = Self::config_file_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| VelocityError::Config(format!("Failed to read {}: {}", path.display(), e)))?;
            let cfg: Self = toml::from_str(&content)
                .map_err(|e| VelocityError::Config(format!("Failed to parse config {}: {}", path.display(), e)))?;
            Ok(cfg)
        } else {
            let default_cfg = Self::default();
            let _ = default_cfg.save();
            Ok(default_cfg)
        }
    }

    pub fn save(&self) -> VelocityResult<()> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(self)
            .map_err(|e| VelocityError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&path, serialized)
            .map_err(|e| VelocityError::Config(format!("Failed to write {}: {}", path.display(), e)))?;
        Ok(())
    }

    pub fn ensure_directories(&self) -> VelocityResult<()> {
        fs::create_dir_all(&self.models_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.threads, 4);
        assert_eq!(cfg.backend, BackendType::Vulkan);
    }
}

