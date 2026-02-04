//! EPU dirty-state caching and active environment collection.
//!
//! This module provides cache tracking for environment configurations to skip
//! rebuilding unchanged static environments, and utilities for collecting and
//! deduplicating active environment IDs.

use super::settings::MAX_ACTIVE_ENVS;
use super::settings::MAX_ENV_STATES;
use super::EpuConfig;

/// Cache entry for dirty-state tracking of environment configurations.
///
/// Each entry stores the hash for an environment slot, allowing the runtime to
/// skip rebuilding unchanged environments.
#[derive(Clone, Copy, Default)]
pub(super) struct EpuCacheEntry {
    /// Hash of the EpuConfig used to detect changes
    pub state_hash: u64,
    /// Whether this cache entry contains valid data
    pub valid: bool,
}

/// Cache storage for dirty-state tracking.
pub(super) struct EpuCache {
    entries: Vec<EpuCacheEntry>,
}

impl EpuCache {
    pub fn new() -> Self {
        Self {
            entries: vec![EpuCacheEntry::default(); MAX_ENV_STATES as usize],
        }
    }

    pub fn invalidate(&mut self, env_id: u32) {
        if let Some(entry) = self.entries.get_mut(env_id as usize) {
            entry.valid = false;
        }
    }

    pub fn invalidate_all(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.valid = false;
        }
    }

    /// Check if an environment needs rebuilding and update cache.
    ///
    /// Returns `true` if the environment needs to be rebuilt.
    pub fn needs_rebuild(&mut self, env_id: u32, config: &EpuConfig) -> bool {
        let hash = config.state_hash();

        if let Some(entry) = self.entries.get_mut(env_id as usize) {
            // Check if we can skip this environment
            if entry.valid && entry.state_hash == hash {
                // Cache hit: same config
                return false;
            }

            // Cache miss: update hash and rebuild
            entry.state_hash = hash;
            entry.valid = true;
        }

        true
    }
}

/// Result of collecting active environments with deduplication and capping.
#[derive(Debug, Clone)]
pub struct ActiveEnvList {
    /// Deduplicated and capped list of unique environment IDs.
    pub unique_ids: Vec<u32>,
    /// Maps original env_id to its slot index in `unique_ids`, or 0 for fallback.
    pub slot_map: std::collections::HashMap<u32, u32>,
    /// Number of environments that were dropped due to cap overflow.
    pub overflow_count: usize,
}

