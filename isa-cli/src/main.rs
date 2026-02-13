use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;

#[derive(Parser)]
#[command(name = "isa")]
#[command(author, version, about = "MA-ISA command-line tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new device state
    Init {
        /// Device identifier
        device_id: String,
        
        /// Output file path (default: <device_id>.state)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Master seed as hex string (generates random if not provided)
        #[arg(short, long)]
        seed: Option<String>,
    },
    
    /// Record a sale transaction
    RecordSale {
        /// State file path
        #[arg(short, long, default_value = "device.state")]
        file: String,
        
        /// Sale amount in cents
        #[arg(short, long)]
        amount: u64,
        
        /// Currency code
        #[arg(short, long, default_value = "USD")]
        currency: String,
        
        /// Additional metadata
        #[arg(short, long)]
        metadata: Option<String>,
    },
    
    /// Record a custom event
    Record {
        /// State file path
        #[arg(short, long, default_value = "device.state")]
        file: String,
        
        /// Event data (hex or string)
        event: String,
        
        /// Entropy data (hex or string, optional)
        #[arg(short, long)]
        entropy: Option<String>,
        
        /// Time delta in milliseconds
        #[arg(short, long, default_value = "1000")]
        delta_t: u64,
    },
    
    /// Verify state integrity
    Verify {
        /// State file path
        file: String,
        
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show current state
    Show {
        /// State file path
        file: String,
        
        /// Output format: text, json, hex
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Compare two device states
    Compare {
        /// First state file
        file1: String,
        
        /// Second state file
        file2: String,
        
        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Init { device_id, output, seed } => {
            commands::init::run(device_id, output, seed)
        }
        Commands::RecordSale { file, amount, currency, metadata } => {
            commands::record_sale::run(file, amount, currency, metadata)
        }
        Commands::Record { file, event, entropy, delta_t } => {
            commands::record::run(file, event, entropy, delta_t)
        }
        Commands::Verify { file, verbose } => {
            commands::verify::run(file, verbose)
        }
        Commands::Show { file, format } => {
            commands::show::run(file, format)
        }
        Commands::Compare { file1, file2, format } => {
            commands::compare::run(file1, file2, format)
        }
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
