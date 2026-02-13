use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence, EventAxis};

pub fn run(file: String, amount: u64, currency: String, metadata: Option<String>) -> Result<()> {
    // Load existing runtime - use dummy seed since we're loading
    let persistence = FilePersistence::new(&file);
    let dummy_seed = [0u8; 32];
    let mut runtime = DeviceRuntime::load_or_create(dummy_seed, persistence)
        .context(format!("Failed to load state from {}", file))?;
    
    // Format sale data
    let sale_data = if let Some(ref meta) = metadata {
        format!("sale:{}:{}:{}", amount, currency, meta)
    } else {
        format!("sale:{}:{}", amount, currency)
    };
    
    // Record sale on finance axis
    let vector = runtime.record_event(EventAxis::Finance, sale_data.as_bytes())
        .context("Failed to record sale")?;
    
    // Save updated state
    runtime.save()
        .context("Failed to save state")?;
    
    println!("{} Sale recorded", "✓".green().bold());
    println!("  Amount: {} {}", format_amount(amount).cyan(), currency.cyan());
    if let Some(meta) = metadata {
        println!("  Metadata: {}", meta.dimmed());
    }
    println!("  State updated:");
    println!("    Finance:  {}", hex::encode(&vector.finance[..8]).yellow());
    println!("    Time:     {}", hex::encode(&vector.time[..8]).yellow());
    println!("    Hardware: {}", hex::encode(&vector.hardware[..8]).yellow());
    
    Ok(())
}

fn format_amount(cents: u64) -> String {
    let dollars = cents / 100;
    let cents = cents % 100;
    format!("{}.{:02}", dollars, cents)
}
