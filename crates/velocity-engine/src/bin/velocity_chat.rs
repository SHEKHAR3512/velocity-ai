use std::io::{self, Write};
use uuid::Uuid;
use velocity_core::{init_tracing, InferenceConfig, Message, Role};
use velocity_engine::{InferenceBackend, LlamaEngine};
use velocity_hardware::detect_hardware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    println!("============================================================");
    println!("       VELOCITY AI — High Performance Terminal Chatbot       ");
    println!("============================================================");

    // 1. Hardware Detection
    let hw = detect_hardware()?;
    println!("Detected CPU: {} (AVX2: {})", hw.cpu_name, hw.has_avx2);
    if let Some(gpu) = hw.gpus.iter().find(|g| g.is_discrete) {
        println!("Detected dGPU: {} ({} MB VRAM)", gpu.name, gpu.total_vram_mb);
    }
    println!("Recommended Backend: {}", hw.recommended_backend);

    // 2. Select Model
    let default_model = "models/qwen2.5-0.5b-instruct-q4_k_m.gguf";
    let model_path = if std::path::Path::new(default_model).exists() {
        default_model.to_string()
    } else if std::path::Path::new("../../models/qwen2.5-0.5b-instruct-q4_k_m.gguf").exists() {
        "../../models/qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string()
    } else {
        println!("Error: Model file not found at {}", default_model);
        return Ok(());
    };

    println!("Active Model: {}", model_path);

    // 3. Engine Configuration
    let config = InferenceConfig {
        model_path,
        backend: hw.recommended_backend,
        threads: hw.recommended_threads,
        gpu_layers: hw.recommended_gpu_layers,
        context_size: 2048,
        batch_size: 512,
        temperature: 0.7,
        top_p: 0.9,
        max_tokens: 1024,
    };

    println!("Starting inference engine with backend [{}]...", config.backend);
    let mut engine = LlamaEngine::new();
    engine.start(&config).await?;
    println!("Inference engine online on port {}!\n", engine.port());
    println!("Type your message and press Enter. Type '/quit' to exit.\n");

    let conv_id = Uuid::new_v4();
    let mut messages: Vec<Message> = vec![Message::new(
        conv_id,
        Role::System,
        "You are VELOCITY AI, an ultra-fast, helpful, and concise local AI assistant.",
    )];

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("User > ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "/quit" || trimmed == "/exit" {
            println!("\nShutting down VELOCITY AI engine...");
            engine.stop().await?;
            println!("Goodbye!");
            break;
        }

        let user_msg = Message::new(conv_id, Role::User, trimmed);
        messages.push(user_msg);

        print!("Assistant > ");
        stdout.flush()?;

        let mut stream = engine.stream_chat(&messages, &config).await?;
        let mut assistant_content = String::new();
        let mut last_metrics = None;

        while let Some(chunk) = stream.recv().await {
            if !chunk.delta.is_empty() {
                print!("{}", chunk.delta);
                stdout.flush()?;
                assistant_content.push_str(&chunk.delta);
            }
            if chunk.is_finished {
                last_metrics = chunk.metrics;
                break;
            }
        }
        println!();

        if let Some(m) = last_metrics {
            println!(
                "  [Metrics: {:.1} tokens/s | TTFT: {:.1} ms | Tokens: {} | Latency: {:.1} ms]",
                m.tokens_per_second,
                m.time_to_first_token_ms,
                m.completion_tokens,
                m.total_duration_ms
            );
        }
        println!();

        let assistant_msg = Message::new(conv_id, Role::Assistant, assistant_content);
        messages.push(assistant_msg);
    }

    Ok(())
}

