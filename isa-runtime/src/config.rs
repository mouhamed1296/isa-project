//! Configuration loading utilities for policies, constraints, and hierarchies.
//!
//! This module provides helpers to load MA-ISA configuration from various sources:
//! - TOML/JSON/YAML files (via serde)
//! - Environment variables
//! - Command-line arguments
//! - Remote configuration services

use crate::policy::{DimensionPolicy, PolicySet, RecoveryStrategy};
use crate::constraints::{DimensionConstraint, ConstraintSet, ConstraintType};
use crate::adaptive::AdaptiveProfile;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Complete MA-ISA configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IsaConfig {
    /// Global settings
    pub global: GlobalConfig,
    
    /// Dimension policies
    pub dimensions: Vec<DimensionConfig>,
    
    /// Cross-dimension constraints
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraints: Vec<ConstraintConfig>,
    
    /// Hierarchy configuration
    #[cfg_attr(feature = "serde", serde(default))]
    pub hierarchy: Option<HierarchyConfig>,
}

/// Global configuration settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GlobalConfig {
    /// Learning rate for adaptive profiles (0.0 to 1.0)
    #[cfg_attr(feature = "serde", serde(default = "default_learning_rate"))]
    pub learning_rate: f32,
    
    /// Minimum observations before adapting
    #[cfg_attr(feature = "serde", serde(default = "default_min_observations"))]
    pub min_observations: u64,
    
    /// Master seed for dimension initialization (hex string)
    #[cfg_attr(feature = "serde", serde(default))]
    pub master_seed: Option<String>,
}

fn default_learning_rate() -> f32 { 0.1 }
fn default_min_observations() -> u64 { 10 }

/// Configuration for a single dimension
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionConfig {
    /// Dimension index
    pub index: usize,
    
    /// Human-readable name
    pub name: String,
    
    /// Divergence threshold
    pub threshold: u64,
    
    /// Recovery strategy
    pub strategy: String,
    
    /// Whether this dimension is critical
    #[cfg_attr(feature = "serde", serde(default))]
    pub critical: bool,
    
    /// Weight/importance (0.0 to 1.0)
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub weight: f32,
    
    /// Whether this dimension is enabled
    #[cfg_attr(feature = "serde", serde(default = "default_enabled"))]
    pub enabled: bool,
}

fn default_weight() -> f32 { 1.0 }
fn default_enabled() -> bool { true }

/// Configuration for a constraint
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConstraintConfig {
    /// Constraint name
    pub name: String,
    
    /// Dimensions involved
    pub dimensions: Vec<usize>,
    
    /// Constraint type
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub constraint_type: String,
    
    /// Ratio (for MaxRatio constraints)
    #[cfg_attr(feature = "serde", serde(default))]
    pub ratio: Option<u32>,
    
    /// Threshold (for SumBelow constraints)
    #[cfg_attr(feature = "serde", serde(default))]
    pub threshold: Option<u64>,
    
    /// Severity level (0-10)
    #[cfg_attr(feature = "serde", serde(default = "default_severity"))]
    pub severity: u8,
}

fn default_severity() -> u8 { 5 }

/// Hierarchy configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HierarchyConfig {
    /// Hierarchy nodes
    pub nodes: Vec<HierarchyNodeConfig>,
}

/// Configuration for a hierarchy node
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HierarchyNodeConfig {
    /// Dimension index
    pub dimension_index: usize,
    
    /// Node name
    pub name: String,
    
    /// Parent dimension index (None for root)
    #[cfg_attr(feature = "serde", serde(default))]
    pub parent: Option<usize>,
    
    /// Weight in parent aggregation
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub weight: f32,
}

impl IsaConfig {
    /// Convert this configuration into a PolicySet
    pub fn to_policy_set(&self) -> PolicySet {
        let mut policies = PolicySet::new();
        
        for dim_config in &self.dimensions {
            let strategy = parse_recovery_strategy(&dim_config.strategy);
            
            let mut policy = DimensionPolicy::new(&dim_config.name)
                .with_threshold(dim_config.threshold)
                .with_recovery(strategy)
                .with_weight(dim_config.weight);
            
            if dim_config.critical {
                policy = policy.critical();
            }
            
            if !dim_config.enabled {
                policy.enabled = false;
            }
            
            policies.add_policy(policy);
        }
        
        policies
    }
    
    /// Convert this configuration into a ConstraintSet
    pub fn to_constraint_set(&self) -> ConstraintSet {
        let mut constraints = ConstraintSet::new();
        
        for constraint_config in &self.constraints {
            let constraint_type = parse_constraint_type(
                &constraint_config.constraint_type,
                constraint_config.ratio,
                constraint_config.threshold,
            );
            
            let constraint = DimensionConstraint::new(
                &constraint_config.name,
                constraint_config.dimensions.clone(),
                constraint_type,
            ).with_severity(constraint_config.severity);
            
            constraints.add_constraint(constraint);
        }
        
        constraints
    }
    
