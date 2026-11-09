use velocity_core::VelocityResult;

pub struct BenchmarkSuite {}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_latest_results() -> VelocityResult<String> {
        let path = std::path::Path::new("benchmarks/results/latest.md");
        if path.exists() {
            std::fs::read_to_string(path).map_err(Into::into)
        } else {
            Ok("No benchmark report found yet.".to_string())
        }
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

