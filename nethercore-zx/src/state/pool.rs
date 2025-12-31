//! Generic state pool with deduplication
//!
//! Provides a reusable pattern for managing pools of unique states with:
//! - O(1) deduplication via HashMap lookup
//! - Type-safe index wrappers via PoolIndex trait
//! - Overflow protection with configurable max capacity
//! - Per-frame clear semantics

use hashbrown::HashMap;
use std::hash::Hash;

/// Trait for type-safe pool indices
///
/// Implement this for newtype wrappers around u32 to enable type-safe
/// pool access. The trait provides conversion to/from raw u32.
pub trait PoolIndex: Copy + Clone + PartialEq + Eq + Hash {
    /// Create index from raw u32
    fn from_raw(value: u32) -> Self;

    /// Get raw u32 value
    fn as_raw(&self) -> u32;
}

/// Generic state pool with deduplication
///
/// Stores unique values of type T, returning type-safe indices I.
/// Uses HashMap for O(1) deduplication on add.
///
/// # Type Parameters
///
/// - `T`: The state type to store (must be Eq + Hash + Clone)
/// - `I`: The index type (must implement PoolIndex)
///
/// # Example
///
/// ```ignore
/// use nethercore_zx::state::{StatePool, PoolIndex};
///
/// #[derive(Copy, Clone, PartialEq, Eq, Hash)]
/// struct MyIndex(u32);
///
/// impl PoolIndex for MyIndex {
///     fn from_raw(value: u32) -> Self { MyIndex(value) }
///     fn as_raw(&self) -> u32 { self.0 }
/// }
///
/// let mut pool: StatePool<MyState, MyIndex> = StatePool::new("my_state", 65536);
/// let idx = pool.add(my_state);
/// let state = pool.get(idx);
/// ```
pub struct StatePool<T, I>
where
    T: Eq + Hash + Clone,
    I: PoolIndex,
{
    /// Pool name (for error messages)
    name: &'static str,
    /// Maximum capacity (overflow panic threshold)
    max_capacity: usize,
    /// State storage (index = I::as_raw())
    states: Vec<T>,
    /// Deduplication map (state -> index)
    map: HashMap<T, I>,
}

impl<T, I> std::fmt::Debug for StatePool<T, I>
where
    T: Eq + Hash + Clone,
    I: PoolIndex,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatePool")
            .field("name", &self.name)
            .field("max_capacity", &self.max_capacity)
            .field("len", &self.states.len())
            .finish()
    }
}

impl<T, I> StatePool<T, I>
where
    T: Eq + Hash + Clone,
    I: PoolIndex,
{
    /// Create a new empty pool
    ///
    /// # Arguments
    ///
    /// - `name`: Pool name used in overflow panic messages
    /// - `max_capacity`: Maximum number of unique entries (typically 65536 for u16 limit)
    pub fn new(name: &'static str, max_capacity: usize) -> Self {
        Self {
            name,
            max_capacity,
            states: Vec::new(),
            map: HashMap::new(),
        }
    }

    /// Create a pool with pre-allocated capacity
    pub fn with_capacity(name: &'static str, max_capacity: usize, initial_capacity: usize) -> Self {
        Self {
            name,
            max_capacity,
            states: Vec::with_capacity(initial_capacity),
            map: HashMap::with_capacity(initial_capacity),
        }
    }

    /// Add a state to the pool, returning its index (deduplicates)
    ///
    /// If the exact state already exists, returns the existing index.
    /// Otherwise, adds a new entry and returns its index.
    ///
    /// # Panics
    ///
    /// Panics if the pool exceeds max_capacity.
    pub fn add(&mut self, state: T) -> I {
        // Check for existing (deduplication)
        if let Some(&existing_idx) = self.map.get(&state) {
            return existing_idx;
        }

        // Add new state
        let idx = self.states.len();
        if idx >= self.max_capacity {
            panic!(
                "{} pool overflow! Maximum {} unique states per frame.",
                self.name, self.max_capacity
            );
        }

        let index = I::from_raw(idx as u32);
        self.states.push(state.clone());
        self.map.insert(state, index);

        index
    }

    /// Get a state by index
    pub fn get(&self, index: I) -> Option<&T> {
        self.states.get(index.as_raw() as usize)
    }

    /// Get the number of unique states in the pool
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Get the last index (len - 1), or None if empty
    pub fn last_index(&self) -> Option<I> {
        if self.states.is_empty() {
            None
        } else {
            Some(I::from_raw((self.states.len() - 1) as u32))
        }
    }

    /// Clear all states (call once per frame)
    pub fn clear(&mut self) {
        self.states.clear();
        self.map.clear();
    }

    /// Get a slice of all states
    pub fn as_slice(&self) -> &[T] {
        &self.states
    }

    /// Iterate over all states
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.states.iter()
    }
}

impl<T, I> Default for StatePool<T, I>
where
    T: Eq + Hash + Clone,
    I: PoolIndex,
{
    fn default() -> Self {
        Self::new("state", 65536)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test index type
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    struct TestIndex(u32);

    impl PoolIndex for TestIndex {
        fn from_raw(value: u32) -> Self {
            TestIndex(value)
        }
        fn as_raw(&self) -> u32 {
            self.0
        }
    }

    // Test state type
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestState {
        value: u32,
    }

    #[test]
    fn test_pool_add_and_get() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        let state1 = TestState { value: 42 };
        let idx1 = pool.add(state1.clone());

        assert_eq!(idx1.0, 0);
        assert_eq!(pool.get(idx1), Some(&state1));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_pool_deduplication() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        let state = TestState { value: 42 };
        let idx1 = pool.add(state.clone());
        let idx2 = pool.add(state.clone()); // Same state

        assert_eq!(idx1, idx2); // Should return same index
        assert_eq!(pool.len(), 1); // Should only have one entry
    }

    #[test]
    fn test_pool_multiple_states() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        let state1 = TestState { value: 1 };
        let state2 = TestState { value: 2 };
        let state3 = TestState { value: 3 };

        let idx1 = pool.add(state1);
        let idx2 = pool.add(state2);
        let idx3 = pool.add(state3);

        assert_eq!(idx1.0, 0);
        assert_eq!(idx2.0, 1);
        assert_eq!(idx3.0, 2);
        assert_eq!(pool.len(), 3);
    }

    #[test]
    fn test_pool_clear() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        pool.add(TestState { value: 1 });
        pool.add(TestState { value: 2 });
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_last_index() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        assert!(pool.last_index().is_none());

        pool.add(TestState { value: 1 });
        assert_eq!(pool.last_index(), Some(TestIndex(0)));

        pool.add(TestState { value: 2 });
        assert_eq!(pool.last_index(), Some(TestIndex(1)));
    }

    #[test]
    #[should_panic(expected = "pool overflow")]
    fn test_pool_overflow() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 2);

        pool.add(TestState { value: 1 });
        pool.add(TestState { value: 2 });
        pool.add(TestState { value: 3 }); // Should panic
    }

    #[test]
    fn test_pool_as_slice() {
        let mut pool: StatePool<TestState, TestIndex> = StatePool::new("test", 1024);

        pool.add(TestState { value: 1 });
        pool.add(TestState { value: 2 });

        let slice = pool.as_slice();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].value, 1);
        assert_eq!(slice[1].value, 2);
    }
}
