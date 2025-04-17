use std::io::{self, Write};
use rustyline::{error::ReadlineError, Editor};
use rustyline::DefaultEditor;
use anyhow::Result;

pub async fn execute() -> Result<()> {
    println!("Welcome to Vito CLI interactive mode. Type 'help' for available commands or 'exit' to quit.");
    
    let mut rl = DefaultEditor::new()?;
    loop {
        // Display prompt and get input
        let readline = rl.readline("vito > ");
        
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
                            println!("Transaction command: {}", &command[3..]);
                            // Here you would parse and execute the tx command
                            // This is a placeholder - you'll implement the actual parsing logic
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
    println!("  tx <args>   - Manage transactions (use 'tx --help' for more info)");
    println!("  help        - Show this help message");
    println!("  exit        - Exit interactive mode");
} 