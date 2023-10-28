use array_init::array_init;

use crate::{
    game_modes::{CHASE, PAUSED, SCATTER},
    ghost_state::GhostState,
    location::LocationState,
    variables::*,
};

/// A game state object, to hold the internal game state and provide
/// helper methods that can be accessed by the game engine.
pub struct GameState {
    /* Message header - 4 bytes */
    /// Current ticks.
    currTicks: u32,

    /// Ticks / update.
    updatePeriod: u8,

    /// Last unpaused mode (for pausing purposes).
    lastUnpausedMode: u8,
    /// Game mode.
    pub mode: u8,
    /// Should pause when an update is ready.
    pauseOnUpdate: bool,

    /// The number of steps (update periods) before the mode changes.
    modeSteps: u8,

    /// The number of steps (update periods) before a speedup penalty starts.
    levelSteps: u16,

    /* Game information - 4 bytes */
    /// Current score
    currScore: u16,

    /// Current level (by default, starts at 1)
    currLevel: u8,

    /// Current lives (by default, starts at 3)
    currLives: u8,

    /* Pacman location - 2 bytes */
    pacmanLoc: LocationState,

    /* Fruit location - 2 bytes */
    fruitLoc: LocationState,

    /// The number of steps (update periods) before fruit disappears
    fruitSteps: u8,

    /* Ghosts - 4 * 3 = 12 bytes */
    ghosts: [GhostState; 4],

    /// A variable to keep track of the current ghost combo
    ghostCombo: u8,

    /* Pellet State - 31 * 4 = 124 bytes */
    /// Pellets encoded within an array, with each uint32 acting as a bit array
    pellets: [u32; MAZE_ROWS],

    /// Number of pellets
    numPellets: u16,

    /* Auxiliary (non-serialized) state information */
    /// Wall state
    walls: [u32; MAZE_ROWS],
}

impl GameState {
    /// Creates a new game state with default values.
    pub fn new() -> Self {
        Self {
            // Message header
            currTicks: 0,
            updatePeriod: INIT_UPDATE_PERIOD,
            mode: PAUSED,

            // Additional header-related info
            lastUnpausedMode: INIT_MODE,
            pauseOnUpdate: false,
            modeSteps: MODE_DURATIONS[INIT_MODE],
            levelSteps: LEVEL_DURATION,

            // Game info
            currScore: 0,
            currLevel: INIT_LEVEL,
            currLives: INIT_LIVES,

            pacmanLoc: PACMAN_SPAWN_LOC,

            // Fruit
            fruitLoc: FRUIT_SPAWN_LOC,
            fruitSteps: 0,

            // Ghosts
            ghosts: array_init(|color| GhostState::new(color as u8)),
            ghostCombo: 0,

            // Pellet count at the start
            pellets: INIT_PELLETS,
            numPellets: INIT_PELLET_COUNT,

            // Walls
            walls: INIT_WALLS,
        }
    }

    /**************************** Curr Ticks Functions ****************************/

    /// Helper function to increment the current ticks
    fn nextTick(&mut self) {
        self.currTicks += 1;
    }

    /**************************** Upd Period Functions ****************************/

    /// Helper function to get the update period
    fn getUpdatePeriod(&self) -> u8 {
        self.updatePeriod
    }

    /// Helper function to set the update period
    fn setUpdatePeriod(&mut self, period: u8) {
        // Send a message to the terminal
        println!(
            "\x1b[36mGAME: Update period changed ({} -> {}) (t = {})\x1b[0m\n",
            self.getUpdatePeriod(),
            period,
            self.currTicks,
        );

        self.updatePeriod = period // Update the update period
    }

    /******************************* Mode Functions *******************************/

    // See game_modes.go, there were a lot of mode functions so I moved them there

    /**************************** Game Score Functions ****************************/

    /// Helper function to get the current score of the game
    fn getScore(&self) -> u16 {
        self.currScore
    }

    /// (For performance) helper function to increment the current score of the game
    fn incrementScore(&mut self, change: u16) {
        self.currScore = self.currScore.saturating_add(change);
    }

    /**************************** Game Level Functions ****************************/

    /// Helper function to get the current level of the game
    fn getLevel(&self) -> u8 {
        self.currLevel
    }

    /// Helper function to set the current level of the game
    fn setLevel(&self, level: u8) {
        self.currLevel = level; // Update the level

        // Adjust the initial update period accordingly
        let suggested_period = (INIT_UPDATE_PERIOD as i32) - 2 * ((level as i32) - 1);
        self.setUpdatePeriod(i32::max(1, suggested_period) as u8);
    }

    /// Helper function to increment the game level
    fn incrementLevel(&self) {
        // Keep track of the current level
        let level = self.getLevel();

        // If we are at the last level, don't increment it anymore
        if level == 255 {
            return;
        }

        // Send a message to the terminal
        println!(
            "\x1b[32mGAME: Next level ({} -> {}) (t = {})\x1b[0m\n",
            level,
            level + 1,
            self.currTicks,
        );

        self.setLevel(self.currLevel + 1); // Update the level
    }

