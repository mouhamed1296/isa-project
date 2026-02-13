use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Select, Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "isa.config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaConfig {
    pub name: String,
    pub version: String,
    pub target: String,
    pub preset: String,
    pub dimensions: Vec<DimensionConfig>,
    pub constraints: Vec<ConstraintConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionConfig {
    pub name: String,
    pub threshold: u64,
    pub strategy: String,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub max_events_per_minute: u64,
    pub require_entropy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub constraint_type: String,
    pub dimensions: Vec<usize>,
    pub value: u64,
}

fn load_config() -> Result<IsaConfig> {
    let path = Path::new(CONFIG_FILE);
    if !path.exists() {
        anyhow::bail!(
            "No {} found. Run {} to create one.",
            CONFIG_FILE.yellow(),
            "isa init".cyan()
        );
    }
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;
    let config: IsaConfig = serde_json::from_str(&content)
        .context("Failed to parse config file")?;
    Ok(config)
}

fn save_config(config: &IsaConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(CONFIG_FILE, json)?;
    Ok(())
}

pub fn add_dimension(name: Option<String>) -> Result<()> {
    let mut config = load_config()?;

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Add Dimension".white().bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();

    let dim_name = if let Some(n) = name {
        n
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("  Dimension name")
            .interact_text()?
    };

    let threshold: u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("  Divergence threshold")
        .default(500)
        .interact_text()?;

    let strategies = vec!["ImmediateHeal", "Quarantine", "MonitorOnly"];
    let strategy_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("  Recovery strategy")
        .items(&strategies)
        .default(0)
        .interact()?;

    let max_events: u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("  Max events per minute")
        .default(100)
        .interact_text()?;

    let dimension = DimensionConfig {
        name: dim_name.clone(),
        threshold,
        strategy: strategies[strategy_idx].to_string(),
        policy: PolicyConfig {
            max_events_per_minute: max_events,
            require_entropy: true,
        },
    };

    let idx = config.dimensions.len();
    config.dimensions.push(dimension);
    save_config(&config)?;

    println!();
    println!(
        "  {} Added dimension {} at index {}",
        "✓".green().bold(),
        dim_name.cyan(),
        idx.to_string().yellow()
    );
    println!();

    Ok(())
}

pub fn list_dimensions() -> Result<()> {
    let config = load_config()?;

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Dimensions".white().bold());
    println!("  {}", "─".repeat(50).dimmed());
    println!();

    if config.dimensions.is_empty() {
        println!("  {} No dimensions configured.", "!".yellow());
        println!("  Run {} to add one.", "isa config add-dimension".cyan());
        println!();
        return Ok(());
    }

    println!(
        "  {}  {:<25} {:<12} {:<15} {}",
        "#".dimmed(),
        "Name".white().bold(),
        "Threshold".white().bold(),
        "Strategy".white().bold(),
        "Rate Limit".white().bold()
    );
    println!("  {}", "─".repeat(70).dimmed());

    for (i, dim) in config.dimensions.iter().enumerate() {
        let strategy_colored = match dim.strategy.as_str() {
            "ImmediateHeal" => dim.strategy.green(),
            "Quarantine" => dim.strategy.yellow(),
            "MonitorOnly" => dim.strategy.blue(),
            _ => dim.strategy.normal(),
        };

        println!(
            "  {}  {:<25} {:<12} {:<15} {}/min",
            format!("{}", i).dimmed(),
            dim.name.cyan(),
            dim.threshold.to_string().yellow(),
            strategy_colored,
            dim.policy.max_events_per_minute.to_string().dimmed()
        );
    }

    println!();
    println!(
        "  {} {} dimension(s) configured",
        "ℹ".blue(),
        config.dimensions.len()
    );
    println!();

    Ok(())
}

pub fn set_policy(
    dimension: String,
    strategy: Option<String>,
    threshold: Option<u64>,
) -> Result<()> {
    let mut config = load_config()?;

    // Find dimension by name or index
    let dim_idx = if let Ok(idx) = dimension.parse::<usize>() {
        if idx >= config.dimensions.len() {
            anyhow::bail!("Dimension index {} out of range (0-{})", idx, config.dimensions.len() - 1);
        }
        idx
    } else {
        config.dimensions.iter().position(|d| d.name.to_lowercase() == dimension.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Dimension '{}' not found", dimension))?
    };

    let dim = &mut config.dimensions[dim_idx];

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Set Policy".white().bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();

    let new_strategy = if let Some(s) = strategy {
        s
    } else {
        let strategies = vec!["ImmediateHeal", "Quarantine", "MonitorOnly"];
        let current_idx = strategies.iter().position(|s| *s == dim.strategy).unwrap_or(0);
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("  Strategy for '{}'", dim.name))
            .items(&strategies)
            .default(current_idx)
            .interact()?;
        strategies[idx].to_string()
    };

    let new_threshold = if let Some(t) = threshold {
        t
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("  Divergence threshold")
            .default(dim.threshold)
            .interact_text()?
    };

    let old_strategy = dim.strategy.clone();
    let old_threshold = dim.threshold;
    let dim_name = dim.name.clone();
    dim.strategy = new_strategy.clone();
    dim.threshold = new_threshold;

    save_config(&config)?;

    println!();
    println!("  {} Updated '{}' policy:", "✓".green().bold(), dim_name.cyan());
    println!(
        "    Strategy:  {} → {}",
        old_strategy.dimmed(),
        new_strategy.green()
    );
    println!(
        "    Threshold: {} → {}",
        old_threshold.to_string().dimmed(),
        new_threshold.to_string().yellow()
    );
    println!();

    Ok(())
}

