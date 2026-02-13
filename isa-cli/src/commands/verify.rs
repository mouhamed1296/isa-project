use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence};

pub fn run(file: String, verbose: bool) -> Result<()> {
    // Try to load the state
    let persistence = FilePersistence::new(&file);
    let dummy_seed = [0u8; 32];
    let runtime = DeviceRuntime::load_or_create(dummy_seed, persistence)
        .context(format!("Failed to load state from {}", file))?;
    
    // If we got here, the state loaded successfully
    println!("{} State is valid", "✓".green().bold());
    println!("  File: {}", file.yellow());
    
    if verbose {
        let vector = runtime.state_vector();
        println!();
        println!("  State Vector:");
        println!("    Finance:  {}", hex::encode(&vector.finance).dimmed());
        println!("    Time:     {}", hex::encode(&vector.time).dimmed());
        println!("    Hardware: {}", hex::encode(&vector.hardware).dimmed());
    }
    
    Ok(())
}
