use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tunnel_common::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    loop {
        let Ok(mut tunnel_stream) = TcpStream::connect((PUBLIC_SERVER_HOST, PUBLIC_SERVER_PORT)).await else {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        };

        log::info!("Connected to tunnel server at {:?}", tunnel_stream.peer_addr());

        _ = tunnel_stream.write_all(SECRET_HANDSHAKE.as_bytes()).await;

        let Ok(local_stream) = TcpStream::connect(TUNNEL_LOCAL_HOST).await else {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        };

        log::info!("Connected to local service at {:?}", TUNNEL_LOCAL_HOST);

        let result = bind_streams(tunnel_stream, local_stream).await;

        log::error!("Streams binding ended with: {:?}", result);
    }
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
                tunnel_stream.write_all(&local_buffer[..n]).await.map_err(|err| anyhow::anyhow!("TunnelStream write error: {err}"))?;
            }
            Poll::Tunnel(Ok(n)) => {
                log::debug!("Forwarding {} bytes from TunnelStream to LocalStream", n);
                local_stream.write_all(&tunnel_buffer[..n]).await.map_err(|err| anyhow::anyhow!("LocalStream write error: {err}"))?;
            }
        }
    }
}
