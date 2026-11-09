use serde::{Deserialize, Serialize};
use std::fs::File;
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
    pub quantization: Option<String>,
    pub fits_gtx_1650: bool,
}

pub fn inspect_gguf(path: &Path) -> VelocityResult<ModelMetadata> {
    let file = File::open(path)
        .map_err(|e| VelocityError::Model(format!("Failed to open GGUF file {}: {}", path.display(), e)))?;
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
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

    // Read metadata key-value pairs (up to kv_count or first 96 keys)
    let keys_to_read = kv_count.min(96);
    for _ in 0..keys_to_read {
        if let Ok(key) = read_gguf_string(&mut reader) {
            let mut type_buf = [0u8; 4];
            if reader.read_exact(&mut type_buf).is_err() {
                break;
            }
            let val_type = u32::from_le_bytes(type_buf);

            if key == "general.architecture" && val_type == 8 {
                if let Ok(arch) = read_gguf_string(&mut reader) {
                    architecture = Some(arch);
                    continue;
                }
            } else if (key.ends_with(".context_length") || key == "general.context_length")
                && (val_type == 4 || val_type == 5)
            {
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

            // Skip unneeded values
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

    // Deduce quantization from filename
    let lower_name = file_name.to_lowercase();
    let quantization = if lower_name.contains("q4_k_m") {
        Some("Q4_K_M".to_string())
    } else if lower_name.contains("q4_0") {
        Some("Q4_0".to_string())
    } else if lower_name.contains("q5_k_m") {
        Some("Q5_K_M".to_string())
    } else if lower_name.contains("q8_0") {
        Some("Q8_0".to_string())
    } else if lower_name.contains("q2_k") {
        Some("Q2_K".to_string())
    } else if lower_name.contains("f16") {
        Some("F16".to_string())
    } else {
        None
    };

    // Acer Predator GTX 1650 has 4096 MB VRAM.
    // 3200 MB leaves ~800 MB for KV cache and Windows DWM desktop compositor.
    let fits_gtx_1650 = file_size_mb <= 3200.0;

    Ok(ModelMetadata {
        file_name,
        file_path: path.to_path_buf(),
        file_size_mb,
        architecture,
        tensor_count,
        context_length,
        quantization,
        fits_gtx_1650,
    })
}

fn read_gguf_string<R: Read>(reader: &mut R) -> std::io::Result<String> {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    if len > 2048 {
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
            // array: type (u32) + count (u64)
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

