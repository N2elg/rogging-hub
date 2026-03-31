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
        let io = tokio::runtime::Builder::new_multi_thread()
            .thread_name("io-worker")
            .enable_all()
            .build()
            .expect("Failed to build IO runtime");

        let parser = tokio::runtime::Builder::new_multi_thread()
            .thread_name("parser-worker")
            .worker_threads(cfg.parser_threads)
            .enable_all()
            .build()
            .expect("Failed to build parser runtime");

        let output = tokio::runtime::Builder::new_multi_thread()
            .thread_name("output-worker")
            .worker_threads(cfg.output_threads)
            .enable_all()
            .build()
            .expect("Failed to build output runtime");

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
