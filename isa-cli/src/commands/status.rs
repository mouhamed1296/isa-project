use anyhow::Result;
use colored::*;
use std::path::Path;

pub fn run(path: Option<String>) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| ".".to_string());

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Project Status".white().bold());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    // Check for config file
    let config_path = Path::new(&project_dir).join("isa.config.json");
    if config_path.exists() {
        println!("  {} {} found", "✓".green().bold(), "isa.config.json".cyan());

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = config.get("name").and_then(|v| v.as_str()) {
                    println!("    {} {}", "Project:".dimmed(), name.white());
                }
                if let Some(target) = config.get("target").and_then(|v| v.as_str()) {
                    println!("    {} {}", "Target:".dimmed(), target.cyan());
                }
                if let Some(dims) = config.get("dimensions").and_then(|v| v.as_array()) {
                    println!("    {} {} dimension(s)", "Dimensions:".dimmed(), dims.len().to_string().yellow());
                }
                if let Some(constraints) = config.get("constraints").and_then(|v| v.as_array()) {
                    println!("    {} {} constraint(s)", "Constraints:".dimmed(), constraints.len().to_string().yellow());
                }
            }
        }
    } else {
        println!("  {} {} not found", "✗".red(), "isa.config.json".dimmed());
        println!("    Run {} to create one", "isa init".cyan());
    }

    println!();

    // Check for Cargo.toml with isa dependencies
    let cargo_path = Path::new(&project_dir).join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if content.contains("isa-core") || content.contains("isa-ffi") {
                println!("  {} {} with ISA dependencies", "✓".green().bold(), "Cargo.toml".cyan());
            } else {
                println!("  {} {} found (no ISA deps)", "~".yellow(), "Cargo.toml".cyan());
            }
        }
    }

    // Check for package.json with isa dependencies
    let pkg_path = Path::new(&project_dir).join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if content.contains("isa-project") || content.contains("isa-ffi") {
                println!("  {} {} with ISA dependencies", "✓".green().bold(), "package.json".cyan());
            } else {
                println!("  {} {} found (no ISA deps)", "~".yellow(), "package.json".cyan());
            }
        }
    }

    // Check for WASM build
    let pkg_dir = Path::new(&project_dir).join("isa-ffi/pkg");
    let web_pkg = Path::new(&project_dir).join("web-demo/pkg");
    if pkg_dir.exists() || web_pkg.exists() {
        println!("  {} WASM package built", "✓".green().bold());
    }

    // Check for .state files
    let state_files: Vec<_> = std::fs::read_dir(&project_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "state"))
        .collect();

    if !state_files.is_empty() {
        println!("  {} {} device state file(s)", "✓".green().bold(), state_files.len().to_string().yellow());
        for f in &state_files {
            println!("    {} {}", "•".dimmed(), f.file_name().to_string_lossy().dimmed());
        }
    }

    println!();

    // Health summary
    let has_config = config_path.exists();
    let has_states = !state_files.is_empty();

    if has_config && has_states {
        println!("  {} Project is fully configured and active", "●".green().bold());
    } else if has_config {
        println!("  {} Project is configured, no device states yet", "●".yellow().bold());
        println!("    Run {} to create a device", "isa device init <name>".cyan());
    } else {
        println!("  {} Project not initialized", "●".red().bold());
        println!("    Run {} to get started", "isa init".cyan());
    }

    println!();
    Ok(())
}