    /// Create an AdaptiveProfile from this configuration
    pub fn to_adaptive_profile(&self, name: &str) -> AdaptiveProfile {
        let dimension_count = self.dimensions.len();
        let mut profile = AdaptiveProfile::new(name, dimension_count);
        
        profile.learning_rate = self.global.learning_rate;
        profile.min_observations = self.global.min_observations;
        
        profile
    }
}

/// Parse recovery strategy from string
fn parse_recovery_strategy(s: &str) -> RecoveryStrategy {
    match s.to_lowercase().as_str() {
        "immediateHeal" | "immediate" => RecoveryStrategy::ImmediateHeal,
        "monitoronly" | "monitor" => RecoveryStrategy::MonitorOnly,
        "quarantine" => RecoveryStrategy::Quarantine,
        "fullrecovery" | "full" => RecoveryStrategy::FullRecovery,
        _ => {
            // Try to parse as Custom(u32)
            if let Some(num_str) = s.strip_prefix("custom:") {
                if let Ok(num) = num_str.parse::<u32>() {
                    return RecoveryStrategy::Custom(num);
                }
            }
            RecoveryStrategy::ImmediateHeal // default
        }
    }
}

/// Parse constraint type from string
fn parse_constraint_type(
    type_str: &str,
    ratio: Option<u32>,
    threshold: Option<u64>,
) -> ConstraintType {
    match type_str.to_lowercase().as_str() {
        "maxratio" => ConstraintType::MaxRatio {
            ratio: ratio.unwrap_or(2),
        },
        "sumbelow" => ConstraintType::SumBelow {
            threshold: threshold.unwrap_or(1000),
        },
        "conditionalcheck" => ConstraintType::ConditionalCheck,
        "correlation" => ConstraintType::Correlation {
            min_correlation: 50, // default
        },
        _ => {
            if let Some(num_str) = type_str.strip_prefix("custom:") {
                if let Ok(num) = num_str.parse::<u32>() {
                    return ConstraintType::Custom(num);
                }
            }
            ConstraintType::SumBelow { threshold: 1000 } // default
        }
    }
}

/// Load configuration from environment variables
pub fn load_from_env(dimension_count: usize) -> IsaConfig {
    use std::env;
    
    let learning_rate = env::var("ISA_LEARNING_RATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.1);
    
    let min_observations = env::var("ISA_MIN_OBSERVATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    let master_seed = env::var("ISA_MASTER_SEED").ok();
    
    let mut dimensions = Vec::new();
    for i in 0..dimension_count {
        let name = env::var(format!("ISA_DIM{}_NAME", i))
            .unwrap_or_else(|_| format!("Dimension {}", i));
        
        let threshold = env::var(format!("ISA_DIM{}_THRESHOLD", i))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);
        
        let strategy = env::var(format!("ISA_DIM{}_STRATEGY", i))
            .unwrap_or_else(|_| "ImmediateHeal".to_string());
        
        let critical = env::var(format!("ISA_DIM{}_CRITICAL", i))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);
        
        let weight = env::var(format!("ISA_DIM{}_WEIGHT", i))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.0);
        
        dimensions.push(DimensionConfig {
            index: i,
            name,
            threshold,
            strategy,
            critical,
            weight,
            enabled: true,
        });
    }
    
    IsaConfig {
        global: GlobalConfig {
            learning_rate,
            min_observations,
            master_seed,
        },
        dimensions,
        constraints: Vec::new(),
        hierarchy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_recovery_strategy() {
        assert!(matches!(
            parse_recovery_strategy("ImmediateHeal"),
            RecoveryStrategy::ImmediateHeal
        ));
        assert!(matches!(
            parse_recovery_strategy("monitor"),
            RecoveryStrategy::MonitorOnly
        ));
        assert!(matches!(
            parse_recovery_strategy("quarantine"),
            RecoveryStrategy::Quarantine
        ));
    }
    
    #[test]
    fn test_config_to_policy_set() {
        let config = IsaConfig {
            global: GlobalConfig {
                learning_rate: 0.1,
                min_observations: 10,
                master_seed: None,
            },
            dimensions: vec![
                DimensionConfig {
                    index: 0,
                    name: "Test Dimension".to_string(),
                    threshold: 1000,
                    strategy: "ImmediateHeal".to_string(),
                    critical: true,
                    weight: 1.0,
                    enabled: true,
                },
            ],
            constraints: Vec::new(),
            hierarchy: None,
        };
        
        let policy_set = config.to_policy_set();
        assert_eq!(policy_set.len(), 1);
        
        let policy = policy_set.get(0).unwrap();
        assert_eq!(policy.name, "Test Dimension");
        assert_eq!(policy.max_divergence, 1000);
        assert!(policy.is_critical);
    }
}
