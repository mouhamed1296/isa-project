use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence};
use serde_json::json;

pub fn run(file: String, format: String) -> Result<()> {
    // Load the state
    let persistence = FilePersistence::new(&file);
    let dummy_seed = [0u8; 32];
    let runtime = DeviceRuntime::load_or_create(dummy_seed, persistence)
        .context(format!("Failed to load state from {}", file))?;
    
    let vector = runtime.state_vector();
    
    match format.as_str() {
        "json" => {
            let output = json!({
                "finance": hex::encode(&vector.finance),
                "time": hex::encode(&vector.time),
                "hardware": hex::encode(&vector.hardware),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "hex" => {
            println!("{}", hex::encode(&vector.finance));
            println!("{}", hex::encode(&vector.time));
            println!("{}", hex::encode(&vector.hardware));
        }
        "text" | _ => {
            println!("{}", "Device State".bold());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("{}", "Finance Axis:".cyan().bold());
            print_hex_grid(&vector.finance);
            println!();
            println!("{}", "Time Axis:".cyan().bold());
            print_hex_grid(&vector.time);
            println!();
            println!("{}", "Hardware Axis:".cyan().bold());
            print_hex_grid(&vector.hardware);
        }
    }
    
    Ok(())
}

fn print_hex_grid(data: &[u8; 32]) {
    for (i, chunk) in data.chunks(8).enumerate() {
        print!("  {:02x}  ", i * 8);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }
}
