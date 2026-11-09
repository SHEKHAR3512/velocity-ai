use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use velocity_core::{BackendType, InferenceConfig, VelocityError, VelocityResult};

pub struct LlamaServerSupervisor {
    child: Option<Child>,
    port: u16,
    backend: BackendType,
}

impl LlamaServerSupervisor {
    pub fn new() -> Self {
        Self {
            child: None,
            port: 0,
            backend: BackendType::Vulkan,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    pub fn backend(&self) -> BackendType {
        self.backend
    }

    fn find_available_port() -> VelocityResult<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| VelocityError::Engine(format!("Failed to bind ephemeral port: {}", e)))?;
        let port = listener
            .local_addr()
            .map_err(|e| VelocityError::Engine(format!("Failed to get local address: {}", e)))?
            .port();
        drop(listener);
        Ok(port)
    }

    fn resolve_binary_dir(backend: BackendType) -> VelocityResult<(PathBuf, PathBuf)> {
        let base_dirs = [
            PathBuf::from("bin"),
            PathBuf::from("../../bin"),
            PathBuf::from("../../../bin"),
        ];

        let sub_folder = match backend {
            BackendType::Vulkan => "llama-vulkan",
            BackendType::Cuda => "llama-cuda",
            BackendType::Cpu => "llama-cpu",
        };

        for base in &base_dirs {
            let candidate_dir = base.join(sub_folder);
            let candidate_exe = candidate_dir.join("llama-server.exe");
            if candidate_exe.exists() {
                let abs_dir = std::fs::canonicalize(&candidate_dir).unwrap_or(candidate_dir);
                let abs_exe = std::fs::canonicalize(&candidate_exe).unwrap_or(candidate_exe);
                return Ok((abs_dir, abs_exe));
            }
        }

        Err(VelocityError::Engine(format!(
            "llama-server.exe for backend '{}' not found in any bin/ directory",
            backend
        )))
    }

    pub async fn start(&mut self, config: &InferenceConfig) -> VelocityResult<()> {
        if self.is_running() {
            self.stop().await?;
        }

        let model_path = Path::new(&config.model_path);
        if !model_path.exists() {
            return Err(VelocityError::Engine(format!(
                "Model file not found at: {}",
                config.model_path
            )));
        }

        let abs_model_path = std::fs::canonicalize(model_path)
            .unwrap_or_else(|_| model_path.to_path_buf());

        let port = Self::find_available_port()?;
        let (bin_dir, exe_path) = Self::resolve_binary_dir(config.backend)?;

        tracing::info!(
            "Starting llama-server [{}] on 127.0.0.1:{} with model {}",
            config.backend,
            port,
            abs_model_path.display()
        );

        let mut cmd = Command::new(&exe_path);
        cmd.current_dir(&bin_dir);
        cmd.args([
            "-m",
            &abs_model_path.to_string_lossy(),
            "--port",
            &port.to_string(),
            "--host",
            "127.0.0.1",
            "-c",
            &config.context_size.to_string(),
            "-t",
            &config.threads.to_string(),
            "-ngl",
            &config.gpu_layers.to_string(),
            "-b",
            &config.batch_size.to_string(),
        ]);

        match config.backend {
            BackendType::Vulkan => {
                // On NVIDIA laptop with Intel iGPU, Vulkan1 is the NVIDIA GTX 1650
                cmd.args(["-dev", "Vulkan1"]);
            }
            BackendType::Cuda => {
                cmd.env("CUDA_VISIBLE_DEVICES", "0");
            }
            BackendType::Cpu => {}
        }

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| VelocityError::Engine(format!("Failed to spawn llama-server: {}", e)))?;

        self.child = Some(child);
        self.port = port;
        self.backend = config.backend;

        // Poll health endpoint until server is ready
        self.wait_for_ready(port).await?;

        tracing::info!("llama-server is healthy and ready for inference on port {}", port);
        Ok(())
    }

    async fn wait_for_ready(&mut self, port: u16) -> VelocityResult<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(800))
            .build()
            .unwrap();

        let health_url = format!("http://127.0.0.1:{}/health", port);
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(45);

        while start.elapsed() < timeout {
            // Check if child exited prematurely
            if let Some(child) = &mut self.child {
                if let Ok(Some(status)) = child.try_wait() {
                    return Err(VelocityError::Engine(format!(
                        "llama-server terminated unexpectedly with status: {}",
                        status
                    )));
                }
            }

            if let Ok(resp) = client.get(&health_url).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }

            sleep(Duration::from_millis(250)).await;
        }

        // Timeout reached, kill process
        let _ = self.stop().await;
        Err(VelocityError::Engine(format!(
            "llama-server failed to respond to /health within {:?}",
            timeout
        )))
    }

    pub async fn stop(&mut self) -> VelocityResult<()> {
        if let Some(mut child) = self.child.take() {
            tracing::info!("Stopping llama-server process...");
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.port = 0;
        Ok(())
    }
}

impl Drop for LlamaServerSupervisor {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}

