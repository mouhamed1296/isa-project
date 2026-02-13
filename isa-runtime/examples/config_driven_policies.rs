//! Example demonstrating configurable policies from files and environment variables.
//!
//! This shows how to load dimension policies, constraints, and hierarchies
//! from configuration files in production environments.

use isa_runtime::{
    DimensionPolicy, PolicySet, RecoveryStrategy,
    DimensionConstraint, ConstraintSet, ConstraintType,
};
use std::env;

fn main() {
    println!("=== Configurable Policies Demo ===\n");
    
    // 1. Load from environment variables
    println!("1. Loading policies from environment variables");
    let policy_set = load_policies_from_env();
    println!("   Loaded {} policies from environment\n", policy_set.len());
    
    // 2. Load from TOML-like structure (simulated)
    println!("2. Loading policies from configuration");
    let policy_set = load_policies_from_config();
    println!("   Loaded {} policies from config\n", policy_set.len());
    
    // 3. Show how to override defaults
    println!("3. Environment variable overrides");
    demonstrate_overrides();
    
    println!("\n=== Configuration Patterns ===\n");
    print_config_examples();
}

/// Load policies from environment variables
fn load_policies_from_env() -> PolicySet {
    let mut policies = PolicySet::new();
    
    // Example: ISA_DIM0_NAME, ISA_DIM0_THRESHOLD, ISA_DIM0_STRATEGY
    for i in 0..3 {
        let name = env::var(format!("ISA_DIM{}_NAME", i))
            .unwrap_or_else(|_| format!("Dimension {}", i));
        
        let threshold = env::var(format!("ISA_DIM{}_THRESHOLD", i))
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1000);
        
        let strategy = env::var(format!("ISA_DIM{}_STRATEGY", i))
            .ok()
            .and_then(|s| parse_strategy(&s))
            .unwrap_or(RecoveryStrategy::ImmediateHeal);
        
        let is_critical = env::var(format!("ISA_DIM{}_CRITICAL", i))
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        
        let weight = env::var(format!("ISA_DIM{}_WEIGHT", i))
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        
        let mut policy = DimensionPolicy::new(name)
            .with_threshold(threshold)
            .with_recovery(strategy)
            .with_weight(weight);
        
        if is_critical {
            policy = policy.critical();
        }
        
        policies.add_policy(policy);
        
        println!("   Loaded: dim{} threshold={} strategy={:?}", i, threshold, strategy);
    }
    
    policies
}

/// Parse recovery strategy from string
fn parse_strategy(s: &str) -> Option<RecoveryStrategy> {
    match s.to_lowercase().as_str() {
        "immediate" | "immediateHeal" => Some(RecoveryStrategy::ImmediateHeal),
        "monitor" | "monitoronly" => Some(RecoveryStrategy::MonitorOnly),
        "quarantine" => Some(RecoveryStrategy::Quarantine),
        "fullrecovery" => Some(RecoveryStrategy::FullRecovery),
        _ => None,
    }
}

/// Load policies from a configuration structure
/// In production, this would parse TOML/JSON/YAML
fn load_policies_from_config() -> PolicySet {
    // Simulated config structure
    let config = r#"
    [[dimensions]]
    name = "Financial Transactions"
    threshold = 1000
    strategy = "ImmediateHeal"
    critical = true
    weight = 1.0
    
    [[dimensions]]
    name = "Temporal Sequence"
    threshold = 5000
    strategy = "MonitorOnly"
    critical = false
    weight = 0.8
    
    [[dimensions]]
    name = "Hardware Events"
    threshold = 2000
    strategy = "Quarantine"
    critical = false
    weight = 0.9
    "#;
    
    println!("   Config preview:\n{}", config);
    
    // In production, use: toml::from_str(config) or serde_json::from_str(config)
    // For now, manually create the policies
    let mut policies = PolicySet::new();
    
    policies.add_policy(
        DimensionPolicy::new("Financial Transactions")
            .with_threshold(1000)
            .with_recovery(RecoveryStrategy::ImmediateHeal)
            .critical()
            .with_weight(1.0)
    );
    
    policies.add_policy(
        DimensionPolicy::new("Temporal Sequence")
            .with_threshold(5000)
            .with_recovery(RecoveryStrategy::MonitorOnly)
            .with_weight(0.8)
    );
    
    policies.add_policy(
        DimensionPolicy::new("Hardware Events")
            .with_threshold(2000)
            .with_recovery(RecoveryStrategy::Quarantine)
            .with_weight(0.9)
    );
    
    policies
}

