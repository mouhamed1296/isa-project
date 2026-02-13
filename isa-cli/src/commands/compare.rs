use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence};
use isa_core::CircularDistance;
use serde_json::json;

pub fn run(file1: String, file2: String, format: String) -> Result<()> {
    // Load both states
    let persistence1 = FilePersistence::new(&file1);
    let dummy_seed = [0u8; 32];
    let runtime1 = DeviceRuntime::load_or_create(dummy_seed, persistence1)
        .context(format!("Failed to load state from {}", file1))?;
    
    let persistence2 = FilePersistence::new(&file2);
    let runtime2 = DeviceRuntime::load_or_create(dummy_seed, persistence2)
        .context(format!("Failed to load state from {}", file2))?;
    
    let vector1 = runtime1.state_vector();
    let vector2 = runtime2.state_vector();
    
    // Calculate divergence
    let finance_div = CircularDistance::compute(&vector1.finance, &vector2.finance);
    let time_div = CircularDistance::compute(&vector1.time, &vector2.time);
    let hardware_div = CircularDistance::compute(&vector1.hardware, &vector2.hardware);
    
    // Calculate magnitude (sum of first 8 bytes as rough estimate)
    let finance_mag = magnitude(&finance_div);
    let time_mag = magnitude(&time_div);
    let hardware_mag = magnitude(&hardware_div);
    let total_mag = finance_mag + time_mag + hardware_mag;
    
    match format.as_str() {
        "json" => {
            let output = json!({
                "file1": file1,
                "file2": file2,
                "divergence": {
                    "finance": hex::encode(&finance_div),
                    "time": hex::encode(&time_div),
                    "hardware": hex::encode(&hardware_div),
                },
                "magnitude": {
                    "finance": finance_mag,
                    "time": time_mag,
                    "hardware": hardware_mag,
                    "total": total_mag,
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "text" | _ => {
            println!("{}", "State Comparison".bold());
            println!("{}", "=".repeat(60).dimmed());
            println!();
            println!("  File 1: {}", file1.yellow());
            println!("  File 2: {}", file2.yellow());
            println!();
            println!("{}", "Divergence:".cyan().bold());
            println!("  Finance:  {} (magnitude: {})", 
                hex::encode(&finance_div[..8]).dimmed(), 
                format_magnitude(finance_mag));
            println!("  Time:     {} (magnitude: {})", 
                hex::encode(&time_div[..8]).dimmed(), 
                format_magnitude(time_mag));
            println!("  Hardware: {} (magnitude: {})", 
                hex::encode(&hardware_div[..8]).dimmed(), 
                format_magnitude(hardware_mag));
            println!();
            println!("  Total Magnitude: {}", format_magnitude(total_mag).bold());
            
            if total_mag == 0 {
                println!();
                println!("{} States are identical", "✓".green().bold());
            } else if total_mag < 1000 {
                println!();
                println!("{} States are very similar", "~".yellow().bold());
            } else {
                println!();
                println!("{} States have diverged significantly", "!".red().bold());
            }
        }
    }
    
    Ok(())
}

fn magnitude(data: &[u8; 32]) -> u64 {
    // Sum first 8 bytes as u64 for rough magnitude estimate
    let mut sum = 0u64;
    for &byte in &data[..8] {
        sum = sum.wrapping_add(byte as u64);
    }
    sum
}

fn format_magnitude(mag: u64) -> ColoredString {
    if mag == 0 {
        "0".green()
    } else if mag < 100 {
        mag.to_string().yellow()
    } else {
        mag.to_string().red()
    }
}
