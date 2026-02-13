//! Adaptive profiles with machine learning-driven dimension importance.
//!
//! ## Conformance Classification
//!
//! **⚠️ EXPERIMENTAL / NON-NORMATIVE** - This module is provided for research and
//! exploratory purposes only. It is NOT required, recommended, or suitable for
//! regulatory or safety-critical deployments.
//!
//! This module defines experimental mechanisms for:
//! - Adaptive parameter estimation based on historical observations
//! - Machine learning model integration
//! - Automatic dimension weight optimization
//!
//! ## Safety Warning
//!
//! Adaptive mechanisms introduce non-determinism and SHALL NOT be used in
//! safety-critical, regulatory compliance, or formally verified systems.
//! For production use, dimension weights and policies SHALL be statically
//! configured and validated.
//!
//! This module provides a framework for dynamically adjusting dimension weights
//! and policies based on observed patterns and ML models.

use isa_core::STATE_SIZE;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Historical observation of dimension behavior.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionObservation {
    /// Timestamp of this observation (milliseconds since epoch).
    pub timestamp: u64,
    
    /// Dimension index.
    pub dimension_index: usize,
    
    /// Divergence value at this time.
    pub divergence: [u8; STATE_SIZE],
    
    /// Event count since last observation.
    pub event_count: u64,
    
    /// Whether recovery was triggered.
    pub recovery_triggered: bool,
}

/// Statistics for a dimension over time.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionStats {
    /// Dimension index.
    pub dimension_index: usize,
    
    /// Mean divergence value (first 8 bytes as u64).
    pub mean_divergence: u64,
    
    /// Standard deviation of divergence.
    pub std_deviation: u64,
    
    /// Maximum observed divergence.
    pub max_divergence: u64,
    
    /// Number of recovery events.
    pub recovery_count: u64,
    
    /// Total number of observations.
    pub observation_count: u64,
    
    /// Calculated importance score (0.0 to 1.0).
    pub importance: f32,
}

impl DimensionStats {
    /// Create new statistics for a dimension.
    pub fn new(dimension_index: usize) -> Self {
        Self {
            dimension_index,
            mean_divergence: 0,
            std_deviation: 0,
            max_divergence: 0,
            recovery_count: 0,
            observation_count: 0,
            importance: 0.5, // Start with neutral importance
        }
    }
    
    /// Update statistics with a new observation.
    pub fn update(&mut self, observation: &DimensionObservation) {
        let div_value = u64::from_le_bytes([
            observation.divergence[0], observation.divergence[1],
            observation.divergence[2], observation.divergence[3],
            observation.divergence[4], observation.divergence[5],
            observation.divergence[6], observation.divergence[7],
        ]);
        
        // Update max
        self.max_divergence = self.max_divergence.max(div_value);
        
        // Update mean (running average)
        let n = self.observation_count as f64;
        let new_mean = ((self.mean_divergence as f64) * n + (div_value as f64)) / (n + 1.0);
        self.mean_divergence = new_mean as u64;
        
        // Update recovery count
        if observation.recovery_triggered {
            self.recovery_count += 1;
        }
        
        self.observation_count += 1;
        
        // Recalculate importance
        self.calculate_importance();
    }
    
    /// Calculate importance score based on statistics.
    ///
    /// Factors:
    /// - Higher mean divergence = more important
    /// - More recovery events = more important
    /// - Higher variance = more important (less stable)
    fn calculate_importance(&mut self) {
        if self.observation_count == 0 {
            self.importance = 0.5;
            return;
        }
        
        // Normalize factors to 0-1 range
        let divergence_factor = (self.mean_divergence as f64 / u64::MAX as f64).min(1.0);
        let recovery_factor = (self.recovery_count as f64 / self.observation_count as f64).min(1.0);
        let variance_factor = (self.std_deviation as f64 / u64::MAX as f64).min(1.0);
        
        // Weighted combination
        let importance = (divergence_factor * 0.4) + (recovery_factor * 0.4) + (variance_factor * 0.2);
        
        self.importance = importance.clamp(0.0, 1.0) as f32;
    }
}

/// Adaptive profile that learns from observations.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdaptiveProfile {
    /// Name of this profile.
    pub name: String,
    
    /// Statistics for each dimension.
    pub dimension_stats: Vec<DimensionStats>,
    
    /// Learning rate (how quickly to adapt, 0.0 to 1.0).
    pub learning_rate: f32,
    
    /// Minimum observations before adapting.
    pub min_observations: u64,
    
    /// Whether adaptation is currently enabled.
    pub enabled: bool,
}

impl AdaptiveProfile {
    /// Create a new adaptive profile.
    pub fn new(name: impl Into<String>, dimension_count: usize) -> Self {
        let dimension_stats = (0..dimension_count)
            .map(|i| DimensionStats::new(i))
            .collect();
        
        Self {
            name: name.into(),
            dimension_stats,
            learning_rate: 0.1,
            min_observations: 10,
            enabled: true,
        }
    }
    
    /// Record an observation for a dimension.
    pub fn record_observation(&mut self, observation: DimensionObservation) {
        if !self.enabled {
            return;
        }
        
        if let Some(stats) = self.dimension_stats.get_mut(observation.dimension_index) {
            stats.update(&observation);
        }
    }
    
