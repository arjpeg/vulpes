mod net;

use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::EnvFilter;

use crate::net::Server;

/// The CLI arguments when running the server.
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(version, about = "The server binary for a Vulpes instance")]
pub struct Args {
    /// Address to bind the server to (IP:PORT)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let args = Args::parse();

    let server = Server::new(args.bind);
    server.run().await?;

    Ok(())
}
