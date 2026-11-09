use velocity_core::init_tracing;
use velocity_hardware::detect_hardware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    println!("VELOCITY AI Desktop Application entry point initialized.");
    let hw = detect_hardware()?;
    println!("Hardware verified: Recommended backend = {}", hw.recommended_backend);
    Ok(())
}

