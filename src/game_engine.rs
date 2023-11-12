use crate::game_state::GameState;

/// A game engine object, to act as an intermediary between the web broker
/// and the internal game state - its responsibility is to read responses from
/// clients and routinely send serialized copies of the game state to them.
pub struct GameEngine {
    state: GameState,
}

impl GameEngine {
    /// Create a new game engine, casting channels to be uni-directional
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
        }
    }

    /// Start the game engine - should be launched as a go-routine.
    pub fn step(&mut self) {
        if self.state.update_ready() {
            self.state.update_all_ghosts();
            self.state.try_respawn_pacman();
            self.state.check_collisions();
            self.state.handle_step_events();
        }
        if self.state.update_ready() {
            self.state.plan_all_ghosts();
        }
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}
