use crate::game_state::GameState;

// Enum-like declaration to hold the game mode options
pub const PAUSED: u8 = 0;
pub const SCATTER: u8 = 1;
pub const CHASE: u8 = 2;
pub const NUM_MODES: u8 = 3;

// Names of the modes (for logging)
// var modeNames [numModes]string = [...]string{
// 	"paused",
// 	"scatter",
// 	"chase",
// }

/******************************** Current Mode ********************************/

impl GameState {
    // Helper function to get the game mode
    pub fn get_mode(&self) -> u8 {
        // Return the current game mode
        self.mode
    }

    // Helper function to set the game mode
    pub fn set_mode(&mut self, mode: u8) {
        // Read the current game mode
        let curr_mode = self.get_mode();

        // If the game is not paused and won't be paused, log the change
        // if curr_mode != paused && mode != paused && curr_mode != mode {
        // 	log.Printf("\033[36mGAME: Mode changed (%s -> %s) (t = %d)\033[0m\n",
        // 		modeNames[curr_mode], modeNames[mode], self.getCurrTicks())
        // }
        self.mode = mode;
    }

    /***************************** Last Unpaused Mode *****************************/

    // Helper function to get the last unpaused mode
    pub fn get_last_unpaused_mode(&self) -> u8 {
        // If the current mode is not paused, return it
        if self.mode != PAUSED {
            return self.mode;
        }

        // Return the last unpaused game mode
        self.lastUnpausedMode
    }

    // Helper function to set the game mode
    pub fn set_last_unpaused_mode(&self, mode: u8) {
        // Get the last unpaused mode
        let unpaused_mode = self.get_last_unpaused_mode();

        // If the game is paused and the last unpaused mode changes, log the change
        // if self.get_mode() == PAUSED && unpausedMode != mode {
        // 	log.Printf("\036[32mGAME: Mode changed while paused (%s -> %s) "+
        // 		"(t = %d)\033[0m\n",
        // 		modeNames[unpausedMode], modeNames[mode], self.getCurrTicks())
        // }

        // (Write) lock the game mode
        self.lastUnpausedMode = mode; // Update the game mode
    }

    /******************************** Pause / Play ********************************/

    // Helper function to determine if the game is paused
    pub fn is_paused(&self) -> bool {
        self.get_mode() == PAUSED
    }

    // Helper function to pause the game
    pub fn pause(&self) {
        // If the game engine is already paused, there's no more to do
        if self.is_paused() {
            return;
        }

        // Otherwise, save the current mode
        self.set_last_unpaused_mode(self.get_mode());

        // Set the mode to paused
        self.set_mode(PAUSED);

        // Log message to alert the user
        // log.Printf("\033[32m\033[2mGAME: Paused  (t = %d)\033[0m\n",
        // 	self.getCurrTicks())
    }

    // Helper function to play the game
    pub fn play(&self) {
        // If the game engine is already playing or can't play, return
        if !self.is_paused() || self.getLives() == 0 {
            return;
        }

        // Otherwise, set the current mode to the last unpaused mode
        self.set_mode(self.get_last_unpaused_mode())

        // Log message to alert the user
        // log.Printf("\033[32mGAME: Resumed (t = %d)\033[0m\n",
        // 	self.getCurrTicks())
    }

    /*************************** Pausing on Next Update ***************************/

    // Helper function to return whether the game should pause after next update
    pub fn get_pause_on_update(&self) -> bool {
        // Return whether the pause on update flag
        return self.pauseOnUpdate;
    }

    // Helper function to pause the game after the next update
    pub fn set_pause_on_update(&self, flag: bool) {
        self.pauseOnUpdate = flag; // Set a flag to pause at the next update
    }

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
