use crate::config::ServerConfig;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::TcpListener;

pub fn create_listener(server: &ServerConfig) -> std::io::Result<TcpListener> {
    let addr: std::net::SocketAddr = server.listen_addr.parse().unwrap();
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.set_recv_buffer_size(server.sock_recv_buf)?;

    socket.bind(&addr.into())?;
    socket.listen(server.backlog)?;

    let std_listener: std::net::TcpListener = socket.into();
    TcpListener::from_std(std_listener)
}
