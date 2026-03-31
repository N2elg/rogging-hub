use RoggingHub::config;
use RoggingHub::logging;
use RoggingHub::runtime;
use RoggingHub::server;
use tracing::info;

fn main() {
    let cfg = config::Config::load();

    let (_console_guard, _file_guard) = logging::init(&cfg.log);

    info!(
        listen_addr = %cfg.server.listen_addr,
        max_connections = cfg.server.max_connections,
        write_mode = %cfg.output.file.write_mode,
        "RoggingHub starting"
    );

    let runtimes: runtime::Runtimes = runtime::Runtimes::build(&cfg.runtime);
    runtimes.io.block_on(server::serve(cfg, &runtimes));
}
