//! Game session management with generic resource handling
//!
//! Provides a generic GameSession<C> that works with any Console implementation.

use crate::console::{Console, ConsoleResourceManager};
use crate::runtime::Runtime;
use anyhow::Result;

/// Active game session with console-specific resource management
///
/// This holds the runtime state for a running game, including the
/// resource manager that handles console-specific resource loading.
pub struct GameSession<C: Console> {
    /// The runtime managing game execution
    pub runtime: Runtime<C>,

    /// Console-specific resource manager
    ///
    /// This handles the mapping between game resource handles and
    /// graphics backend handles. The type is determined by the console.
    pub resource_manager: C::ResourceManager,
}

impl<C: Console> GameSession<C> {
    /// Create a new game session
    pub fn new(runtime: Runtime<C>, resource_manager: C::ResourceManager) -> Self {
        Self {
            runtime,
            resource_manager,
        }
    }

    /// Process pending resources from game state
    ///
    /// This should be called after game.init() and after each game.render()
    /// to upload resources (textures, meshes, audio) that were requested
    /// during those phases.
    pub fn process_pending_resources(
        &mut self,
        graphics: &mut C::Graphics,
        audio: &mut C::Audio,
    ) -> Result<()> {
        let game = self.runtime.game_mut()
            .ok_or_else(|| anyhow::anyhow!("No game loaded"))?;

        let state = game.console_state_mut();

        self.resource_manager.process_pending_resources(
            graphics,
            audio,
            state,
        );

        Ok(())
    }

    /// Execute draw commands
    ///
    /// This should be called after game.render() to execute all draw commands
    /// that were recorded during that frame.
    pub fn execute_draw_commands(
        &mut self,
        graphics: &mut C::Graphics,
    ) -> Result<()> {
        let game = self.runtime.game_mut()
            .ok_or_else(|| anyhow::anyhow!("No game loaded"))?;

        let state = game.console_state_mut();

        self.resource_manager.execute_draw_commands(graphics, state);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestConsole;

    #[test]
    fn test_game_session_creation() {
        let console = TestConsole;
        let runtime = Runtime::new(console);
        let resource_manager = console.create_resource_manager();

        let _session = GameSession::new(runtime, resource_manager);
        // Should compile and run without errors
    }
}
