use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod config;

/// A CLI tool for Ethereum Safe operations
#[derive(Parser)]
#[command(
    name = "vito",
    author, 
    version, 
    about = "A powerful CLI tool for managing Safe wallet transactions", 
    long_about = "A feature-rich command-line interface tool designed to help you interact with Ethereum Safe wallets."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch transaction data from Safe transaction pool
    Tx {
        /// Ethereum Safe wallet address (0x...)
        #[arg(short, long)]
        safe: String,

        /// Provider RPC URL (http:// or https://) - Optional, defaults to Ethereum mainnet
        #[arg(short, long)]
        rpc: Option<String>,

        /// Transaction hash (0x...) - Optional
        #[arg(short = 't', long)]
        hash: Option<String>,

        /// Custom Safe transaction pool address (0x...) - Optional
        #[arg(long)]
        tx_pool: Option<String>,
    },
    
    /// Start interactive shell mode
    Shell,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Tx { safe, rpc, hash, tx_pool }) => {
            commands::tx::execute(safe, rpc, hash, tx_pool).await?;
        },
        Some(Commands::Shell) => {
            commands::shell::execute().await?;
        },
        None => {
            // No command provided, start interactive mode by default
            commands::shell::execute().await?;
        }
    }
    
    Ok(())
}
