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

    match tokio::io::copy_bidirectional(&mut local_stream, &mut tunnel_stream).await {
        Ok((from_local, from_tunnel)) => {
            log::info!("Connection closed. {from_local} bytes sent to tunnel, {from_tunnel} bytes sent to local");
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!("Stream error: {err}"))
    }
}
