use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_parser_threads")]
    pub parser_threads: usize,
    #[serde(default = "default_output_threads")]
    pub output_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            parser_threads: default_parser_threads(),
            output_threads: default_output_threads(),
        }
    }
}

fn default_parser_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn default_output_threads() -> usize {
    2
}
