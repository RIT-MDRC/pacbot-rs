use crate::game_state::GameState;

// The possible game modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameMode {
    SCATTER, // 1
    CHASE,   // 2
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
    /********************************* Mode Steps *********************************/

    // Helper function to get the number of steps until the mode changes
    pub fn get_mode_steps(&self) -> u8 {
        // Return the mode steps
        return self.modeSteps;
    }

    // Helper function to set the number of steps until the mode changes
    pub fn set_mode_steps(&self, steps: u8) {
        // (Write) lock the mode steps
        self.modeSteps = steps; // Set the mode steps
    }

    // Helper function to decrement the number of steps until the mode changes
    pub fn decrement_mode_steps(&self) {
        if self.modeSteps != 0 {
            self.modeSteps -= 1; // Decrease the mode steps
        }
    }
}
