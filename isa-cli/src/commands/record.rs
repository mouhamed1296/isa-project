use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence, EventAxis};

pub fn run(file: String, event: String, entropy: Option<String>, delta_t: u64) -> Result<()> {
    // Load existing runtime
    let persistence = FilePersistence::new(&file);
    let dummy_seed = [0u8; 32];
    let mut runtime = DeviceRuntime::load_or_create(dummy_seed, persistence)
        .context(format!("Failed to load state from {}", file))?;
    
    // Parse event data (try hex first, then use as string)
    let event_bytes = parse_data(&event)?;
    
    // Parse entropy if provided
    let entropy_bytes = if let Some(e) = entropy {
        parse_data(&e)?
    } else {
        vec![0u8; 16] // Default entropy
    };
    
    // Record event on finance axis (default)
    let _vector = runtime.record_event(EventAxis::Finance, &event_bytes)
        .context("Failed to record event")?;
    
    // Save updated state
    runtime.save()
        .context("Failed to save state")?;
    
    println!("{} Event recorded", "✓".green().bold());
    println!("  Event: {} bytes", event_bytes.len().to_string().cyan());
    println!("  Entropy: {} bytes", entropy_bytes.len().to_string().cyan());
    println!("  Delta-t: {} ms", delta_t.to_string().cyan());
    
    let vector = runtime.state_vector();
    println!("  State updated:");
    println!("    Finance:  {}", hex::encode(&vector.finance[..8]).yellow());
    println!("    Time:     {}", hex::encode(&vector.time[..8]).yellow());
    println!("    Hardware: {}", hex::encode(&vector.hardware[..8]).yellow());
    
    Ok(())
}

fn parse_data(input: &str) -> Result<Vec<u8>> {
    // Try to parse as hex first
    if let Ok(bytes) = hex::decode(input) {
        return Ok(bytes);
    }
    
    // Otherwise use as UTF-8 string
    Ok(input.as_bytes().to_vec())
}
