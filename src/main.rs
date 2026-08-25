mod cli;
mod commands;
mod error;
mod network;
mod protocol;
mod server;
mod transfer;
mod transport;
mod tunnel;

use cli::{parse, Command};
use error::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = parse();

    let env_filter = cli
        .log_level
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("shuttle=info");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter)))
        .init();

    match cli.command {
        Command::Send { path, address } => {
            commands::handle_send(&path, &address).await?;
        }
        Command::Receive {
            bind_address,
            output,
        } => {
            commands::handle_receive(&bind_address, &output).await?;
        }
        Command::Serve {
            path,
            bind,
            port,
            public,
            token,
        } => {
            commands::handle_serve(&path, &bind, port, public, token).await?;
        }
        Command::Tunnel {
            local_address,
            remote_address,
        } => {
            commands::handle_tunnel_client(&local_address, &remote_address).await?;
        }
    }

    Ok(())
}
