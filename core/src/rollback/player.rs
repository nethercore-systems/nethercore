//! Player session configuration
//!
//! Manages local vs remote player assignments for multiplayer sessions.

/// Maximum number of players in a session
pub const MAX_PLAYERS: usize = 4;

/// Configuration for player assignments in a multiplayer session
///
/// Specifies how many players are in the session, which are local (controlled
/// by physical input devices on this machine), and which are remote (controlled
/// by networked peers).
///
/// # Examples
///
/// ```
/// use emberware_core::rollback::PlayerSessionConfig;
///
/// // 4 local players on one machine (couch co-op)
/// let config = PlayerSessionConfig::all_local(4);
/// assert_eq!(config.num_players(), 4);
/// assert_eq!(config.local_player_count(), 4);
///
/// // 1 local + 1 remote (standard online 1v1)
/// let config = PlayerSessionConfig::one_local(2);
/// assert_eq!(config.num_players(), 2);
/// assert_eq!(config.local_player_count(), 1);
///
/// // 2 local + 2 remote (2v2 with a friend on the couch)
/// let config = PlayerSessionConfig::new(4, 0b0011); // Players 0,1 local
/// assert_eq!(config.local_player_count(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSessionConfig {
    /// Total number of players (1-4)
    num_players: u32,
    /// Bitmask indicating which players are local (bit N = player N is local)
    local_player_mask: u32,
}

impl PlayerSessionConfig {
    /// Create a new player session configuration
    ///
    /// # Arguments
    /// * `num_players` - Total players in session (1-4)
    /// * `local_player_mask` - Bitmask of local players (e.g., 0b0011 = players 0 and 1)
    ///
    /// # Panics
    /// Panics if `num_players` is 0 or greater than 4.
    pub fn new(num_players: u32, local_player_mask: u32) -> Self {
        assert!(
            num_players >= 1 && num_players <= MAX_PLAYERS as u32,
            "num_players must be 1-4, got {}",
            num_players
        );

        // Mask off bits beyond num_players
        let valid_mask = (1u32 << num_players) - 1;
        let local_player_mask = local_player_mask & valid_mask;

        Self {
            num_players,
            local_player_mask,
        }
    }

    /// Create configuration for all local players (single machine)
    ///
    /// All players are controlled by local input devices.
    /// Use this for single player or couch co-op.
    pub fn all_local(num_players: u32) -> Self {
        assert!(
            num_players >= 1 && num_players <= MAX_PLAYERS as u32,
            "num_players must be 1-4, got {}",
            num_players
        );
        Self {
            num_players,
            local_player_mask: (1u32 << num_players) - 1, // All bits set
        }
    }

    /// Create configuration with only player 0 as local
    ///
    /// Player 0 is controlled locally, all others are remote.
    /// This is the common case for online play.
    pub fn one_local(num_players: u32) -> Self {
        assert!(
            num_players >= 1 && num_players <= MAX_PLAYERS as u32,
            "num_players must be 1-4, got {}",
            num_players
        );
        Self {
            num_players,
            local_player_mask: 0b0001, // Only player 0 is local
        }
    }

    /// Create configuration with multiple specific local players
    ///
    /// # Arguments
    /// * `num_players` - Total players in session
    /// * `local_players` - Slice of player indices that are local (0-based)
    ///
    /// # Example
    /// ```
    /// use emberware_core::rollback::PlayerSessionConfig;
    ///
    /// // 4 players, players 0 and 2 are local
    /// let config = PlayerSessionConfig::with_local_players(4, &[0, 2]);
    /// assert!(config.is_local_player(0));
    /// assert!(!config.is_local_player(1));
    /// assert!(config.is_local_player(2));
    /// assert!(!config.is_local_player(3));
    /// ```
    pub fn with_local_players(num_players: u32, local_players: &[usize]) -> Self {
        assert!(
            num_players >= 1 && num_players <= MAX_PLAYERS as u32,
            "num_players must be 1-4, got {}",
            num_players
        );

        let mut mask = 0u32;
        for &player in local_players {
            if player < num_players as usize {
                mask |= 1u32 << player;
            }
        }

        Self {
            num_players,
            local_player_mask: mask,
        }
    }

    /// Get the total number of players
    pub fn num_players(&self) -> u32 {
        self.num_players
    }

    /// Get the local player mask
    pub fn local_player_mask(&self) -> u32 {
        self.local_player_mask
    }

    /// Check if a player is local
    pub fn is_local_player(&self, player: usize) -> bool {
        if player >= self.num_players as usize {
            return false;
        }
        (self.local_player_mask & (1u32 << player)) != 0
    }

    /// Get the number of local players
    pub fn local_player_count(&self) -> u32 {
        self.local_player_mask.count_ones()
    }

    /// Get the number of remote players
    pub fn remote_player_count(&self) -> u32 {
        self.num_players - self.local_player_count()
    }

    /// Get indices of all local players
    pub fn local_player_indices(&self) -> Vec<usize> {
        (0..self.num_players as usize)
            .filter(|&i| self.is_local_player(i))
            .collect()
    }

