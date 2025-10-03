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

    match tokio::io::copy_bidirectional(&mut local_stream, &mut tunnel_stream).await {
        Ok((from_local, from_tunnel)) => {
            log::info!("Connection closed. {from_local} bytes sent to tunnel, {from_tunnel} bytes sent to local");
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!("Stream error: {err}"))
    }
}
