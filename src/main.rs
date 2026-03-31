use RoggingHub::config;
use RoggingHub::config::LogConfig;
use RoggingHub::logging;
use RoggingHub::runtime;
use RoggingHub::server;
use tracing::info;

fn main() {
    let log_cfg = LogConfig::load();
    let _log_guards = logging::init(&log_cfg);

    let cfg = config::Config::load();

    info!(
        listen_addr = %cfg.server.listen_addr,
        max_connections = cfg.server.max_connections,
        write_mode = %cfg.output.file.write_mode,
        "RoggingHub starting"
    );

    let runtimes: runtime::Runtimes = runtime::Runtimes::build(&cfg.runtime);
    runtimes.io.block_on(server::serve(cfg, &runtimes));
}