/// Demonstrate environment variable overrides
fn demonstrate_overrides() {
    // Set some environment variables for demonstration
    env::set_var("ISA_DIM0_THRESHOLD", "2000");
    env::set_var("ISA_DIM0_STRATEGY", "quarantine");
    
    let threshold = env::var("ISA_DIM0_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1000);
    
    let strategy = env::var("ISA_DIM0_STRATEGY")
        .ok()
        .and_then(|s| parse_strategy(&s))
        .unwrap_or(RecoveryStrategy::ImmediateHeal);
    
    println!("   ISA_DIM0_THRESHOLD={} (overridden from default 1000)", threshold);
    println!("   ISA_DIM0_STRATEGY={:?} (overridden from default ImmediateHeal)", strategy);
}

/// Print example configuration files
fn print_config_examples() {
    println!("Example TOML configuration (policies.toml):");
    println!("```toml");
    println!(r#"
[global]
learning_rate = 0.1
min_observations = 10

[[dimensions]]
index = 0
name = "Financial Transactions"
threshold = 1000
strategy = "ImmediateHeal"
critical = true
weight = 1.0
enabled = true

[[dimensions]]
index = 1
name = "Temporal Sequence"
threshold = 5000
strategy = "MonitorOnly"
critical = false
weight = 0.8
enabled = true

[[constraints]]
name = "Financial-Temporal Ratio"
dimensions = [0, 1]
type = "MaxRatio"
ratio = 2
severity = 8

[[constraints]]
name = "Total Divergence Limit"
dimensions = [0, 1, 2]
type = "SumBelow"
threshold = 1000
severity = 10
"#);
    println!("```\n");
    
    println!("Example JSON configuration (policies.json):");
    println!("```json");
    println!(r#"
{{
  "global": {{
    "learning_rate": 0.1,
    "min_observations": 10
  }},
  "dimensions": [
    {{
      "index": 0,
      "name": "Financial Transactions",
      "threshold": 1000,
      "strategy": "ImmediateHeal",
      "critical": true,
      "weight": 1.0,
      "enabled": true
    }},
    {{
      "index": 1,
      "name": "Temporal Sequence",
      "threshold": 5000,
      "strategy": "MonitorOnly",
      "critical": false,
      "weight": 0.8,
      "enabled": true
    }}
  ],
  "constraints": [
    {{
      "name": "Financial-Temporal Ratio",
      "dimensions": [0, 1],
      "type": "MaxRatio",
      "ratio": 2,
      "severity": 8
    }}
  ]
}}
"#);
    println!("```\n");
    
    println!("Environment variables:");
    println!("```bash");
    println!("# Dimension 0 configuration");
    println!("export ISA_DIM0_NAME=\"Financial Transactions\"");
    println!("export ISA_DIM0_THRESHOLD=1000");
    println!("export ISA_DIM0_STRATEGY=ImmediateHeal");
    println!("export ISA_DIM0_CRITICAL=true");
    println!("export ISA_DIM0_WEIGHT=1.0");
    println!();
    println!("# Dimension 1 configuration");
    println!("export ISA_DIM1_NAME=\"Temporal Sequence\"");
    println!("export ISA_DIM1_THRESHOLD=5000");
    println!("export ISA_DIM1_STRATEGY=MonitorOnly");
    println!("export ISA_DIM1_CRITICAL=false");
    println!("export ISA_DIM1_WEIGHT=0.8");
    println!();
    println!("# Global settings");
    println!("export ISA_LEARNING_RATE=0.1");
    println!("export ISA_MIN_OBSERVATIONS=10");
    println!("```");
}
