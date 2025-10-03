use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tunnel_common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let tunnel_server = TcpListener::bind(("0.0.0.0", PUBLIC_SERVER_PORT)).await?;
    let local_server = TcpListener::bind(("127.0.0.1", PRIVATE_SERVER_PORT)).await?;

    log::info!("Listening for tunnel connections on {:?}", tunnel_server.local_addr());
    log::info!("Listening for local connections on {:?}", local_server.local_addr());

    loop {
        let Ok((mut tunnel_stream, _)) = tunnel_server.accept().await else { continue };
        log::info!("Accepted tunnel connection from {:?}", tunnel_stream.peer_addr());

        validate_tunnel_stream(&mut tunnel_stream, &SECRET_HANDSHAKE).await.map_err(|err| anyhow::anyhow!("Failed to validate tunnel stream: {err}"))?;
        log::info!("Tunnel stream validated");

        let Ok((local_stream, _)) = local_server.accept().await else { continue };
        log::info!("Accepted local connection from {:?}", local_stream.peer_addr());

        let result = bind_streams(tunnel_stream, local_stream).await;
        log::error!("Streams binding ended with: {:?}", result);
    }
}

async fn validate_tunnel_stream(stream: &mut TcpStream, expected: &str) -> anyhow::Result<()> {
    let mut buffer = vec![0u8; expected.len()];

    log::info!("Waiting for tunnel handshake");

    stream.read_exact(&mut buffer).await.map_err(|err| anyhow::anyhow!("Failed to read from tunnel stream: {err}"))?;

    log::info!("Received tunnel handshake: {:?}", String::from_utf8_lossy(&buffer));

    if buffer != expected.as_bytes() {
        return Err(anyhow::anyhow!("Invalid handshake from tunnel stream"));
    }

    Ok(())
}

async fn bind_streams(mut tunnel_stream: TcpStream, mut local_stream: TcpStream) -> anyhow::Result<()> {
    log::info!("Tunnel and local streams bound");

    let mut tunnel_buffer = [0u8; 4096];
    let mut local_buffer = [0u8; 4096];

    loop {
        enum Poll {
            Tunnel(tokio::io::Result<usize>),
            Local(tokio::io::Result<usize>)
        }

        let poll = tokio::select! {
            biased;
            res = tunnel_stream.read(&mut tunnel_buffer) => Poll::Tunnel(res),
            res = local_stream.read(&mut local_buffer) => Poll::Local(res),
        };

        match poll {
            Poll::Local(Err(err)) => return Err(anyhow::anyhow!("LocalStream read error: {err}")),
            Poll::Tunnel(Err(err)) => return Err(anyhow::anyhow!("TunnelStream read error: {err}")),
            Poll::Local(Ok(0)) => return Err(anyhow::anyhow!("LocalStream closed")),
            Poll::Tunnel(Ok(0)) => return Err(anyhow::anyhow!("TunnelStream closed")),
            Poll::Local(Ok(n)) => {
                log::debug!("Forwarding {} bytes from LocalStream to TunnelStream", n);
                tunnel_stream.write_all(&local_buffer[..n]).await.map_err(|err| anyhow::anyhow!("TunnelStream write error: {err}"))?
            }
            Poll::Tunnel(Ok(n)) => {
                log::debug!("Forwarding {} bytes from TunnelStream to LocalStream", n);
                local_stream.write_all(&tunnel_buffer[..n]).await.map_err(|err| anyhow::anyhow!("LocalStream write error: {err}"))?
            }
        }
    }
}
