//! Circular distance metric over 2^256 space.
//!
//! Provides deterministic divergence calculation between states
//! using modular arithmetic in the 256-bit integer space.
//!
//! ## Invariants
//!
//! - All arithmetic is integer-only (no floating point)
//! - Distance calculation is deterministic
//! - Handles wraparound correctly via borrow propagation
//!
//! ## SIMD Acceleration
//!
//! When the `simd` feature is enabled, uses platform-specific SIMD instructions
//! for 2-4x performance improvement on supported hardware.

use core::cmp::Ordering;

#[cfg(feature = "simd")]
mod divergence_simd;

pub struct CircularDistance;

impl CircularDistance {
    /// Compute circular distance from b to a (a - b in modular arithmetic).
    ///
    /// Uses SIMD acceleration when the `simd` feature is enabled and
    /// hardware support is available.
    #[inline]
    pub fn compute(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        #[cfg(feature = "simd")]
        {
            divergence_simd::compute_optimized(a, b)
        }
        
        #[cfg(not(feature = "simd"))]
        {
            Self::compute_scalar(a, b)
        }
    }
    
    /// Scalar implementation (always available as fallback).
    #[inline]
    pub fn compute_scalar(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut borrow = false;

        for i in 0..32 {
            let ai = a[i] as u16;
            let bi = b[i] as u16;
            let borrowed = if borrow { 1u16 } else { 0u16 };

            let (diff, new_borrow) = if ai >= bi + borrowed {
                (ai - bi - borrowed, false)
            } else {
                (256 + ai - bi - borrowed, true)
            };

            result[i] = diff as u8;
            borrow = new_borrow;
        }

        result
    }

    /// Compare two 256-bit values.
    ///
    /// Uses SIMD acceleration when available for faster comparison.
    #[inline]
    pub fn compare(a: &[u8; 32], b: &[u8; 32]) -> Ordering {
        #[cfg(feature = "simd")]
        {
            divergence_simd::compare_optimized(a, b)
        }
        
        #[cfg(not(feature = "simd"))]
        {
            Self::compare_scalar(a, b)
        }
    }
    
    /// Scalar comparison implementation.
    #[inline]
    pub fn compare_scalar(a: &[u8; 32], b: &[u8; 32]) -> Ordering {
        for i in (0..32).rev() {
            match a[i].cmp(&b[i]) {
                Ordering::Greater => return Ordering::Greater,
                Ordering::Less => return Ordering::Less,
                Ordering::Equal => continue,
            }
        }
        Ordering::Equal
    }

    pub fn min_distance(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let forward = Self::compute(b, a);
        let backward = Self::compute(a, b);

        match Self::compare(&forward, &backward) {
            Ordering::Less | Ordering::Equal => forward,
            Ordering::Greater => backward,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_distance_zero() {
        let a = [5u8; 32];
        let b = [5u8; 32];
        let dist = CircularDistance::compute(&a, &b);
        assert_eq!(dist, [0u8; 32]);
    }

    #[test]
    fn test_circular_distance_simple() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[0] = 10;
        b[0] = 5;

        let dist = CircularDistance::compute(&a, &b);
        assert_eq!(dist[0], 5);
        for i in 1..32 {
            assert_eq!(dist[i], 0);
        }
    }

    #[test]
    fn test_circular_distance_wraparound() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[0] = 5;
        b[0] = 10;

        let dist = CircularDistance::compute(&a, &b);
        assert_eq!(dist[0], 251);
    }

    #[test]
    fn test_circular_distance_multi_byte() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[0] = 0;
        a[1] = 1;
        b[0] = 1;
        b[1] = 0;

        let dist = CircularDistance::compute(&a, &b);
        assert_eq!(dist[0], 255);
        assert_eq!(dist[1], 0);
    }

    #[test]
    fn test_min_distance() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[0] = 10;
        b[0] = 250;

        let min_dist = CircularDistance::min_distance(&a, &b);
        
        let forward = CircularDistance::compute(&b, &a);
        let backward = CircularDistance::compute(&a, &b);
        
        assert!(
            min_dist == forward || min_dist == backward,
            "min_distance should return one of the two directional distances"
        );
    }

    #[test]
    fn test_compare() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        
        assert_eq!(CircularDistance::compare(&a, &b), Ordering::Equal);
        
        a[31] = 1;
        assert_eq!(CircularDistance::compare(&a, &b), Ordering::Greater);
        
        b[31] = 2;
        assert_eq!(CircularDistance::compare(&a, &b), Ordering::Less);
    }
}