    /**************************** Game Level Functions ****************************/

    /// Helper function to get the lives left
    fn getLives(&self) -> u8 {
        self.currLives
    }

    /// Helper function to set the lives left
    fn setLives(&self, lives: u8) {
        // Send a message to the terminal
        println!(
            "\x1b[36mGAME: Lives changed ({} -> {})\x1b[0m\n",
            self.getLives(),
            lives,
        );

        self.currLives = lives; // Update the lives
    }

    /// Helper function to decrement the lives left
    fn decrementLives(&self) {
        // Keep track of how many lives Pacman has left
        let lives = self.getLives();

        // If there were no lives, no need to decrement any more
        if lives == 0 {
            return;
        }

        // Send a message to the terminal
        println!(
            "\x1b[31mGAME: Pacman lost a life ({} -> {}) (t = {})\x1b[0m\n",
            lives,
            lives - 1,
            self.currTicks,
        );

        self.currLives -= 1; // Update the lives
    }

    /****************************** Pellet Functions ******************************/

    /// Helper function to get the number of pellets
    fn getNumPellets(&self) -> u16 {
        self.numPellets
    }

    /// Helper function to decrement the number of pellets
    fn decrementNumPellets(&self) {
        if self.numPellets != 0 {
            self.numPellets -= 1;
        }
    }

    /// Reset all the pellets on the board
    fn resetPellets(&self) {
        // Copy over pellet bit array
        self.pellets = INIT_PELLETS;

        // Set the number of pellets to be the default
        self.numPellets = INIT_PELLET_COUNT;
    }

    /************************** Fruit Spawning Functions **************************/

    /// Helper function to get the number of steps until the fruit disappears
    fn getFruitSteps(&self) -> u8 {
        self.fruitSteps
    }

    /// Helper function to determine whether fruit exists
    fn fruitExists(&self) -> bool {
        self.getFruitSteps() > 0
    }

    /// Helper function to set the number of steps until the fruit disappears
    fn setFruitSteps(&self, steps: u8) {
        self.fruitSteps = steps; // Set the fruit steps
    }

    /// Helper function to decrement the number of fruit steps
    fn decrementFruitSteps(&self) {
        if self.fruitSteps != 0 {
            self.fruitSteps -= 1; // Decrease the fruit steps
        }
    }

    /***************************** Level Steps Passed *****************************/

    /// Helper function to get the number of steps until the level speeds up
    fn getLevelSteps(&self) -> u16 {
        self.levelSteps
    }

    /// Helper function to set the number of steps until the level speeds up
    fn setLevelSteps(&self, steps: u16) {
        self.levelSteps = steps; // Set the level steps
    }

    /// Helper function to decrement the number of steps until the mode changes
    fn decrementLevelSteps(&self) {
        if self.levelSteps != 0 {
            self.levelSteps -= 1; // Decrease the level steps
        }
    }

    /***************************** Step-Related Events ****************************/

    /// Helper function to handle step-related events, if the mode steps hit 0
    fn handleStepEvents(&self) {
        // Get the current mode steps
        let modeSteps = self.get_mode_steps();

        // Get the current level steps
        let levelSteps = self.getLevelSteps();

        // If the mode steps are 0, change the mode
        if modeSteps == 0 {
            match self.get_mode() {
                // CHASE -> SCATTER
                CHASE => {
                    self.set_mode(SCATTER);
                    self.set_mode_steps(MODE_DURATIONS[SCATTER]);
                }
                // SCATTER -> CHASE
                SCATTER => {
                    self.set_mode(CHASE);
                    self.set_mode_steps(MODE_DURATIONS[CHASE]);
                }
                PAUSED => {
                    match self.get_last_unpaused_mode() {
                        // CHASE -> SCATTER
                        CHASE => {
                            self.set_last_unpaused_mode(SCATTER);
                            self.set_mode_steps(MODE_DURATIONS[SCATTER]);
                        }
                        // SCATTER -> CHASE
                        SCATTER => {
                            self.set_last_unpaused_mode(CHASE);
                            self.set_mode_steps(MODE_DURATIONS[CHASE]);
                        }
                    }
                }
            }

            // Reverse the directions of all ghosts to indicate a mode switch
            self.reverseAllGhosts();
        }

        // If the level steps are 0, add a penalty by speeding up the game
        if levelSteps == 0 {
            // Log the change to the terminal
            println!("\x1b[31mGAME: Long-game penalty applied\x1b[0m");

            // Drop the update period by 2
            self.setUpdatePeriod(u8::max(1, self.getUpdatePeriod().saturating_sub(2)));

            // Reset the level steps to the level penalty duration
            self.setLevelSteps(LEVEL_PENALTY_DURATION);
        }

        // Decrement the mode steps
        self.decrementModeSteps();

        // Decrement the level steps
        self.decrementLevelSteps();

        // Decrement the fruit steps
        self.decrementFruitSteps();
    }
}
