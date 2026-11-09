pub mod catalog;
pub mod downloader;
pub mod integrity;
pub mod parser;

use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::Sender;
use velocity_core::{VelocityError, VelocityResult};

pub use catalog::{find_curated_model, get_curated_models, CuratedModel};
pub use downloader::{DownloadProgress, ResumableDownloader};
pub use integrity::{compute_sha256, verify_sha256};
pub use parser::{inspect_gguf, ModelMetadata};

pub struct ModelManager {
    models_dir: PathBuf,
    downloader: ResumableDownloader,
}

impl ModelManager {
    pub fn new<P: Into<PathBuf>>(models_dir: P) -> Self {
        let dir = models_dir.into();
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        Self {
            models_dir: dir,
            downloader: ResumableDownloader::new(),
        }
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    /// Lists all valid `.gguf` models in the models directory with parsed metadata.
    pub fn list_models(&self) -> VelocityResult<Vec<ModelMetadata>> {
        if !self.models_dir.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        let entries = fs::read_dir(&self.models_dir)
            .map_err(|e| VelocityError::Model(format!("Failed to read models directory: {}", e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "gguf" {
                        match inspect_gguf(&path) {
                            Ok(meta) => models.push(meta),
                            Err(e) => {
                                tracing::warn!("Skipping unparseable GGUF {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        models.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        Ok(models)
    }

    /// Imports an existing local GGUF model file into the managed models directory.
    /// Validates the file header first to ensure it is valid GGUF before copying.
    pub fn import_local_model(&self, source_path: &Path, overwrite: bool) -> VelocityResult<ModelMetadata> {
        if !source_path.exists() {
            return Err(VelocityError::Model(format!(
                "Source file not found: {}",
                source_path.display()
            )));
        }

        // Validate GGUF file integrity before copying
        let meta = inspect_gguf(source_path)?;

        let target_path = self.models_dir.join(&meta.file_name);
        if target_path.exists() && !overwrite {
            return Err(VelocityError::Model(format!(
                "Model already exists in target directory: {}",
                target_path.display()
            )));
        }

        // If source is already at target path, do not copy
        if source_path != target_path {
            fs::copy(source_path, &target_path).map_err(|e| {
                VelocityError::Model(format!(
                    "Failed to copy model from {} to {}: {}",
                    source_path.display(),
                    target_path.display(),
                    e
                ))
            })?;
        }

        inspect_gguf(&target_path)
    }

    /// Deletes a model from the models directory.
    pub fn delete_model(&self, file_name: &str) -> VelocityResult<()> {
        let path = self.models_dir.join(file_name);
        if !path.exists() {
            return Err(VelocityError::Model(format!(
                "Model file not found: {}",
                path.display()
            )));
        }

        fs::remove_file(&path).map_err(|e| {
            VelocityError::Model(format!("Failed to delete model {}: {}", path.display(), e))
        })?;

        tracing::info!("Deleted model file: {}", path.display());
        Ok(())
    }

    /// Computes the SHA-256 hash of a model file.
    pub fn verify_integrity(&self, file_name: &str, expected_sha256: &str) -> VelocityResult<bool> {
        let path = self.models_dir.join(file_name);
        verify_sha256(&path, expected_sha256)
    }

    /// Downloads a curated or custom model with resumption support.
    pub async fn download_model(
        &self,
        url: &str,
        file_name: &str,
        progress_tx: Option<Sender<DownloadProgress>>,
    ) -> VelocityResult<ModelMetadata> {
        let target_path = self.models_dir.join(file_name);
        let completed_path = self.downloader.download(url, &target_path, progress_tx).await?;
        inspect_gguf(&completed_path)
    }

    /// Queries the size of a model before starting a download (so user can confirm).
    pub async fn query_download_size(&self, url: &str) -> VelocityResult<Option<u64>> {
        self.downloader.query_file_size(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_real_model() {
        let model_path = Path::new("../../models/qwen2.5-0.5b-instruct-q4_k_m.gguf");
        if model_path.exists() {
            let meta = inspect_gguf(model_path).expect("inspect gguf");
            assert_eq!(meta.architecture.as_deref(), Some("qwen2"));
            assert!(meta.tensor_count > 0);
            assert!(meta.file_size_mb > 400.0);
            assert!(meta.fits_gtx_1650);
            assert_eq!(meta.quantization.as_deref(), Some("Q4_K_M"));
            println!("Inspected metadata: {:?}", meta);
        }
    }

    #[test]
    fn test_curated_models_registry() {
        let models = get_curated_models();
        assert!(models.len() >= 4);
        let qwen05 = find_curated_model("qwen2.5-0.5b-q4").expect("qwen 0.5b in catalog");
        assert!(qwen05.fits_gtx_1650_vram);
        assert_eq!(qwen05.param_size, "0.5B");
    }

    #[test]
    fn test_import_and_delete_model() {
        let temp_dir = std::env::temp_dir().join("velocity_models_test");
        let mgr = ModelManager::new(&temp_dir);

        let source = Path::new("../../models/qwen2.5-0.5b-instruct-q4_k_m.gguf");
        if source.exists() {
            let imported = mgr.import_local_model(source, true).expect("import model");
            assert_eq!(imported.file_name, "qwen2.5-0.5b-instruct-q4_k_m.gguf");

            let list = mgr.list_models().expect("list models");
            assert!(list.iter().any(|m| m.file_name == imported.file_name));

            mgr.delete_model(&imported.file_name).expect("delete model");
            let list_after = mgr.list_models().expect("list after delete");
            assert!(!list_after.iter().any(|m| m.file_name == imported.file_name));
        }

        let _ = fs::remove_dir_all(temp_dir);
    }
}
