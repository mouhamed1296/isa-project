//! Dimension hierarchies for organizing dimensions into parent-child relationships.
//!
//! ## Conformance Classification
//!
//! **OPTIONAL** - This module defines optional mechanisms for hierarchical dimension
//! organization and aggregation. Use of this module is NOT required for conformance
//! with the MA-ISA core specification.
//!
//! Implementations MAY organize dimensions hierarchically but SHALL NOT require
//! hierarchies for basic integrity monitoring functionality.
//!
//! This module allows dimensions to be organized in hierarchical structures,
//! where child dimensions inherit properties from parents and can be aggregated.

use isa_core::STATE_SIZE;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A node in the dimension hierarchy.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionNode {
    /// Index of this dimension in the state.
    pub dimension_index: usize,
    
    /// Name/label for this dimension.
    pub name: String,
    
    /// Parent dimension index (None for root dimensions).
    pub parent: Option<usize>,
    
    /// Child dimension indices.
    pub children: Vec<usize>,
    
    /// Aggregation weight (how much this dimension contributes to parent).
    pub weight: f32,
    
    /// Metadata for this dimension.
    pub metadata: DimensionMetadata,
}

/// Metadata for a dimension node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionMetadata {
    /// Description of what this dimension tracks.
    pub description: String,
    
    /// Tags for categorization.
    pub tags: Vec<String>,
    
    /// Whether this is a leaf node (no children).
    pub is_leaf: bool,
    
    /// Custom key-value properties.
    pub properties: Vec<(String, String)>,
}

impl DimensionNode {
    /// Create a new dimension node.
    pub fn new(dimension_index: usize, name: impl Into<String>) -> Self {
        Self {
            dimension_index,
            name: name.into(),
            parent: None,
            children: Vec::new(),
            weight: 1.0,
            metadata: DimensionMetadata {
                description: String::new(),
                tags: Vec::new(),
                is_leaf: true,
                properties: Vec::new(),
            },
        }
    }
    
    /// Set the parent of this node.
    pub fn with_parent(mut self, parent_index: usize) -> Self {
        self.parent = Some(parent_index);
        self
    }
    
    /// Set the weight of this node.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }
    
    /// Add a child to this node.
    pub fn add_child(&mut self, child_index: usize) {
        if !self.children.contains(&child_index) {
            self.children.push(child_index);
            self.metadata.is_leaf = false;
        }
    }
    
    /// Remove a child from this node.
    pub fn remove_child(&mut self, child_index: usize) {
        self.children.retain(|&idx| idx != child_index);
        self.metadata.is_leaf = self.children.is_empty();
    }
}

/// Hierarchical organization of dimensions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionHierarchy {
    nodes: Vec<DimensionNode>,
}

impl DimensionHierarchy {
    /// Create a new empty hierarchy.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
    
    /// Add a dimension node to the hierarchy.
    pub fn add_node(&mut self, node: DimensionNode) {
        // Update parent's children list if this node has a parent
        if let Some(parent_idx) = node.parent {
            if let Some(parent) = self.nodes.iter_mut().find(|n| n.dimension_index == parent_idx) {
                parent.add_child(node.dimension_index);
            }
        }
        
        self.nodes.push(node);
    }
    
