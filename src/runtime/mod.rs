mod io;
mod output;
mod parser;

use crate::config::RuntimeConfig;
use std::sync::Arc;
use tracing::info;

pub struct Runtimes {
    pub io: tokio::runtime::Runtime,
    pub parser: Arc<tokio::runtime::Runtime>,
    pub output: Arc<tokio::runtime::Runtime>,
}

impl Runtimes {
    pub fn build(cfg: &RuntimeConfig) -> Self {
        let io = io::build_io_runtime();
        let parser = parser::build_parser_runtime(cfg.parser_threads);
        let output = output::build_output_runtime(cfg.output_threads);

        info!(
            parser_threads = cfg.parser_threads,
            output_threads = cfg.output_threads,
            "Runtimes created"
        );

        Self {
            io,
            parser: Arc::new(parser),
            output: Arc::new(output),
        }
    }
}
