use serde::{Deserialize, Serialize};
use crate::game_state::GameState;
use crate::location::LocationState;

/// A game engine object, to act as an intermediary between the web broker
/// and the internal game state - its responsibility is to read responses from
/// clients and routinely send serialized copies of the game state to them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameEngine {
    state: GameState,
    paused: bool,
}

impl GameEngine {
    /// Create a new game engine, casting channels to be uni-directional
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            paused: false,
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

    /// Pause the game engine
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Unpause the game engine
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Get the current game state
    pub fn get_state(&self) -> &GameState {
        &self.state
    }

    /// Get whether the game is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Set pacman's location
    pub fn set_pacman_location(&mut self, location: LocationState) {
        self.state.pacman_loc = location;
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}
