use std::io::{self, Write};
use rustyline::{error::ReadlineError, Editor};
use rustyline::DefaultEditor;
use anyhow::Result;
use std::env;

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

pub async fn execute() -> Result<()> {
    println!("Welcome to Vito CLI interactive mode. Type 'help' for available commands or 'exit' to quit.");
    
    // Try to get the safe from environment if it's not already set
    let mut current_safe = env::var("SAFE_ADDRESS").ok();
    if let Some(ref safe) = current_safe {
        println!("Current Safe address: {}", safe);
    } else {
        println!("No Safe address set. Use 'safe <address>' to set it.");
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
            Some(ref safe) => format!("{} vito> ", shorten_address(safe)),
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
                        if command.starts_with("safe ") {
                            let addr = command[5..].trim();
                            if !addr.is_empty() {
                                current_safe = Some(addr.to_string());
                                println!("Safe address set to: {}", addr);
                            } else {
                                println!("Please provide a Safe address");
                            }
                        } else if command.starts_with("tx ") {
                            if let Some(ref safe) = current_safe {
                                println!("Transaction command for Safe {}: {}", safe, &command[3..]);
                                // Here you would parse and execute the tx command
                                // This is a placeholder - you'll implement the actual parsing logic
                            } else {
                                println!("Error: No Safe address set. Use 'safe <address>' first.");
                            }
                        } else {
                            println!("Unknown command: {}. Type 'help' for available commands.", command);
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
    println!("Available commands:");
    println!("  safe <addr> - Set the Ethereum Safe wallet address (0x...)");
    println!("  tx <args>   - Manage transactions (use 'tx --help' for more info)");
    println!("  help        - Show this help message");
    println!("  exit        - Exit interactive mode");
} 