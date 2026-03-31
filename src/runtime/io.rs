pub fn build_io_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("io-worker")
        .enable_all()
        .build()
        .expect("Failed to build IO runtime")
}