    /// Get the current importance score for a dimension.
    pub fn get_importance(&self, dimension_index: usize) -> Option<f32> {
        self.dimension_stats.get(dimension_index).map(|s| s.importance)
    }
    
    /// Get recommended weights for all dimensions.
    ///
    /// Returns normalized weights that sum to 1.0.
    pub fn get_recommended_weights(&self) -> Vec<f32> {
        if self.dimension_stats.is_empty() {
            return Vec::new();
        }
        
        // Check if we have enough observations
        let ready = self.dimension_stats.iter()
            .all(|s| s.observation_count >= self.min_observations);
        
        if !ready {
            // Return equal weights if not enough data
            let equal_weight = 1.0 / self.dimension_stats.len() as f32;
            return vec![equal_weight; self.dimension_stats.len()];
        }
        
        // Calculate total importance
        let total_importance: f32 = self.dimension_stats.iter()
            .map(|s| s.importance)
            .sum();
        
        if total_importance == 0.0 {
            let equal_weight = 1.0 / self.dimension_stats.len() as f32;
            return vec![equal_weight; self.dimension_stats.len()];
        }
        
        // Normalize to sum to 1.0
        self.dimension_stats.iter()
            .map(|s| s.importance / total_importance)
            .collect()
    }
    
    /// Get statistics for a specific dimension.
    pub fn get_stats(&self, dimension_index: usize) -> Option<&DimensionStats> {
        self.dimension_stats.get(dimension_index)
    }
    
    /// Reset all statistics.
    pub fn reset(&mut self) {
        for stats in &mut self.dimension_stats {
            *stats = DimensionStats::new(stats.dimension_index);
        }
    }
    
    /// Get the number of dimensions being tracked.
    pub fn dimension_count(&self) -> usize {
        self.dimension_stats.len()
    }
}

/// ML model interface for advanced adaptive behavior.
///
/// This trait allows plugging in external ML models for prediction and optimization.
pub trait MLModel: Send + Sync {
    /// Predict the expected divergence for a dimension.
    fn predict_divergence(&self, dimension_index: usize, context: &ModelContext) -> Option<u64>;
    
    /// Recommend optimal weights based on current state.
    fn recommend_weights(&self, context: &ModelContext) -> Vec<f32>;
    
    /// Train the model with new observations.
    fn train(&mut self, observations: &[DimensionObservation]);
    
    /// Get model metadata.
    fn metadata(&self) -> ModelMetadata;
}

/// Context provided to ML models for predictions.
#[derive(Debug, Clone)]
pub struct ModelContext {
    /// Current dimension statistics.
    pub dimension_stats: Vec<DimensionStats>,
    
    /// Recent observations (limited window).
    pub recent_observations: Vec<DimensionObservation>,
    
    /// Current timestamp.
    pub timestamp: u64,
    
    /// Additional context data.
    pub metadata: Vec<(String, String)>,
}

/// Metadata about an ML model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModelMetadata {
    /// Model name.
    pub name: String,
    
    /// Model version.
    pub version: String,
    
    /// Model type (e.g., "linear_regression", "neural_network").
    pub model_type: String,
    
    /// Training data size.
    pub training_samples: u64,
    
    /// Model accuracy/confidence (0.0 to 1.0).
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_profile_creation() {
        let profile = AdaptiveProfile::new("test", 3);
        assert_eq!(profile.dimension_count(), 3);
        assert_eq!(profile.get_recommended_weights(), vec![1.0/3.0, 1.0/3.0, 1.0/3.0]);
    }
    
    #[test]
    fn test_observation_recording() {
        let mut profile = AdaptiveProfile::new("test", 2);
        profile.min_observations = 1;
        
        let obs = DimensionObservation {
            timestamp: 1000,
            dimension_index: 0,
            divergence: [100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            event_count: 10,
            recovery_triggered: true,
        };
        
        profile.record_observation(obs);
        
        let stats = profile.get_stats(0).unwrap();
        assert_eq!(stats.observation_count, 1);
        assert_eq!(stats.recovery_count, 1);
        assert!(stats.importance > 0.0);
    }
    
    #[test]
    fn test_importance_calculation() {
        let mut profile = AdaptiveProfile::new("test", 3);
        profile.min_observations = 2;
        
        // Dimension 0: high divergence, many recoveries
        for i in 0..5 {
            profile.record_observation(DimensionObservation {
                timestamp: 1000 + i * 100,
                dimension_index: 0,
                divergence: [200u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                event_count: 10,
                recovery_triggered: true,
            });
        }
        
        // Dimension 1: low divergence, no recoveries
        for i in 0..5 {
            profile.record_observation(DimensionObservation {
                timestamp: 1000 + i * 100,
                dimension_index: 1,
                divergence: [10u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                event_count: 10,
                recovery_triggered: false,
            });
        }
        
        let importance0 = profile.get_importance(0).unwrap();
        let importance1 = profile.get_importance(1).unwrap();
        
        // Dimension 0 should be more important
        assert!(importance0 > importance1);
    }
}
