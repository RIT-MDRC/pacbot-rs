use std::io::Cursor;
use std::sync::{Arc, RwLock};
use byteorder::{BigEndian, ReadBytesExt};

use array_init::array_init;
use serde::{Deserialize, Serialize};

use crate::{game_modes::GameMode, ghost_state::GhostState, location::LocationState, variables::*};

/// A game state object, to hold the internal game state and provide
/// helper methods that can be accessed by the game engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    /* Message header - 4 bytes */
    /// Current ticks.
    pub curr_ticks: u32,

    /// Ticks / update.
    pub update_period: u8,

    /// Game mode.
    pub mode: GameMode,

    /// The number of steps (update periods) before the mode changes.
    pub mode_steps: u8,

    /// The number of steps (update periods) before a speedup penalty starts.
    pub level_steps: u16,

    /* Game information - 4 bytes */
    /// Current score
    pub curr_score: u16,

    /// Current level (by default, starts at 1)
    pub curr_level: u8,

    /// Current lives (by default, starts at 3)
    pub curr_lives: u8,

    /* Pacman location - 2 bytes */
    pub pacman_loc: LocationState,

    /* Fruit location - 2 bytes */
    pub fruit_loc: LocationState,

    /// The number of steps (update periods) before fruit disappears
    pub fruit_steps: u8,

    /* Ghosts - 4 * 3 = 12 bytes */
    pub ghosts: [Arc<RwLock<GhostState>>; 4],

    /// The current ghost combo.
    pub ghost_combo: u8,

    /* Pellet State - 31 * 4 = 124 bytes */
    /// Pellets encoded within an array, with each uint32 acting as a bit array
    pub pellets: [u32; MAZE_ROWS],

    /// Number of pellets
    pub num_pellets: u16,

    /* Auxiliary (non-serialized) state information */
    /// Wall state
    pub walls: [u32; MAZE_ROWS],
}

impl GameState {
    /// Creates a new game state with default values.
    pub fn new() -> Self {
        Self {
            // Message header
            curr_ticks: 0,
            update_period: INIT_UPDATE_PERIOD,
            mode: INIT_MODE,

            // Additional header-related info
            mode_steps: INIT_MODE.duration(),
            level_steps: LEVEL_DURATION,

            // Game info
            curr_score: 0,
            curr_level: INIT_LEVEL,
            curr_lives: INIT_LIVES,

            pacman_loc: PACMAN_SPAWN_LOC,

            // Fruit
            fruit_loc: FRUIT_SPAWN_LOC,
            fruit_steps: 0,

            // Ghosts
            ghosts: array_init(|color| Arc::new(RwLock::new(GhostState::new(color as u8)))),
            ghost_combo: 0,

            // Pellet count at the start
            pellets: INIT_PELLETS,
            num_pellets: INIT_PELLET_COUNT,

            // Walls
            walls: INIT_WALLS,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) -> bool {
        let mut cursor = Cursor::new(bytes);

        // General game info
        self.curr_ticks = cursor.read_u16::<BigEndian>().unwrap() as u32;
        self.update_period = cursor.read_u8().unwrap();
        let (mode, paused) = match cursor.read_u8().unwrap() {
            0 => (GameMode::CHASE, true),
            1 => (GameMode::SCATTER, false),
            2 => (GameMode::CHASE, false),
            _ => unreachable!()
        };
        self.mode = mode;
        self.mode_steps = cursor.read_u8().unwrap();
        let _mode_duration = cursor.read_u8().unwrap();
        self.curr_score = cursor.read_u16::<BigEndian>().unwrap();
        self.curr_level = cursor.read_u8().unwrap();
        self.curr_lives = cursor.read_u8().unwrap();

        // red ghost info
        for g in 0..4 {
            let mut ghost = self.ghosts[g].write().unwrap();
            ghost.loc.update(cursor.read_u16::<BigEndian>().unwrap());
            ghost.update_aux(cursor.read_u8().unwrap());
        }

        // pacman location info
        self.pacman_loc.update(cursor.read_u16::<BigEndian>().unwrap());

        // fruit location info
        self.fruit_loc.update(cursor.read_u16::<BigEndian>().unwrap());
        self.fruit_steps = cursor.read_u8().unwrap();
        let _fruit_duration = cursor.read_u8().unwrap();

        // Pellet info
        for i in 0..31 {
            self.pellets[i] = cursor.read_u32::<BigEndian>().unwrap();
        }

        paused
    }

    /**************************** Ghost Array Helpers *****************************/

    /// Returns an iterator that yields mutable references to the four ghosts.
    pub fn ghosts_mut(&self) -> impl Iterator<Item = Arc<RwLock<GhostState>>> + '_ {
        self.ghosts.iter().map(|a| a.clone())
    }

    /**************************** Curr Ticks Functions ****************************/

    /// Helper function to increment the current ticks
    pub fn next_tick(&mut self) {
        self.curr_ticks += 1;
    }

    /**************************** Upd Period Functions ****************************/

    /// Helper function to get the update period
    pub fn get_update_period(&self) -> u8 {
        self.update_period
    }

    /// Helper function to set the update period
    pub fn set_update_period(&mut self, period: u8) {
        // Send a message to the terminal
        println!(
            "\x1b[36mGAME: Update period changed ({} -> {}) (t = {})\x1b[0m\n",
            self.get_update_period(),
            period,
            self.curr_ticks,
        );

        self.update_period = period // Update the update period
    }

