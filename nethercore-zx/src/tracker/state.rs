//! Tracker rollback state caching
//!
//! Row state cache for fast rollback reconstruction, using BTreeMap for
//! efficient range queries.

use std::collections::BTreeMap;

use super::MAX_TRACKER_CHANNELS;
use super::channels::TrackerChannel;

/// Row state cache for fast rollback reconstruction
///
/// Uses BTreeMap for O(log n) range queries instead of O(n) linear search.
#[derive(Debug)]
pub struct RowStateCache {
    /// Cached channel states: (order, row) -> channels (sorted by key)
    cache: BTreeMap<(u32, u16, u16), CachedRowState>,
    /// Maximum cache entries
    max_entries: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedControlState {
    pub channel_mutes: [bool; MAX_TRACKER_CHANNELS],
    pub global_volume: f32,
    pub pattern_delay: u8,
    pub pattern_delay_count: u8,
    pub fine_pattern_delay: u16,
    pub xm_next_pattern_row: u16,
    pub xm_loop_owner: Option<usize>,
    pub last_global_vol_slide: u8,
    pub is_it_format: bool,
    pub old_effects_mode: bool,
    pub link_g_memory: bool,
    pub tempo_slide: i8,
}

#[derive(Debug, Clone)]
pub struct CachedRowState {
    pub channels: Box<[TrackerChannel; MAX_TRACKER_CHANNELS]>,
    pub control: CachedControlState,
}

impl Default for RowStateCache {
    fn default() -> Self {
        Self {
            cache: BTreeMap::new(),
            max_entries: 256, // ~256 * 32 channels * ~200 bytes = ~1.6MB max
        }
    }
}

impl RowStateCache {
    /// Check if we should cache this row (every 4 rows or pattern boundary)
    pub fn should_cache(row: u16) -> bool {
        row.is_multiple_of(4)
    }

    /// Find nearest cached state before or at target position (O(log n) with BTreeMap)
    pub fn find_nearest(
        &self,
        handle: u32,
        target_order: u16,
        target_row: u16,
    ) -> Option<((u16, u16), &CachedRowState)> {
        // Use range query to find the greatest key <= (target_order, target_row)
        self.cache
            .range((handle, 0, 0)..=(handle, target_order, target_row))
            .next_back()
            .map(|(pos, state)| ((pos.1, pos.2), state))
    }

    /// Store state at row
    pub fn store(
        &mut self,
        handle: u32,
        order: u16,
        row: u16,
        channels: &[TrackerChannel; MAX_TRACKER_CHANNELS],
        control: CachedControlState,
    ) {
        // Evict oldest entry if at capacity (BTreeMap keeps entries sorted, so first is oldest by position)
        if self.cache.len() >= self.max_entries
            && let Some(&key) = self.cache.keys().next()
        {
            self.cache.remove(&key);
        }

        self.cache.insert(
            (handle, order, row),
            CachedRowState {
                channels: Box::new(channels.clone()),
                control,
            },
        );
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_cache() {
        assert!(RowStateCache::should_cache(0));
        assert!(!RowStateCache::should_cache(1));
        assert!(!RowStateCache::should_cache(2));
        assert!(!RowStateCache::should_cache(3));
        assert!(RowStateCache::should_cache(4));
        assert!(RowStateCache::should_cache(8));
    }

    #[test]
    fn test_find_nearest() {
        let mut cache = RowStateCache::default();
        let channels: [TrackerChannel; MAX_TRACKER_CHANNELS] =
            std::array::from_fn(|_| TrackerChannel::default());

        // Store some entries
        let control = CachedControlState {
            channel_mutes: [false; MAX_TRACKER_CHANNELS],
            global_volume: 1.0,
            pattern_delay: 0,
            pattern_delay_count: 0,
            fine_pattern_delay: 0,
            xm_next_pattern_row: 0,
            xm_loop_owner: None,
            last_global_vol_slide: 0,
            is_it_format: false,
            old_effects_mode: false,
            link_g_memory: false,
            tempo_slide: 0,
        };
        cache.store(1, 0, 0, &channels, control);
        cache.store(1, 0, 4, &channels, control);
        cache.store(1, 1, 0, &channels, control);

        // Find nearest to (0, 2) should return (0, 0)
        let result = cache.find_nearest(1, 0, 2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, (0, 0));

        // Find nearest to (0, 5) should return (0, 4)
        let result = cache.find_nearest(1, 0, 5);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, (0, 4));

        // Find nearest to (1, 2) should return (1, 0)
        let result = cache.find_nearest(1, 1, 2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, (1, 0));
    }
}
