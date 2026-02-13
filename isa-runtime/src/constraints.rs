//! Cross-dimension constraints and relationships.
//!
//! ## Conformance Classification
//!
//! **OPTIONAL** - This module defines optional mechanisms for integrity aggregation
//! and system-level constraint evaluation. Use of this module is NOT required for
//! conformance with the MA-ISA core specification.
//!
//! Implementations MAY use these constraints but SHALL NOT require them for
//! basic integrity monitoring functionality.
//!
//! This module defines constraints that span multiple dimensions,
//! allowing you to express relationships like:
//! - "Dimension A must not diverge more than 2x dimension B"
//! - "If dimension A exceeds threshold, dimension B must be checked"
//! - "Dimensions A and B must maintain a specific ratio"

use isa_core::STATE_SIZE;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Type of relationship between dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ConstraintType {
    /// Dimension A divergence must be <= ratio * Dimension B divergence.
    MaxRatio { ratio: u32 },
    
    /// Sum of divergences must be below threshold.
    SumBelow { threshold: u64 },
    
    /// If dimension A exceeds threshold, dimension B must be checked.
    ConditionalCheck,
    
    /// Dimensions must maintain correlation (for ML/statistical analysis).
    Correlation { min_correlation: i32 }, // -100 to 100
    
    /// Custom constraint function (index into registry).
    Custom(u32),
}

/// A constraint between two or more dimensions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionConstraint {
    /// Name/description of this constraint.
    pub name: String,
    
    /// Dimensions involved in this constraint.
    pub dimensions: Vec<usize>,
    
    /// Type of constraint.
    pub constraint_type: ConstraintType,
    
    /// Whether this constraint is currently active.
    pub enabled: bool,
    
    /// Severity level (0-10, where 10 is critical).
    pub severity: u8,
}

impl DimensionConstraint {
    /// Create a new constraint.
    pub fn new(name: impl Into<String>, dimensions: Vec<usize>, constraint_type: ConstraintType) -> Self {
        Self {
            name: name.into(),
            dimensions,
            constraint_type,
            enabled: true,
            severity: 5,
        }
    }
    
    /// Set the severity level.
    pub fn with_severity(mut self, severity: u8) -> Self {
        self.severity = severity.min(10);
        self
    }
    
    /// Evaluate this constraint against divergence values.
    ///
    /// Returns true if the constraint is violated.
    pub fn evaluate(&self, divergences: &[[u8; STATE_SIZE]]) -> bool {
        if !self.enabled || self.dimensions.is_empty() {
            return false;
        }
        
        match self.constraint_type {
            ConstraintType::MaxRatio { ratio } => {
                if self.dimensions.len() < 2 {
                    return false;
                }
                
                let div_a = self.get_divergence_value(divergences, self.dimensions[0]);
                let div_b = self.get_divergence_value(divergences, self.dimensions[1]);
                
                if div_b == 0 {
                    return div_a > 0;
                }
                
                div_a > div_b.saturating_mul(ratio as u64)
            }
            
            ConstraintType::SumBelow { threshold } => {
                let sum: u64 = self.dimensions.iter()
                    .map(|&idx| self.get_divergence_value(divergences, idx))
                    .sum();
                
                sum > threshold
            }
            
            ConstraintType::ConditionalCheck => {
                // This requires external logic to implement
                false
            }
            
            ConstraintType::Correlation { .. } => {
                // Correlation analysis requires historical data
                false
            }
            
            ConstraintType::Custom(_) => {
                // Custom constraints require external implementation
                false
            }
        }
    }
    
    fn get_divergence_value(&self, divergences: &[[u8; STATE_SIZE]], index: usize) -> u64 {
        divergences.get(index).map(|div| {
            u64::from_le_bytes([
                div[0], div[1], div[2], div[3],
                div[4], div[5], div[6], div[7],
            ])
        }).unwrap_or(0)
    }
}

/// Set of constraints for an integrity state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConstraintSet {
    constraints: Vec<DimensionConstraint>,
}

impl ConstraintSet {
    /// Create a new empty constraint set.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }
    
    /// Add a constraint.
    pub fn add_constraint(&mut self, constraint: DimensionConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Get a constraint by index.
    pub fn get(&self, index: usize) -> Option<&DimensionConstraint> {
        self.constraints.get(index)
    }
    
    /// Evaluate all constraints against divergence values.
    ///
    /// Returns a vector of (constraint_index, constraint) pairs for violated constraints.
    pub fn evaluate(&self, divergences: &[[u8; STATE_SIZE]]) -> Vec<(usize, &DimensionConstraint)> {
        let mut violations = Vec::new();
        
        for (i, constraint) in self.constraints.iter().enumerate() {
            if constraint.evaluate(divergences) {
                violations.push((i, constraint));
            }
        }
        
        violations
    }
    
    /// Get the number of constraints.
    pub fn len(&self) -> usize {
        self.constraints.len()
    }
    
    /// Check if the constraint set is empty.
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_max_ratio_constraint() {
        let constraint = DimensionConstraint::new(
            "ratio_check",
            vec![0, 1],
            ConstraintType::MaxRatio { ratio: 2 }
        );
        
        // Dimension 0 = 100, Dimension 1 = 60
        // 100 > 60 * 2 = 120? No, so not violated
        let divergences = vec![
            [100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [60u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        
        assert!(!constraint.evaluate(&divergences));
        
        // Dimension 0 = 100, Dimension 1 = 40
        // 100 > 40 * 2 = 80, so 100 > 80 = true
        let divergences2 = vec![
            [100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [30u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        
        assert!(constraint.evaluate(&divergences2));
    }
    
    #[test]
    fn test_sum_below_constraint() {
        let constraint = DimensionConstraint::new(
            "sum_check",
            vec![0, 1, 2],
            ConstraintType::SumBelow { threshold: 200 }
        );
        
        let divergences = vec![
            [50u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [60u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [70u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        
        // Sum = 180, threshold = 200, so not violated
        assert!(!constraint.evaluate(&divergences));
        
        let divergences2 = vec![
            [80u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [80u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [80u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        
        // Sum = 240, threshold = 200, so violated
        assert!(constraint.evaluate(&divergences2));
    }
}
