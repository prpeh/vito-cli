use std::io::{self, Write};
use rustyline::{error::ReadlineError, Editor};
use rustyline::DefaultEditor;
use anyhow::Result;
use std::env;
use colored::*;

// Helper function to shorten an Ethereum address
fn shorten_address(address: &str) -> String {
    if address.len() >= 10 {
        let prefix = &address[0..6];  // 0x1234
        let suffix = &address[address.len() - 4..];  // abcd
        format!("{}...{}", prefix, suffix)
    } else {
        address.to_string()
    }
}

pub async fn execute(safe: Option<String>) -> Result<()> {
    println!("Welcome to Vito CLI interactive mode. Type 'help' for available commands or 'exit' to quit.");
    
    // Use the provided safe address or start with none
    let current_safe = safe;
    if let Some(ref safe) = current_safe {
        println!("Current Safe address: {}", safe);
    } else {
        println!("{}", "No Safe address set. Please restart with --safe <address> option.".red());
    }
    
    // Get RPC URL from environment
    let rpc_url = env::var("RPC_URL").ok();
    if let Some(ref rpc) = rpc_url {
        println!("Using RPC URL: {}", rpc);
    } else {
        println!("No RPC URL set. Please set the RPC_URL environment variable.");
    }
    
    let mut rl = DefaultEditor::new()?;
    loop {
        // Create prompt based on whether a safe address is set
        let prompt = match current_safe {
            Some(ref safe) => {
                let addr_part = shorten_address(safe).dimmed().to_string();
                format!("{} vito> ", addr_part)
            },
            None => "vito> ".to_string(),
        };
        
        // Display prompt and get input
        let readline = rl.readline(&prompt);
        
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                
                // Process the command
                let command = line.trim();
                
                match command {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    },
                    "help" => {
                        display_help();
                    },
                    "" => {
                        // Do nothing for empty command
                    },
                    _ => {
                        if command.starts_with("tx ") {
                            if let Some(ref safe) = current_safe {
                                println!("Transaction command for Safe {}: {}", 
                                    safe.green(), 
                                    command[3..].bright_white());
                                // Here you would parse and execute the tx command
                                // This is a placeholder - you'll implement the actual parsing logic
                            } else {
                                println!("{}", "Error: No Safe address provided. Please restart with --safe <address> option.".red());
                            }
                        } else {
                            println!("{}: {}. Type 'help' for available commands.", 
                                "Unknown command".red(), 
                                command.yellow());
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    Ok(())
}

fn display_help() {
    println!("{}", "Available commands:".bright_green());
    println!("  {} - Manage transactions (use 'tx --help' for more info)", "tx <args>".yellow());
    println!("  {} - Show this help message", "help".yellow());
    println!("  {} - Exit interactive mode", "exit".yellow());
} 