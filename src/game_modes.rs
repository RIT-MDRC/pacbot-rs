use crate::game_state::GameState;

// The possible game modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameMode {
    SCATTER,
    CHASE,
}

impl GameMode {
    /// Returns the length of the game mode, in units of steps (update periods).
    pub fn duration(self) -> u8 {
        match self {
            GameMode::SCATTER => 60, // 30 seconds at 24 fps
            GameMode::CHASE => 180,  // 90 seconds at 24 fps
        }
    }
}

impl GameState {
    // Helper function to get the number of steps until the mode changes
    pub fn get_mode_steps(&self) -> u8 {
        self.modeSteps
    }

    // Helper function to set the number of steps until the mode changes
    pub fn set_mode_steps(&mut self, steps: u8) {
        self.modeSteps = steps;
    }

    // Helper function to decrement the number of steps until the mode changes
    pub fn decrement_mode_steps(&mut self) {
        if self.modeSteps != 0 {
            self.modeSteps -= 1;
        }
    }
}
