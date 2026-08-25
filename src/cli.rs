use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "shuttle")]
#[command(
    about = "a cross-platform utility for moving files, data, and services between machines."
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(global = true, long)]
    pub log_level: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    Send {
        #[arg(value_name = "PATH")]
        path: PathBuf,

        #[arg(value_name = "ADDRESS")]
        address: String,
    },

    Receive {
        #[arg(value_name = "BIND_ADDRESS")]
        bind_address: String,

        #[arg(value_name = "OUTPUT")]
        output: PathBuf,
    },

    Serve {
        #[arg(value_name = "PATH")]
        path: PathBuf,

        #[arg(long, default_value = "127.0.0.1")]
        bind: String,

        #[arg(long, default_value = "8080")]
        port: u16,

        #[arg(long)]
        public: bool,

        #[arg(long)]
        token: bool,
    },

    Tunnel {
        #[arg(value_name = "LOCAL_ADDRESS")]
        local_address: String,

        #[arg(value_name = "REMOTE_ADDRESS")]
        remote_address: String,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
