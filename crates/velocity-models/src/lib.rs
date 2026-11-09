use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use velocity_core::{VelocityError, VelocityResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size_mb: f64,
    pub architecture: Option<String>,
    pub tensor_count: u64,
    pub context_length: Option<u64>,
}

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new<P: Into<PathBuf>>(models_dir: P) -> Self {
        Self {
            models_dir: models_dir.into(),
        }
    }

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
                        match Self::inspect_gguf(&path) {
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

    pub fn inspect_gguf(path: &Path) -> VelocityResult<ModelMetadata> {
        let file = File::open(path)
            .map_err(|e| VelocityError::Model(format!("Failed to open GGUF file {}: {}", path.display(), e)))?;
        let file_len = file
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0);
        let file_size_mb = (file_len as f64) / (1024.0 * 1024.0);

        let mut reader = BufReader::new(file);

        // Check magic bytes: GGUF (0x47, 0x47, 0x55, 0x46)
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"GGUF" {
            return Err(VelocityError::Model(format!(
                "Invalid GGUF magic in {}: expected 'GGUF', got {:?}",
                path.display(),
                magic
            )));
        }

        let mut u32_buf = [0u8; 4];
        reader.read_exact(&mut u32_buf)?;
        let version = u32::from_le_bytes(u32_buf);

        if !(2..=3).contains(&version) {
            return Err(VelocityError::Model(format!(
                "Unsupported GGUF version in {}: {}",
                path.display(),
                version
            )));
        }

        let mut u64_buf = [0u8; 8];
        reader.read_exact(&mut u64_buf)?;
        let tensor_count = u64::from_le_bytes(u64_buf);

        reader.read_exact(&mut u64_buf)?;
        let kv_count = u64::from_le_bytes(u64_buf);

        let mut architecture = None;
        let mut context_length = None;

        // Read metadata key-value pairs (up to kv_count or first 64 keys)
        let keys_to_read = kv_count.min(64);
        for _ in 0..keys_to_read {
            if let Ok(key) = read_gguf_string(&mut reader) {
                let mut type_buf = [0u8; 4];
                if reader.read_exact(&mut type_buf).is_err() {
                    break;
                }
                let val_type = u32::from_le_bytes(type_buf);

                if key == "general.architecture" && val_type == 8 {
                    // String type
                    if let Ok(arch) = read_gguf_string(&mut reader) {
                        architecture = Some(arch);
                        continue;
                    }
                } else if (key.ends_with(".context_length") || key == "general.context_length")
                    && (val_type == 4 || val_type == 5)
                {
                    // uint32 (4) or uint64 (5)
                    if val_type == 4 {
                        let mut v32 = [0u8; 4];
                        if reader.read_exact(&mut v32).is_ok() {
                            context_length = Some(u32::from_le_bytes(v32) as u64);
                        }
                    } else {
                        let mut v64 = [0u8; 8];
                        if reader.read_exact(&mut v64).is_ok() {
                            context_length = Some(u64::from_le_bytes(v64));
                        }
                    }
                    continue;
                }

                // Skip unneeded values to avoid reading whole file
                if skip_gguf_value(&mut reader, val_type).is_err() {
                    break;
                }
            } else {
                break;
            }
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.gguf")
            .to_string();

        Ok(ModelMetadata {
            file_name,
            file_path: path.to_path_buf(),
            file_size_mb,
            architecture,
            tensor_count,
            context_length,
        })
    }
}

fn read_gguf_string<R: Read>(reader: &mut R) -> std::io::Result<String> {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    if len > 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "String length too large for metadata key",
        ));
    }
    let mut str_buf = vec![0u8; len];
    reader.read_exact(&mut str_buf)?;
    String::from_utf8(str_buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn skip_gguf_value<R: Read + Seek>(reader: &mut R, val_type: u32) -> std::io::Result<()> {
    match val_type {
        0 | 1 | 7 => reader.seek(SeekFrom::Current(1)).map(|_| ()), // u8, i8, bool
        2 | 3 => reader.seek(SeekFrom::Current(2)).map(|_| ()),     // u16, i16
        4 | 5 | 6 => reader.seek(SeekFrom::Current(4)).map(|_| ()), // u32, i32, f32
        8 => {
            // string
            let mut len_buf = [0u8; 8];
            reader.read_exact(&mut len_buf)?;
            let len = u64::from_le_bytes(len_buf) as i64;
            reader.seek(SeekFrom::Current(len)).map(|_| ())
        }
        9 => {
            // array: type (u32) + len (u64)
            let mut arr_type_buf = [0u8; 4];
            reader.read_exact(&mut arr_type_buf)?;
            let elem_type = u32::from_le_bytes(arr_type_buf);
            let mut arr_len_buf = [0u8; 8];
            reader.read_exact(&mut arr_len_buf)?;
            let count = u64::from_le_bytes(arr_len_buf);
            for _ in 0..count {
                skip_gguf_value(reader, elem_type)?;
            }
            Ok(())
        }
        10 | 11 | 12 => reader.seek(SeekFrom::Current(8)).map(|_| ()), // u64, i64, f64
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown GGUF value type: {}", val_type),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_real_model() {
        let model_path = Path::new("../../models/qwen2.5-0.5b-instruct-q4_k_m.gguf");
        if model_path.exists() {
            let meta = ModelManager::inspect_gguf(model_path).expect("inspect gguf");
            assert_eq!(meta.architecture.as_deref(), Some("qwen2"));
            assert!(meta.tensor_count > 0);
            assert!(meta.file_size_mb > 400.0);
            println!("Inspected metadata: {:?}", meta);
        }
    }
}

