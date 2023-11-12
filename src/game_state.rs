use array_init::array_init;

use crate::{game_modes::GameMode, ghost_state::GhostState, location::LocationState, variables::*};

/// A game state object, to hold the internal game state and provide
/// helper methods that can be accessed by the game engine.
pub struct GameState {
    /* Message header - 4 bytes */
    /// Current ticks.
    pub currTicks: u32,

    /// Ticks / update.
    pub updatePeriod: u8,

    /// Game mode.
    pub mode: GameMode,

    /// The number of steps (update periods) before the mode changes.
    pub modeSteps: u8,

    /// The number of steps (update periods) before a speedup penalty starts.
    pub levelSteps: u16,

    /* Game information - 4 bytes */
    /// Current score
    pub currScore: u16,

    /// Current level (by default, starts at 1)
    pub currLevel: u8,

    /// Current lives (by default, starts at 3)
    pub currLives: u8,

    /* Pacman location - 2 bytes */
    pub pacmanLoc: LocationState,

    /* Fruit location - 2 bytes */
    pub fruitLoc: LocationState,

    /// The number of steps (update periods) before fruit disappears
    pub fruitSteps: u8,

    /* Ghosts - 4 * 3 = 12 bytes */
    pub ghosts: [GhostState; 4],

    /// The current ghost combo.
    pub ghostCombo: u8,

    /* Pellet State - 31 * 4 = 124 bytes */
    /// Pellets encoded within an array, with each uint32 acting as a bit array
    pub pellets: [u32; MAZE_ROWS],

    /// Number of pellets
    pub numPellets: u16,

    /* Auxiliary (non-serialized) state information */
    /// Wall state
    pub walls: [u32; MAZE_ROWS],
}

impl GameState {
    /// Creates a new game state with default values.
    pub fn new() -> Self {
        Self {
            // Message header
            currTicks: 0,
            updatePeriod: INIT_UPDATE_PERIOD,
            mode: INIT_MODE,

            // Additional header-related info
            modeSteps: INIT_MODE.duration(),
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
    pub fn nextTick(&mut self) {
        self.currTicks += 1;
    }

    /**************************** Upd Period Functions ****************************/

    /// Helper function to get the update period
    pub fn getUpdatePeriod(&self) -> u8 {
        self.updatePeriod
    }

    /// Helper function to set the update period
    pub fn setUpdatePeriod(&mut self, period: u8) {
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
    pub fn getScore(&self) -> u16 {
        self.currScore
    }

    /// (For performance) helper function to increment the current score of the game
    pub fn incrementScore(&mut self, change: u16) {
        self.currScore = self.currScore.saturating_add(change);
    }

    /**************************** Game Level Functions ****************************/

    /// Helper function to get the current level of the game
    pub fn getLevel(&self) -> u8 {
        self.currLevel
    }

    /// Helper function to set the current level of the game
    pub fn setLevel(&mut self, level: u8) {
        self.currLevel = level; // Update the level

        // Adjust the initial update period accordingly
        let suggested_period = (INIT_UPDATE_PERIOD as i32) - 2 * ((level as i32) - 1);
        self.setUpdatePeriod(i32::max(1, suggested_period) as u8);
    }

    /// Helper function to increment the game level
    pub fn incrementLevel(&mut self) {
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
    pub fn getLives(&self) -> u8 {
        self.currLives
    }

    /// Helper function to set the lives left
    pub fn setLives(&mut self, lives: u8) {
        // Send a message to the terminal
        println!(
            "\x1b[36mGAME: Lives changed ({} -> {})\x1b[0m\n",
            self.getLives(),
            lives,
        );

        self.currLives = lives; // Update the lives
    }

    /// Helper function to decrement the lives left
    pub fn decrementLives(&mut self) {
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
    pub fn getNumPellets(&self) -> u16 {
        self.numPellets
    }

    /// Helper function to decrement the number of pellets
    pub fn decrementNumPellets(&mut self) {
        if self.numPellets != 0 {
            self.numPellets -= 1;
        }
    }

    /// Reset all the pellets on the board
    pub fn resetPellets(&mut self) {
        // Copy over pellet bit array
        self.pellets = INIT_PELLETS;

        // Set the number of pellets to be the default
        self.numPellets = INIT_PELLET_COUNT;
    }

    /************************** Fruit Spawning Functions **************************/

    /// Helper function to get the number of steps until the fruit disappears
    pub fn getFruitSteps(&self) -> u8 {
        self.fruitSteps
    }

    /// Helper function to determine whether fruit exists
    pub fn fruitExists(&self) -> bool {
        self.getFruitSteps() > 0
    }

    /// Helper function to set the number of steps until the fruit disappears
    pub fn setFruitSteps(&mut self, steps: u8) {
        self.fruitSteps = steps; // Set the fruit steps
    }

    /// Helper function to decrement the number of fruit steps
    pub fn decrementFruitSteps(&mut self) {
        if self.fruitSteps != 0 {
            self.fruitSteps -= 1; // Decrease the fruit steps
        }
    }

    /***************************** Level Steps Passed *****************************/

    /// Helper function to get the number of steps until the level speeds up
    pub fn getLevelSteps(&self) -> u16 {
        self.levelSteps
    }

    /// Helper function to set the number of steps until the level speeds up
    pub fn setLevelSteps(&mut self, steps: u16) {
        self.levelSteps = steps; // Set the level steps
    }

    /// Helper function to decrement the number of steps until the mode changes
    pub fn decrementLevelSteps(&mut self) {
        if self.levelSteps != 0 {
            self.levelSteps -= 1; // Decrease the level steps
        }
    }

    /***************************** Step-Related Events ****************************/

    /// Helper function to handle step-related events, if the mode steps hit 0
    pub fn handleStepEvents(&mut self) {
        // If the mode steps are 0, change the mode
        if self.modeSteps == 0 {
            match self.mode {
                // CHASE -> SCATTER
                GameMode::CHASE => {
                    self.mode = GameMode::SCATTER;
                    self.set_mode_steps(GameMode::SCATTER.duration());
                }
                // SCATTER -> CHASE
                GameMode::SCATTER => {
                    self.mode = GameMode::CHASE;
                    self.set_mode_steps(GameMode::CHASE.duration());
                }
            }

            // Reverse the directions of all ghosts to indicate a mode switch
            self.reverseAllGhosts();
        }

        // If the level steps are 0, add a penalty by speeding up the game
        if self.levelSteps == 0 {
            // Log the change to the terminal
            println!("\x1b[31mGAME: Long-game penalty applied\x1b[0m");

            // Drop the update period by 2
            self.setUpdatePeriod(u8::max(1, self.getUpdatePeriod().saturating_sub(2)));

            // Reset the level steps to the level penalty duration
            self.setLevelSteps(LEVEL_PENALTY_DURATION);
        }

        // Decrement the mode steps
        self.decrement_mode_steps();

        // Decrement the level steps
        self.decrementLevelSteps();

        // Decrement the fruit steps
        self.decrementFruitSteps();
    }
}
