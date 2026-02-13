use anyhow::Result;
use colored::*;
use dialoguer::{Select, Input, Confirm, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::time::Duration;

use super::config::{IsaConfig, DimensionConfig, PolicyConfig, ConstraintConfig};

const PRESETS: &[(&str, &str)] = &[
    ("POS / Retail", "Point-of-sale with transaction, inventory, payment, and user integrity"),
    ("IoT / Embedded", "IoT devices with sensor, network, firmware, and battery integrity"),
    ("Government", "Secure communications, inter-agency transfers, classified docs, infrastructure"),
    ("Mobile Money", "Cross-border payments, fraud detection, DeFi integration, micropayments"),
    ("Custom", "Define your own dimensions and policies from scratch"),
];

pub fn run(path: Option<String>, defaults: bool) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| ".".to_string());
    let config_path = Path::new(&project_dir).join("isa.config.json");

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Project Setup".white().bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();

    if defaults {
        return create_default_config(&config_path);
    }

    // Step 1: Choose project type
    println!("  {} {}", "1/5".dimmed(), "Choose your project type:".yellow().bold());
    println!();

    let preset_labels: Vec<String> = PRESETS.iter()
        .map(|(name, desc)| format!("{} — {}", name, desc))
        .collect();

    let preset_idx = Select::with_theme(&ColorfulTheme::default())
        .items(&preset_labels)
        .default(0)
        .interact()?;

    let preset_name = PRESETS[preset_idx].0;
    println!();

    // Step 2: Project name
    println!("  {} {}", "2/5".dimmed(), "Project name:".yellow().bold());
    let project_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("  Name")
        .default("my-isa-project".to_string())
        .interact_text()?;
    println!();

    // Step 3: Choose target platform
    println!("  {} {}", "3/5".dimmed(), "Target platform:".yellow().bold());
    let targets = vec![
        "NestJS (TypeScript/WASM)",
        "Tauri (Rust + React)",
        "Rust (native library)",
        "WASM (browser)",
    ];
    let target_idx = Select::with_theme(&ColorfulTheme::default())
        .items(&targets)
        .default(0)
        .interact()?;
    println!();

    // Step 4: Dimensions
    let dimensions = if preset_idx == 4 {
        // Custom: ask for dimensions
        println!("  {} {}", "4/5".dimmed(), "Define your dimensions:".yellow().bold());
        collect_custom_dimensions()?
    } else {
        println!("  {} {}", "4/5".dimmed(), "Using preset dimensions:".yellow().bold());
        let dims = get_preset_dimensions(preset_idx);
        for (i, d) in dims.iter().enumerate() {
            println!("    {} {} (threshold: {}, strategy: {})",
                format!("{}.", i).dimmed(),
                d.name.cyan(),
                d.threshold.to_string().yellow(),
                d.strategy.green()
            );
        }
        println!();
        dims
    };

    // Step 5: Confirm
    println!("  {} {}", "5/5".dimmed(), "Review:".yellow().bold());
    println!("    {} {}", "Project:".dimmed(), project_name.white());
    println!("    {} {}", "Type:".dimmed(), preset_name.white());
    println!("    {} {}", "Target:".dimmed(), targets[target_idx].white());
    println!("    {} {}", "Dimensions:".dimmed(), dimensions.len().to_string().white());
    println!();

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("  Create project configuration?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("\n  {} Setup cancelled.", "✗".red());
        return Ok(());
    }

    // Generate config
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("  {spinner:.cyan} [{bar:30.cyan/dim}] {msg}")
        .unwrap()
        .progress_chars("█▓░"));

    pb.set_message("Creating configuration...");
    let target_str = match target_idx {
        0 => "nestjs",
        1 => "tauri",
        2 => "rust",
        3 => "wasm",
        _ => "nestjs",
    };

    let config = IsaConfig {
        name: project_name.clone(),
        version: "0.1.0".to_string(),
        target: target_str.to_string(),
        preset: preset_name.to_string(),
        dimensions: dimensions.clone(),
        constraints: get_default_constraints(&dimensions),
    };
    pb.inc(1);
    std::thread::sleep(Duration::from_millis(200));

    pb.set_message("Writing isa.config.json...");
    let json = serde_json::to_string_pretty(&config)?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config_path, &json)?;
    pb.inc(1);
    std::thread::sleep(Duration::from_millis(200));

    pb.set_message("Generating install instructions...");
    pb.inc(1);
    std::thread::sleep(Duration::from_millis(200));

    pb.set_message("Done!");
    pb.inc(1);
    pb.finish_and_clear();

    println!();
    println!("  {} Project initialized!", "✓".green().bold());
    println!();
    println!("  {} {}", "Config:".dimmed(), config_path.display().to_string().yellow());
    println!();
    println!("  {}", "Next steps:".yellow().bold());

    match target_idx {
        0 => {
            println!("    {} Install the WASM package:", "1.".dimmed());
            println!("       {}", "npm install github:mouhamed1296/isa-project#main --save".cyan());
            println!("    {} Import in your NestJS service:", "2.".dimmed());
            println!("       {}", "const { WasmAxisAccumulator } = require('isa-project');".cyan());
            println!("    {} Run `isa install --target nestjs` for full setup guide", "3.".dimmed());
        }
        1 => {
            println!("    {} Add to src-tauri/Cargo.toml:", "1.".dimmed());
            println!("       {}", "isa-core = { git = \"https://github.com/mouhamed1296/isa-project.git\", path = \"isa-core\" }".cyan());
            println!("    {} Run `isa install --target tauri` for full setup guide", "2.".dimmed());
        }
        2 => {
            println!("    {} Add to Cargo.toml:", "1.".dimmed());
            println!("       {}", "isa-core = { git = \"https://github.com/mouhamed1296/isa-project.git\", path = \"isa-core\" }".cyan());
            println!("    {} Run `isa install --target rust` for full setup guide", "2.".dimmed());
        }
        3 => {
            println!("    {} Install the WASM package:", "1.".dimmed());
            println!("       {}", "npm install github:mouhamed1296/isa-project#main --save".cyan());
            println!("    {} Run `isa install --target wasm` for full setup guide", "2.".dimmed());
        }
        _ => {}
    }

    println!();
    Ok(())
}