    /// Get indices of all remote players
    pub fn remote_player_indices(&self) -> Vec<usize> {
        (0..self.num_players as usize)
            .filter(|&i| !self.is_local_player(i))
            .collect()
    }
}

impl Default for PlayerSessionConfig {
    /// Default configuration: 1 player, local
    fn default() -> Self {
        Self::all_local(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_session_config_all_local_1p() {
        let config = PlayerSessionConfig::all_local(1);
        assert_eq!(config.num_players(), 1);
        assert_eq!(config.local_player_mask(), 0b0001);
        assert_eq!(config.local_player_count(), 1);
        assert_eq!(config.remote_player_count(), 0);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
    }

    #[test]
    fn test_player_session_config_all_local_4p() {
        let config = PlayerSessionConfig::all_local(4);
        assert_eq!(config.num_players(), 4);
        assert_eq!(config.local_player_mask(), 0b1111);
        assert_eq!(config.local_player_count(), 4);
        assert_eq!(config.remote_player_count(), 0);
        for i in 0..4 {
            assert!(config.is_local_player(i), "Player {} should be local", i);
        }
        assert!(!config.is_local_player(4)); // Out of range
    }

    #[test]
    fn test_player_session_config_one_local_2p() {
        let config = PlayerSessionConfig::one_local(2);
        assert_eq!(config.num_players(), 2);
        assert_eq!(config.local_player_mask(), 0b0001);
        assert_eq!(config.local_player_count(), 1);
        assert_eq!(config.remote_player_count(), 1);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
    }

    #[test]
    fn test_player_session_config_one_local_4p() {
        let config = PlayerSessionConfig::one_local(4);
        assert_eq!(config.num_players(), 4);
        assert_eq!(config.local_player_mask(), 0b0001);
        assert_eq!(config.local_player_count(), 1);
        assert_eq!(config.remote_player_count(), 3);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
        assert!(!config.is_local_player(2));
        assert!(!config.is_local_player(3));
    }

    #[test]
    fn test_player_session_config_custom_mask() {
        // 4 players, players 0 and 2 are local
        let config = PlayerSessionConfig::new(4, 0b0101);
        assert_eq!(config.num_players(), 4);
        assert_eq!(config.local_player_mask(), 0b0101);
        assert_eq!(config.local_player_count(), 2);
        assert_eq!(config.remote_player_count(), 2);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
        assert!(config.is_local_player(2));
        assert!(!config.is_local_player(3));
    }

    #[test]
    fn test_player_session_config_with_local_players() {
        let config = PlayerSessionConfig::with_local_players(4, &[0, 2]);
        assert_eq!(config.num_players(), 4);
        assert_eq!(config.local_player_mask(), 0b0101);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
        assert!(config.is_local_player(2));
        assert!(!config.is_local_player(3));
    }

    #[test]
    fn test_player_session_config_with_local_players_invalid_indices() {
        // Indices beyond num_players should be ignored
        let config = PlayerSessionConfig::with_local_players(2, &[0, 5, 10]);
        assert_eq!(config.num_players(), 2);
        assert_eq!(config.local_player_mask(), 0b0001);
        assert!(config.is_local_player(0));
        assert!(!config.is_local_player(1));
    }

    #[test]
    fn test_player_session_config_mask_clamps_to_num_players() {
        // If mask has bits beyond num_players, they should be masked off
        let config = PlayerSessionConfig::new(2, 0b1111);
        assert_eq!(config.num_players(), 2);
        // Only bits 0 and 1 should be set
        assert_eq!(config.local_player_mask(), 0b0011);
    }

    #[test]
    fn test_player_session_config_local_player_indices() {
        let config = PlayerSessionConfig::new(4, 0b1010); // Players 1 and 3
        let indices = config.local_player_indices();
        assert_eq!(indices, vec![1, 3]);
    }

    #[test]
    fn test_player_session_config_remote_player_indices() {
        let config = PlayerSessionConfig::new(4, 0b1010); // Players 1 and 3 local
        let indices = config.remote_player_indices();
        assert_eq!(indices, vec![0, 2]); // Players 0 and 2 are remote
    }

    #[test]
    fn test_player_session_config_default() {
        let config = PlayerSessionConfig::default();
        assert_eq!(config.num_players(), 1);
        assert_eq!(config.local_player_mask(), 1);
        assert!(config.is_local_player(0));
    }

    #[test]
    fn test_player_session_config_equality() {
        let config1 = PlayerSessionConfig::new(4, 0b0011);
        let config2 = PlayerSessionConfig::new(4, 0b0011);
        let config3 = PlayerSessionConfig::new(4, 0b0101);

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_player_session_config_copy() {
        let config1 = PlayerSessionConfig::new(4, 0b0011);
        let config2 = config1; // Copy
        assert_eq!(config1, config2);
    }

    #[test]
    #[should_panic(expected = "num_players must be 1-4")]
    fn test_player_session_config_zero_players_panics() {
        PlayerSessionConfig::all_local(0);
    }

    #[test]
    #[should_panic(expected = "num_players must be 1-4")]
    fn test_player_session_config_five_players_panics() {
        PlayerSessionConfig::all_local(5);
    }
}
