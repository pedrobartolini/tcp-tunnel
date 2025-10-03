# Transparent TCP Tunnel

A simple client-server application that creates a secure TCP tunnel between systems, allowing traffic to flow between networks that might otherwise be isolated.

## Overview

This project consists of two main components:

-   **Server**: Runs on a publicly accessible host and manages tunnel connections
-   **Client**: Runs on a local machine and connects to the server to establish the tunnel

## Features

-   Secure connection with a configurable handshake mechanism
-   Bidirectional data forwarding
-   Automatic reconnection on failure
-   Asynchronous I/O for high performance

## Configuration

Configuration is done via environment variables:

-   `PUBLIC_SERVER_HOST`: Hostname of the public server
-   `PUBLIC_SERVER_PORT`: Port on which the server listens for tunnel connections
-   `PRIVATE_SERVER_PORT`: Port on which the server listens for local connections
-   `TUNNEL_LOCAL_HOST`: Local service to tunnel (address:port)
-   `SECRET_HANDSHAKE`: Secret key for validating tunnel connections

## Usage

1. Create a `.env` file with your configuration
2. Start the server on your public host:
    ```
    cargo run --bin tunnel_server
    ```
3. Start the client on your local machine:
    ```
    cargo run --bin tunnel_client
    ```

## Building

```
cargo build --release
```

The compiled binaries will be available in the `target/release` directory.

## Todo

-   Add TLS support for secure communication
-   Implement extra communication to manage the Tunnel Client, e.g., to close the tunnel remotely, change the target address, etc.
-   Add TUI or GUI to configure the Tunnel Client, like said above