    /******************************* Mode Functions *******************************/

    // See game_modes.go, there were a lot of mode functions so I moved them there

    /**************************** Game Score Functions ****************************/

    /// Helper function to get the current score of the game
    pub fn get_score(&self) -> u16 {
        self.curr_score
    }

    /// Helper function to increment the current score of the game
    pub fn increment_score(&mut self, change: u16) {
        self.curr_score = self.curr_score.saturating_add(change);
    }

    /**************************** Game Level Functions ****************************/

    /// Helper function to get the current level of the game
    pub fn get_level(&self) -> u8 {
        self.curr_level
    }

    /// Helper function to set the current level of the game
    pub fn set_level(&mut self, level: u8) {
        self.curr_level = level; // Update the level

        // Adjust the initial update period accordingly
        let suggested_period = (INIT_UPDATE_PERIOD as i32) - 2 * ((level as i32) - 1);
        self.set_update_period(i32::max(1, suggested_period) as u8);
    }

    /// Helper function to increment the game level
    pub fn increment_level(&mut self) {
        // Keep track of the current level
        let level = self.get_level();

        // If we are at the last level, don't increment it anymore
        if level == 255 {
            return;
        }

        // Send a message to the terminal
        println!(
            "\x1b[32mGAME: Next level ({} -> {}) (t = {})\x1b[0m\n",
            level,
            level + 1,
            self.curr_ticks,
        );

        self.set_level(self.curr_level + 1); // Update the level
    }

    /**************************** Game Level Functions ****************************/

    /// Helper function to get the lives left
    pub fn get_lives(&self) -> u8 {
        self.curr_lives
    }

    /// Helper function to set the lives left
    pub fn set_lives(&mut self, lives: u8) {
        // Send a message to the terminal
        println!(
            "\x1b[36mGAME: Lives changed ({} -> {})\x1b[0m\n",
            self.get_lives(),
            lives,
        );

        self.curr_lives = lives; // Update the lives
    }

    /// Helper function to decrement the lives left
    pub fn decrement_lives(&mut self) {
        // Keep track of how many lives Pacman has left
        let lives = self.get_lives();

        // If there were no lives, no need to decrement any more
        if lives == 0 {
            return;
        }

        // Send a message to the terminal
        println!(
            "\x1b[31mGAME: Pacman lost a life ({} -> {}) (t = {})\x1b[0m\n",
            lives,
            lives - 1,
            self.curr_ticks,
        );

        self.curr_lives -= 1; // Update the lives
    }

    /****************************** Pellet Functions ******************************/

    /// Helper function to get the number of pellets
    pub fn get_num_pellets(&self) -> u16 {
        self.num_pellets
    }

    /// Helper function to decrement the number of pellets
    pub fn decrement_num_pellets(&mut self) {
        if self.num_pellets != 0 {
            self.num_pellets -= 1;
        }
    }

    /// Reset all the pellets on the board
    pub fn reset_pellets(&mut self) {
        // Copy over pellet bit array
        self.pellets = INIT_PELLETS;

        // Set the number of pellets to be the default
        self.num_pellets = INIT_PELLET_COUNT;
    }

    /************************** Fruit Spawning Functions **************************/

    /// Helper function to get the number of steps until the fruit disappears
    pub fn get_fruit_steps(&self) -> u8 {
        self.fruit_steps
    }

    /// Helper function to determine whether fruit exists
    pub fn fruit_exists(&self) -> bool {
        self.get_fruit_steps() > 0
    }

    /// Helper function to set the number of steps until the fruit disappears
    pub fn set_fruit_steps(&mut self, steps: u8) {
        self.fruit_steps = steps; // Set the fruit steps
    }

    /// Helper function to decrement the number of fruit steps
    pub fn decrement_fruit_steps(&mut self) {
        if self.fruit_steps != 0 {
            self.fruit_steps -= 1; // Decrease the fruit steps
        }
    }

    /***************************** Level Steps Passed *****************************/

    /// Helper function to get the number of steps until the level speeds up
    pub fn get_level_steps(&self) -> u16 {
        self.level_steps
    }

    /// Helper function to set the number of steps until the level speeds up
    pub fn set_level_steps(&mut self, steps: u16) {
        self.level_steps = steps; // Set the level steps
    }

    /// Helper function to decrement the number of steps until the mode changes
    pub fn decrement_level_steps(&mut self) {
        if self.level_steps != 0 {
            self.level_steps -= 1; // Decrease the level steps
        }
    }

    /***************************** Step-Related Events ****************************/

    /// Helper function to handle step-related events, if the mode steps hit 0
    pub fn handle_step_events(&mut self) {
        // If the mode steps are 0, change the mode
        if self.mode_steps == 0 {
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
            self.reverse_all_ghosts();
        }

        // If the level steps are 0, add a penalty by speeding up the game
        if self.level_steps == 0 {
            // Log the change to the terminal
            println!("\x1b[31mGAME: Long-game penalty applied\x1b[0m");

            // Drop the update period by 2
            self.set_update_period(u8::max(1, self.get_update_period().saturating_sub(2)));

            // Reset the level steps to the level penalty duration
            self.set_level_steps(LEVEL_PENALTY_DURATION);
        }

        // Decrement the mode steps
        if self.get_num_pellets() >= ANGER_THRESHOLD1 {
            self.decrement_mode_steps();
        }

        // Decrement the level steps
        self.decrement_level_steps();

        // Decrement the fruit steps
        self.decrement_fruit_steps();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
