use clap::{Parser, Subcommand};
use anyhow::Result;
use std::env;

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
    /// Ethereum Safe wallet address (0x...)
    #[arg(short, long)]
    safe: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch transaction data from Safe transaction pool
    Tx {
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
    
    // Get RPC URL from environment variables
    let rpc = env::var("RPC_URL").ok();
    
    match cli.command {
        Some(Commands::Tx { hash, tx_pool }) => {
            // Pass the safe address from the top-level CLI
            commands::tx::execute(cli.safe, rpc, hash, tx_pool).await?;
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