    /// Get a node by dimension index.
    pub fn get_node(&self, dimension_index: usize) -> Option<&DimensionNode> {
        self.nodes.iter().find(|n| n.dimension_index == dimension_index)
    }
    
    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, dimension_index: usize) -> Option<&mut DimensionNode> {
        self.nodes.iter_mut().find(|n| n.dimension_index == dimension_index)
    }
    
    /// Get all root nodes (nodes with no parent).
    pub fn get_roots(&self) -> Vec<&DimensionNode> {
        self.nodes.iter().filter(|n| n.parent.is_none()).collect()
    }
    
    /// Get all leaf nodes (nodes with no children).
    pub fn get_leaves(&self) -> Vec<&DimensionNode> {
        self.nodes.iter().filter(|n| n.metadata.is_leaf).collect()
    }
    
    /// Get the children of a node.
    pub fn get_children(&self, dimension_index: usize) -> Vec<&DimensionNode> {
        if let Some(node) = self.get_node(dimension_index) {
            node.children.iter()
                .filter_map(|&idx| self.get_node(idx))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get the path from a node to the root.
    pub fn get_path_to_root(&self, dimension_index: usize) -> Vec<usize> {
        let mut path = vec![dimension_index];
        let mut current = dimension_index;
        
        while let Some(node) = self.get_node(current) {
            if let Some(parent_idx) = node.parent {
                path.push(parent_idx);
                current = parent_idx;
            } else {
                break;
            }
        }
        
        path
    }
    
    /// Calculate aggregated divergence for a parent based on its children.
    ///
    /// Uses weighted average of child divergences.
    pub fn aggregate_divergence(
        &self,
        parent_index: usize,
        divergences: &[[u8; STATE_SIZE]]
    ) -> Option<[u8; STATE_SIZE]> {
        let children = self.get_children(parent_index);
        if children.is_empty() {
            return None;
        }
        
        let mut weighted_sum = [0u64; 4]; // First 32 bytes as 4x u64
        let mut total_weight = 0.0f32;
        
        for child in children {
            if let Some(div) = divergences.get(child.dimension_index) {
                let weight = child.weight;
                total_weight += weight;
                
                // Convert first 32 bytes to u64 chunks and accumulate
                for i in 0..4 {
                    let offset = i * 8;
                    let value = u64::from_le_bytes([
                        div[offset], div[offset+1], div[offset+2], div[offset+3],
                        div[offset+4], div[offset+5], div[offset+6], div[offset+7],
                    ]);
                    weighted_sum[i] = weighted_sum[i].saturating_add(
                        ((value as f64) * (weight as f64)) as u64
                    );
                }
            }
        }
        
        if total_weight == 0.0 {
            return None;
        }
        
        // Convert back to bytes
        let mut result = [0u8; STATE_SIZE];
        for i in 0..4 {
            let avg = ((weighted_sum[i] as f64) / (total_weight as f64)) as u64;
            let bytes = avg.to_le_bytes();
            let offset = i * 8;
            result[offset..offset+8].copy_from_slice(&bytes);
        }
        
        Some(result)
    }
    
    /// Get the depth of a node in the hierarchy.
    pub fn get_depth(&self, dimension_index: usize) -> usize {
        self.get_path_to_root(dimension_index).len() - 1
    }
    
    /// Get the number of nodes in the hierarchy.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if the hierarchy is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for DimensionHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hierarchy_creation() {
        let mut hierarchy = DimensionHierarchy::new();
        
        // Create root node
        let root = DimensionNode::new(0, "root");
        hierarchy.add_node(root);
        
        // Create child nodes
        let child1 = DimensionNode::new(1, "child1").with_parent(0);
        let child2 = DimensionNode::new(2, "child2").with_parent(0);
        
        hierarchy.add_node(child1);
        hierarchy.add_node(child2);
        
        assert_eq!(hierarchy.len(), 3);
        assert_eq!(hierarchy.get_roots().len(), 1);
        assert_eq!(hierarchy.get_children(0).len(), 2);
    }
    
    #[test]
    fn test_path_to_root() {
        let mut hierarchy = DimensionHierarchy::new();
        
        hierarchy.add_node(DimensionNode::new(0, "root"));
        hierarchy.add_node(DimensionNode::new(1, "level1").with_parent(0));
        hierarchy.add_node(DimensionNode::new(2, "level2").with_parent(1));
        
        let path = hierarchy.get_path_to_root(2);
        assert_eq!(path, vec![2, 1, 0]);
        
        assert_eq!(hierarchy.get_depth(0), 0);
        assert_eq!(hierarchy.get_depth(1), 1);
        assert_eq!(hierarchy.get_depth(2), 2);
    }
    
    #[test]
    fn test_aggregate_divergence() {
        let mut hierarchy = DimensionHierarchy::new();
        
        hierarchy.add_node(DimensionNode::new(0, "parent"));
        hierarchy.add_node(DimensionNode::new(1, "child1").with_parent(0).with_weight(0.5));
        hierarchy.add_node(DimensionNode::new(2, "child2").with_parent(0).with_weight(0.5));
        
        let divergences = vec![
            [0u8; 32], // parent (will be calculated)
            [100u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [200u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];
        
        let agg = hierarchy.aggregate_divergence(0, &divergences);
        assert!(agg.is_some());
        
        // Weighted average: (100 * 0.5 + 200 * 0.5) / 1.0 = 150
        let result = agg.unwrap();
        assert_eq!(result[0], 150);
    }
}
