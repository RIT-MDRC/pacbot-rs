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
    pub fn step(&self) {
        if self.state.updateReady() {
            self.state.updateAllGhosts();
            self.state.tryRespawnPacman();
            if self.state.pauseOnUpdate {
                self.state.pause();
                self.state.set_pause_on_update(false);
            }
            self.state.checkCollisions();
            self.state.handleStepEvents();
        }
        if self.state.updateReady() {
            self.state.planAllGhosts();
        }
    }
}
