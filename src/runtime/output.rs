pub fn build_output_runtime(worker_threads: usize) -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("output-worker")
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .expect("Failed to build output runtime")
}