fn create_default_config(config_path: &Path) -> Result<()> {
    let config = IsaConfig {
        name: "my-isa-project".to_string(),
        version: "0.1.0".to_string(),
        target: "nestjs".to_string(),
        preset: "POS / Retail".to_string(),
        dimensions: get_preset_dimensions(0),
        constraints: vec![
            ConstraintConfig {
                constraint_type: "SumBelow".to_string(),
                dimensions: vec![0, 1, 2, 3],
                value: 2000,
            },
        ],
    };

    let json = serde_json::to_string_pretty(&config)?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, &json)?;

    println!("  {} Default config created at {}", "✓".green().bold(), config_path.display().to_string().yellow());
    Ok(())
}

fn collect_custom_dimensions() -> Result<Vec<DimensionConfig>> {
    let mut dimensions = Vec::new();

    loop {
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("    Dimension name")
            .interact_text()?;

        let threshold: u64 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("    Divergence threshold")
            .default(500)
            .interact_text()?;

        let strategies = vec!["ImmediateHeal", "Quarantine", "MonitorOnly"];
        let strategy_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("    Recovery strategy")
            .items(&strategies)
            .default(0)
            .interact()?;

        dimensions.push(DimensionConfig {
            name,
            threshold,
            strategy: strategies[strategy_idx].to_string(),
            policy: PolicyConfig {
                max_events_per_minute: 100,
                require_entropy: true,
            },
        });

        println!();
        let add_more = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("    Add another dimension?")
            .default(dimensions.len() < 3)
            .interact()?;

        if !add_more {
            break;
        }
    }

    println!();
    Ok(dimensions)
}

fn get_preset_dimensions(preset_idx: usize) -> Vec<DimensionConfig> {
    match preset_idx {
        0 => vec![
            DimensionConfig {
                name: "Transaction Integrity".to_string(),
                threshold: 200,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 200, require_entropy: true },
            },
            DimensionConfig {
                name: "Inventory Integrity".to_string(),
                threshold: 300,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 100, require_entropy: true },
            },
            DimensionConfig {
                name: "Payment Integrity".to_string(),
                threshold: 150,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 150, require_entropy: true },
            },
            DimensionConfig {
                name: "User Action Integrity".to_string(),
                threshold: 400,
                strategy: "MonitorOnly".to_string(),
                policy: PolicyConfig { max_events_per_minute: 50, require_entropy: false },
            },
        ],
        1 => vec![
            DimensionConfig {
                name: "Sensor Integrity".to_string(),
                threshold: 500,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 300, require_entropy: true },
            },
            DimensionConfig {
                name: "Network Integrity".to_string(),
                threshold: 800,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 100, require_entropy: true },
            },
            DimensionConfig {
                name: "Firmware Integrity".to_string(),
                threshold: 200,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 10, require_entropy: true },
            },
            DimensionConfig {
                name: "Battery Integrity".to_string(),
                threshold: 1000,
                strategy: "MonitorOnly".to_string(),
                policy: PolicyConfig { max_events_per_minute: 60, require_entropy: false },
            },
        ],
        2 => vec![
            DimensionConfig {
                name: "Secure Communications".to_string(),
                threshold: 100,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 500, require_entropy: true },
            },
            DimensionConfig {
                name: "Inter-Agency Transfer".to_string(),
                threshold: 50,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 50, require_entropy: true },
            },
            DimensionConfig {
                name: "Classified Documents".to_string(),
                threshold: 25,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 20, require_entropy: true },
            },
            DimensionConfig {
                name: "Infrastructure Monitoring".to_string(),
                threshold: 300,
                strategy: "MonitorOnly".to_string(),
                policy: PolicyConfig { max_events_per_minute: 1000, require_entropy: false },
            },
        ],
        3 => vec![
            DimensionConfig {
                name: "Cross-Border Payments".to_string(),
                threshold: 100,
                strategy: "ImmediateHeal".to_string(),
                policy: PolicyConfig { max_events_per_minute: 500, require_entropy: true },
            },
            DimensionConfig {
                name: "AI Fraud Detection".to_string(),
                threshold: 200,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 1000, require_entropy: true },
            },
            DimensionConfig {
                name: "DeFi Integration".to_string(),
                threshold: 150,
                strategy: "Quarantine".to_string(),
                policy: PolicyConfig { max_events_per_minute: 200, require_entropy: true },
            },
            DimensionConfig {
                name: "Micropayment Network".to_string(),
                threshold: 500,
                strategy: "MonitorOnly".to_string(),
                policy: PolicyConfig { max_events_per_minute: 5000, require_entropy: false },
            },
        ],
        _ => vec![],
    }
}

fn get_default_constraints(dimensions: &[DimensionConfig]) -> Vec<ConstraintConfig> {
    let dim_indices: Vec<usize> = (0..dimensions.len()).collect();
    vec![
        ConstraintConfig {
            constraint_type: "SumBelow".to_string(),
            dimensions: dim_indices,
            value: dimensions.iter().map(|d| d.threshold).sum::<u64>() * 2,
        },
    ]
}
