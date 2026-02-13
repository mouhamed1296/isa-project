use anyhow::{Context, Result};
use colored::*;
use isa_runtime::{DeviceRuntime, FilePersistence};

pub fn run(device_id: String, output: Option<String>, seed_hex: Option<String>) -> Result<()> {
    let output_path = output.unwrap_or_else(|| format!("{}.state", device_id));
    
    // Generate or parse seed
    let seed = if let Some(hex) = seed_hex {
        parse_hex_seed(&hex)?
    } else {
        generate_random_seed()?
    };
    
    // Create runtime with persistence
    let persistence = FilePersistence::new(&output_path);
    let runtime = DeviceRuntime::load_or_create(seed, persistence)
        .context("Failed to create device runtime")?;
    
    // Save initial state
    runtime.save()
        .context("Failed to save initial state")?;
    
    println!("{} Initialized device: {}", "✓".green().bold(), device_id.cyan());
    println!("  State file: {}", output_path.yellow());
    println!("  Seed: {}", hex::encode(seed).dimmed());
    println!();
    println!("{} Keep the seed secure! It cannot be recovered if lost.", "⚠".yellow().bold());
    
    Ok(())
}

fn parse_hex_seed(hex: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex)
        .context("Invalid hex string for seed")?;
    
    if bytes.len() != 32 {
        anyhow::bail!("Seed must be exactly 32 bytes (64 hex characters), got {}", bytes.len());
    }
    
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Ok(seed)
}

fn generate_random_seed() -> Result<[u8; 32]> {
    use std::fs::File;
    use std::io::Read;
    
    let mut seed = [0u8; 32];
    
    // Try to read from /dev/urandom (Unix) or use getrandom
    #[cfg(unix)]
    {
        let mut f = File::open("/dev/urandom")
            .context("Failed to open /dev/urandom")?;
        f.read_exact(&mut seed)
            .context("Failed to read random bytes")?;
    }
    
    #[cfg(not(unix))]
    {
        // For Windows, use a simple approach
        // In production, you'd want a proper RNG crate
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for (i, chunk) in seed.chunks_mut(16).enumerate() {
            let val = nanos.wrapping_add(i as u128);
            chunk.copy_from_slice(&val.to_le_bytes()[..chunk.len()]);
        }
    }
    
    Ok(seed)
}
