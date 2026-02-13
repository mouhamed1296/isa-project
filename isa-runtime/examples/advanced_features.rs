//! Comprehensive example demonstrating all advanced MA-ISA features:
//! 1. Dynamic dimensions
//! 2. Dimension policies
//! 3. Cross-dimension constraints
//! 4. Dimension hierarchies
//! 5. Adaptive profiles

use isa_core::DynamicIntegrityState;
use isa_runtime::{
    DimensionPolicy, PolicySet, RecoveryStrategy,
    DimensionConstraint, ConstraintSet, ConstraintType,
    DimensionNode, DimensionHierarchy,
    AdaptiveProfile, DimensionObservation,
};

fn main() {
    println!("=== MA-ISA Advanced Features Demo ===\n");
    
    // 1. DYNAMIC DIMENSIONS
    println!("1. Dynamic Dimensions (Runtime-configurable dimension count)");
    println!("   Creating a state with 5 dimensions...");
    
    let master_seed = [42u8; 32];
    let mut dynamic_state = DynamicIntegrityState::new(5, master_seed);
    
    println!("   Initial dimension count: {}", dynamic_state.dimension_count());
    
    // Add a new dimension at runtime
    dynamic_state.add_dimension(master_seed);
    println!("   After adding dimension: {}", dynamic_state.dimension_count());
    
    // Remove a dimension
    dynamic_state.remove_dimension();
    println!("   After removing dimension: {}\n", dynamic_state.dimension_count());
    
    // 2. DIMENSION POLICIES
    println!("2. Dimension Policies (Per-dimension thresholds and recovery)");
    
    let mut policy_set = PolicySet::new();
    
    // Critical financial dimension with immediate healing
    policy_set.add_policy(
        DimensionPolicy::new("Financial Transactions")
            .with_threshold(1000)
            .with_recovery(RecoveryStrategy::ImmediateHeal)
            .critical()
            .with_weight(1.0)
    );
    
    // Time dimension with monitoring only
    policy_set.add_policy(
        DimensionPolicy::new("Temporal Sequence")
            .with_threshold(5000)
            .with_recovery(RecoveryStrategy::MonitorOnly)
            .with_weight(0.8)
    );
    
    // Hardware dimension with quarantine on breach
    policy_set.add_policy(
        DimensionPolicy::new("Hardware Events")
            .with_threshold(2000)
            .with_recovery(RecoveryStrategy::Quarantine)
            .with_weight(0.9)
    );
    
    println!("   Created {} dimension policies", policy_set.len());
    println!("   - Financial: threshold=1000, strategy=ImmediateHeal, critical=true");
    println!("   - Temporal: threshold=5000, strategy=MonitorOnly");
    println!("   - Hardware: threshold=2000, strategy=Quarantine\n");
    
    // Simulate divergence evaluation
    let divergences = vec![
        [150u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [200u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [250u8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    
    let violations = policy_set.evaluate(&divergences);
    println!("   Policy violations detected: {}", violations.len());
    for (idx, policy) in violations {
        println!("     - Dimension {}: {} (strategy: {:?})", 
            idx, policy.name, policy.recovery_strategy);
    }
    println!();
    
    // 3. CROSS-DIMENSION CONSTRAINTS
    println!("3. Cross-Dimension Constraints (Relationships between dimensions)");
    
    let mut constraint_set = ConstraintSet::new();
    
    // Financial divergence must not exceed 2x temporal divergence
    constraint_set.add_constraint(
        DimensionConstraint::new(
            "Financial-Temporal Ratio",
            vec![0, 1],
            ConstraintType::MaxRatio { ratio: 2 }
        ).with_severity(8)
    );
    
    // Sum of all divergences must stay below threshold
    constraint_set.add_constraint(
        DimensionConstraint::new(
            "Total Divergence Limit",
            vec![0, 1, 2],
            ConstraintType::SumBelow { threshold: 1000 }
        ).with_severity(10)
    );
    
    println!("   Created {} cross-dimension constraints", constraint_set.len());
    println!("   - Financial-Temporal Ratio: dim0 <= 2 * dim1");
    println!("   - Total Divergence Limit: sum(dim0,dim1,dim2) < 1000\n");
    
    let constraint_violations = constraint_set.evaluate(&divergences);
    println!("   Constraint violations: {}", constraint_violations.len());
    for (idx, constraint) in constraint_violations {
        println!("     - {}: severity={}", constraint.name, constraint.severity);
    }
    println!();
    
    // 4. DIMENSION HIERARCHIES
    println!("4. Dimension Hierarchies (Parent-child relationships)");
    
    let mut hierarchy = DimensionHierarchy::new();
    
    // Create a 3-level hierarchy:
    // Root: System Integrity
    //   ├─ Financial Subsystem (dim 0)
    //   │   ├─ Transactions (dim 3)
    //   │   └─ Balances (dim 4)
    //   └─ Operational Subsystem (dim 1)
    //       ├─ Time Sync (dim 5)
    //       └─ Hardware (dim 2)
    
    hierarchy.add_node(DimensionNode::new(0, "Financial Subsystem"));
    hierarchy.add_node(DimensionNode::new(1, "Operational Subsystem"));
    hierarchy.add_node(DimensionNode::new(2, "Hardware").with_parent(1).with_weight(0.6));
    hierarchy.add_node(DimensionNode::new(3, "Transactions").with_parent(0).with_weight(0.7));
    hierarchy.add_node(DimensionNode::new(4, "Balances").with_parent(0).with_weight(0.3));
    hierarchy.add_node(DimensionNode::new(5, "Time Sync").with_parent(1).with_weight(0.4));
    
    println!("   Created hierarchy with {} nodes", hierarchy.len());
    println!("   Root nodes: {}", hierarchy.get_roots().len());
    println!("   Leaf nodes: {}", hierarchy.get_leaves().len());
    
    // Show hierarchy structure
    for root in hierarchy.get_roots() {
        println!("\n   {} (dim {})", root.name, root.dimension_index);
        for child in hierarchy.get_children(root.dimension_index) {
            println!("     ├─ {} (dim {}, weight={})", 
                child.name, child.dimension_index, child.weight);
            for grandchild in hierarchy.get_children(child.dimension_index) {
                println!("     │  └─ {} (dim {}, weight={})", 
                    grandchild.name, grandchild.dimension_index, grandchild.weight);
            }
        }
    }
    println!();
    
    // 5. ADAPTIVE PROFILES
    println!("\n5. Adaptive Profiles (ML-driven dimension importance)");
    
    let mut adaptive_profile = AdaptiveProfile::new("Production System", 3);
    adaptive_profile.learning_rate = 0.15;
    adaptive_profile.min_observations = 5;
    
    println!("   Created adaptive profile: {}", adaptive_profile.name);
    println!("   Learning rate: {}", adaptive_profile.learning_rate);
    println!("   Min observations: {}\n", adaptive_profile.min_observations);
    
    // Simulate observations over time
    println!("   Simulating 10 observations...");
    
    for i in 0..10 {
        // Dimension 0: High divergence, frequent recoveries (critical)
        adaptive_profile.record_observation(DimensionObservation {
            timestamp: 1000 + i * 100,
            dimension_index: 0,
            divergence: [200u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            event_count: 50,
            recovery_triggered: i % 2 == 0,
        });
        
        // Dimension 1: Medium divergence, occasional recoveries
        adaptive_profile.record_observation(DimensionObservation {
            timestamp: 1000 + i * 100,
            dimension_index: 1,
            divergence: [100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            event_count: 30,
            recovery_triggered: i % 4 == 0,
        });
        
        // Dimension 2: Low divergence, rare recoveries (stable)
        adaptive_profile.record_observation(DimensionObservation {
            timestamp: 1000 + i * 100,
            dimension_index: 2,
            divergence: [20u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            event_count: 10,
            recovery_triggered: i == 9,
        });
    }
    
    println!("\n   Learned dimension statistics:");
    for i in 0..3 {
        if let Some(stats) = adaptive_profile.get_stats(i) {
            println!("     Dimension {}: importance={:.3}, mean_div={}, recoveries={}/{}", 
                i, stats.importance, stats.mean_divergence, 
                stats.recovery_count, stats.observation_count);
        }
    }
    
    let weights = adaptive_profile.get_recommended_weights();
    println!("\n   Recommended weights (normalized):");
    for (i, weight) in weights.iter().enumerate() {
        println!("     Dimension {}: {:.3}", i, weight);
    }
    
    println!("\n=== Demo Complete ===");
    println!("\nKey Takeaways:");
    println!("✓ Dynamic dimensions allow runtime flexibility");
    println!("✓ Policies enable per-dimension recovery strategies");
    println!("✓ Constraints enforce relationships between dimensions");
    println!("✓ Hierarchies organize dimensions into logical structures");
    println!("✓ Adaptive profiles learn optimal weights from observations");
}