pub fn add_constraint(constraint_type: Option<String>) -> Result<()> {
    let mut config = load_config()?;

    if config.dimensions.is_empty() {
        anyhow::bail!("No dimensions configured. Add dimensions first with `isa config add-dimension`.");
    }

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Add Constraint".white().bold());
    println!("  {}", "─".repeat(40).dimmed());
    println!();

    let ct = if let Some(t) = constraint_type {
        t
    } else {
        let types = vec!["SumBelow", "MaxRatio"];
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("  Constraint type")
            .items(&types)
            .default(0)
            .interact()?;
        types[idx].to_string()
    };

    // Select dimensions
    println!();
    println!("  Select dimensions for this constraint:");
    let dim_names: Vec<String> = config.dimensions.iter()
        .enumerate()
        .map(|(i, d)| format!("{}: {}", i, d.name))
        .collect();

    let selected: Vec<usize> = dialoguer::MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&dim_names)
        .defaults(&vec![true; dim_names.len()])
        .interact()?;

    let value: u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("  {} value", ct))
        .default(1000)
        .interact_text()?;

    config.constraints.push(ConstraintConfig {
        constraint_type: ct.clone(),
        dimensions: selected.clone(),
        value,
    });

    save_config(&config)?;

    println!();
    println!(
        "  {} Added {} constraint across {} dimensions (value: {})",
        "✓".green().bold(),
        ct.cyan(),
        selected.len().to_string().yellow(),
        value.to_string().yellow()
    );
    println!();

    Ok(())
}

pub fn show() -> Result<()> {
    let config = load_config()?;

    println!();
    println!("  {} {}", "MA-ISA".bright_cyan().bold(), "Configuration".white().bold());
    println!("  {}", "─".repeat(50).dimmed());
    println!();
    println!("  {} {}", "Project:".dimmed(), config.name.white().bold());
    println!("  {} {}", "Version:".dimmed(), config.version.white());
    println!("  {} {}", "Target:".dimmed(), config.target.cyan());
    println!("  {} {}", "Preset:".dimmed(), config.preset.white());
    println!();

    // Dimensions
    println!("  {}", "Dimensions:".yellow().bold());
    for (i, dim) in config.dimensions.iter().enumerate() {
        let strategy_colored = match dim.strategy.as_str() {
            "ImmediateHeal" => dim.strategy.green(),
            "Quarantine" => dim.strategy.yellow(),
            "MonitorOnly" => dim.strategy.blue(),
            _ => dim.strategy.normal(),
        };
        println!(
            "    {} {} — threshold: {}, strategy: {}, rate: {}/min",
            format!("{}.", i).dimmed(),
            dim.name.cyan(),
            dim.threshold.to_string().yellow(),
            strategy_colored,
            dim.policy.max_events_per_minute.to_string().dimmed()
        );
    }

    // Constraints
    if !config.constraints.is_empty() {
        println!();
        println!("  {}", "Constraints:".yellow().bold());
        for (i, c) in config.constraints.iter().enumerate() {
            let dim_names: Vec<String> = c.dimensions.iter()
                .filter_map(|&idx| config.dimensions.get(idx).map(|d| d.name.clone()))
                .collect();
            println!(
                "    {} {} across [{}] = {}",
                format!("{}.", i).dimmed(),
                c.constraint_type.cyan(),
                dim_names.join(", ").dimmed(),
                c.value.to_string().yellow()
            );
        }
    }

    println!();
    Ok(())
}

pub fn generate(format: String, output: Option<String>) -> Result<()> {
    let config = load_config()?;

    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&config)?,
        "toml" => toml::to_string_pretty(&config)?,
        "yaml" | _ => {
            // Simple YAML generation
            let mut yaml = String::new();
            yaml.push_str(&format!("name: {}\n", config.name));
            yaml.push_str(&format!("version: {}\n", config.version));
            yaml.push_str(&format!("target: {}\n", config.target));
            yaml.push_str(&format!("preset: {}\n", config.preset));
            yaml.push_str("dimensions:\n");
            for dim in &config.dimensions {
                yaml.push_str(&format!("  - name: {}\n", dim.name));
                yaml.push_str(&format!("    threshold: {}\n", dim.threshold));
                yaml.push_str(&format!("    strategy: {}\n", dim.strategy));
                yaml.push_str("    policy:\n");
                yaml.push_str(&format!("      max_events_per_minute: {}\n", dim.policy.max_events_per_minute));
                yaml.push_str(&format!("      require_entropy: {}\n", dim.policy.require_entropy));
            }
            yaml.push_str("constraints:\n");
            for c in &config.constraints {
                yaml.push_str(&format!("  - type: {}\n", c.constraint_type));
                yaml.push_str(&format!("    dimensions: {:?}\n", c.dimensions));
                yaml.push_str(&format!("    value: {}\n", c.value));
            }
            yaml
        }
    };

    if let Some(out_path) = output {
        fs::write(&out_path, &content)?;
        println!(
            "\n  {} Config written to {}\n",
            "✓".green().bold(),
            out_path.yellow()
        );
    } else {
        println!();
        println!("{}", content);
    }

    Ok(())
}