/// Collects unique environment IDs, caps to MAX_ACTIVE_ENVS, logs warning in debug builds if overflow.
///
/// Returns an `ActiveEnvList` containing:
/// - `unique_ids`: The deduplicated and capped list of environment IDs
/// - `slot_map`: Maps each env_id to its slot index (0-31), or 0 for envs that exceeded the cap
/// - `overflow_count`: Number of environments that were dropped due to exceeding the cap
///
/// # Arguments
/// * `env_ids` - Slice of environment IDs (may contain duplicates)
///
/// # Example
/// ```ignore
/// let env_ids = &[5, 2, 5, 10, 2, 7];
/// let result = collect_active_envs(env_ids);
/// // result.unique_ids = [2, 5, 7, 10] (sorted, deduplicated)
/// // result.slot_map = {2: 0, 5: 1, 7: 2, 10: 3}
/// ```
pub fn collect_active_envs(env_ids: &[u32]) -> ActiveEnvList {
    // Deduplicate
    let mut unique: Vec<u32> = env_ids.to_vec();
    unique.sort_unstable();
    unique.dedup();

    // Track overflow before capping
    let overflow_count = unique.len().saturating_sub(MAX_ACTIVE_ENVS as usize);

    // Cap and log warning in debug builds
    if unique.len() > MAX_ACTIVE_ENVS as usize {
        #[cfg(debug_assertions)]
        eprintln!(
            "EPU: {} unique envs exceed cap of {}, falling back to env_id=0 for {} envs",
            unique.len(),
            MAX_ACTIVE_ENVS,
            overflow_count
        );
        unique.truncate(MAX_ACTIVE_ENVS as usize);
    }

    // Build mapping: env_id -> slot index
    let mut slot_map = std::collections::HashMap::new();
    for (slot, &env_id) in unique.iter().enumerate() {
        slot_map.insert(env_id, slot as u32);
    }
    // Note: Any env_id not in slot_map should use slot 0 as fallback.
    // The caller can check with: slot_map.get(&env_id).copied().unwrap_or(0)

    ActiveEnvList {
        unique_ids: unique,
        slot_map,
        overflow_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_default() {
        let entry = EpuCacheEntry::default();
        assert_eq!(entry.state_hash, 0);
        assert!(!entry.valid);
    }

    #[test]
    fn test_epu_cache_needs_rebuild_first_call() {
        let mut cache = EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };

        // First call should always need rebuild (cache not valid)
        assert!(cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_hit_static_config() {
        let mut cache = EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ], // Static config
        };

        // First call: cache miss
        assert!(cache.needs_rebuild(0, &config));

        // Second call with same config: cache hit
        assert!(!cache.needs_rebuild(0, &config));

        // Third call: still a hit
        assert!(!cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_miss_different_config() {
        let mut cache = EpuCache::new();
        let config1 = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };
        let config2 = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 17],
            ], // Different
        };

        // First config
        assert!(cache.needs_rebuild(0, &config1));
        assert!(!cache.needs_rebuild(0, &config1));

        // Different config: cache miss
        assert!(cache.needs_rebuild(0, &config2));

        // Same different config: cache hit
        assert!(!cache.needs_rebuild(0, &config2));
    }

    #[test]
    fn test_epu_cache_invalidate_single() {
        let mut cache = EpuCache::new();
        let config = EpuConfig {
            layers: [
                [1, 2],
                [3, 4],
                [5, 6],
                [7, 8],
                [9, 10],
                [11, 12],
                [13, 14],
                [15, 16],
            ],
        };

        // Populate cache
        assert!(cache.needs_rebuild(0, &config));
        assert!(!cache.needs_rebuild(0, &config)); // Hit

        // Invalidate
        cache.invalidate(0);

        // Should need rebuild again
        assert!(cache.needs_rebuild(0, &config));
    }

    #[test]
    fn test_epu_cache_invalidate_all() {
        let mut cache = EpuCache::new();
        let config1 = EpuConfig {
            layers: [
                [1, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        let config2 = EpuConfig {
            layers: [
                [2, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        // Populate cache for multiple envs
        assert!(cache.needs_rebuild(0, &config1));
        assert!(cache.needs_rebuild(1, &config2));
        assert!(!cache.needs_rebuild(0, &config1)); // Hit
        assert!(!cache.needs_rebuild(1, &config2)); // Hit

        // Invalidate all
        cache.invalidate_all();

        // Both should need rebuild
        assert!(cache.needs_rebuild(0, &config1));
        assert!(cache.needs_rebuild(1, &config2));
    }

    #[test]
    fn test_epu_cache_multiple_env_ids() {
        let mut cache = EpuCache::new();
        let config_a = EpuConfig {
            layers: [
                [0xA, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        let config_b = EpuConfig {
            layers: [
                [0xB, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };

        // Different env IDs should have independent cache entries
        assert!(cache.needs_rebuild(10, &config_a));
        assert!(cache.needs_rebuild(20, &config_b));

        // Each should be cached independently
        assert!(!cache.needs_rebuild(10, &config_a));
        assert!(!cache.needs_rebuild(20, &config_b));

        // Changing one doesn't affect the other
        let config_a_modified = EpuConfig {
            layers: [
                [0xAA, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
                [0, 0],
            ],
        };
        assert!(cache.needs_rebuild(10, &config_a_modified));
        assert!(!cache.needs_rebuild(20, &config_b)); // Still cached
    }

    #[test]
    fn test_collect_active_envs_deduplication() {
        // Input with duplicates
        let env_ids = &[5, 2, 5, 10, 2, 7, 10, 5];
        let result = collect_active_envs(env_ids);

        // Should be sorted and deduplicated
        assert_eq!(result.unique_ids, vec![2, 5, 7, 10]);
        assert_eq!(result.overflow_count, 0);

        // Check slot mapping
        assert_eq!(result.slot_map.get(&2), Some(&0));
        assert_eq!(result.slot_map.get(&5), Some(&1));
        assert_eq!(result.slot_map.get(&7), Some(&2));
        assert_eq!(result.slot_map.get(&10), Some(&3));
    }

    #[test]
    fn test_collect_active_envs_capping() {
        // Create more than MAX_ACTIVE_ENVS unique IDs (0..40)
        let env_ids: Vec<u32> = (0..40).collect();
        let result = collect_active_envs(&env_ids);

        // Should be capped to MAX_ACTIVE_ENVS
        assert_eq!(result.unique_ids.len(), MAX_ACTIVE_ENVS as usize);
        assert_eq!(result.overflow_count, 40 - MAX_ACTIVE_ENVS as usize);

        // IDs should be sorted, so 0..31 should be kept
        for i in 0..MAX_ACTIVE_ENVS {
            assert!(result.unique_ids.contains(&i));
            assert_eq!(result.slot_map.get(&i), Some(&i));
        }

        // IDs 32..39 should NOT be in the mapping
        for i in MAX_ACTIVE_ENVS..40 {
            assert!(!result.slot_map.contains_key(&i));
        }
    }

    #[test]
    fn test_collect_active_envs_fallback_mapping() {
        // Simple case with a few IDs
        let env_ids = &[100, 50, 25];
        let result = collect_active_envs(env_ids);

        // Sorted order: 25, 50, 100
        assert_eq!(result.unique_ids, vec![25, 50, 100]);

        // Verify slot mapping
        assert_eq!(result.slot_map.get(&25), Some(&0));
        assert_eq!(result.slot_map.get(&50), Some(&1));
        assert_eq!(result.slot_map.get(&100), Some(&2));

        // Unknown ID should return None (caller uses unwrap_or(0) for fallback)
        assert_eq!(result.slot_map.get(&999), None);
        assert_eq!(result.slot_map.get(&999).copied().unwrap_or(0), 0);
    }

    #[test]
    fn test_collect_active_envs_empty() {
        let result = collect_active_envs(&[]);
        assert!(result.unique_ids.is_empty());
        assert!(result.slot_map.is_empty());
        assert_eq!(result.overflow_count, 0);
    }

    #[test]
    fn test_collect_active_envs_single() {
        let result = collect_active_envs(&[42]);
        assert_eq!(result.unique_ids, vec![42]);
        assert_eq!(result.slot_map.get(&42), Some(&0));
        assert_eq!(result.overflow_count, 0);
    }

    #[test]
    fn test_collect_active_envs_exactly_at_cap() {
        // Exactly MAX_ACTIVE_ENVS unique IDs
        let env_ids: Vec<u32> = (0..MAX_ACTIVE_ENVS).collect();
        let result = collect_active_envs(&env_ids);

        assert_eq!(result.unique_ids.len(), MAX_ACTIVE_ENVS as usize);
        assert_eq!(result.overflow_count, 0);

        // All IDs should be mapped
        for i in 0..MAX_ACTIVE_ENVS {
            assert_eq!(result.slot_map.get(&i), Some(&i));
        }
    }
}
