use array_init::array_init;
use core2::io;
use core2::io::{Cursor, Read};
#[cfg(feature = "std")]
use rand::{prelude::SmallRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::game_helpers::Position;
use crate::ghost_state::{GhostColor, GHOST_NAMES};
use crate::location::Direction;
use crate::{game_modes::GameMode, ghost_state::GhostState, location::LocationState, variables::*};

/// A game state object, to hold the internal game state and provide
/// helper methods that can be accessed by the game engine.
///
/// Logging can be disabled by setting the environment variable DISABLE_PACMAN_LOGGING
#[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct GameState {
    /* Message header - 4 bytes */
    /// Current ticks.
    pub curr_ticks: u32,

    /// Ticks / update.
    pub update_period: u8,

    /// Game mode.
    pub mode: GameMode,

    /// Whether game is paused - has no effect on internal messages, simply copies value
    /// from incoming game states
    pub paused: bool,

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
    pub ghosts: [GhostState; 4],

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

    /// Used to generate ghost moves (updated each time)
    pub seed: u64,
}

#[cfg(feature = "std")]
impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Creates a new game state with default values.
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        Self::new_with_seed(SmallRng::from_entropy().gen())
    }

    /// Creates a new game state with default values.
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            // Message header
            curr_ticks: 0,
            update_period: INIT_UPDATE_PERIOD,
            mode: INIT_MODE,
            paused: true,

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
            ghosts: array_init(|color| GhostState::new((color as u8).try_into().unwrap())),
            ghost_combo: 0,

            // Pellet count at the start
            pellets: INIT_PELLETS,
            num_pellets: INIT_PELLET_COUNT,

            // Walls
            walls: INIT_WALLS,

            // For ghost moves
            seed,
        }
    }

    pub fn from_bytes(bytes: &[u8], seed: u64) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        fn get_u8(cursor: &mut Cursor<&[u8]>) -> io::Result<u8> {
            let mut buf = [0];
            cursor.read_exact(&mut buf)?;
            Ok(buf[0])
        }

        fn get_u16(cursor: &mut Cursor<&[u8]>) -> io::Result<u16> {
            let mut buf = [0; 2];
            cursor.read_exact(&mut buf)?;
            Ok(u16::from_be_bytes(buf))
        }

        fn get_u32(cursor: &mut Cursor<&[u8]>) -> io::Result<u32> {
            let mut buf = [0; 4];
            cursor.read_exact(&mut buf)?;
            Ok(u32::from_be_bytes(buf))
        }

        // General game info
        let curr_ticks = get_u16(&mut cursor)? as u32;
        let update_period = get_u8(&mut cursor)?;
        let (mode, paused) = match get_u8(&mut cursor)? {
            0 => (GameMode::CHASE, true),
            1 => (GameMode::SCATTER, false),
            2 => (GameMode::CHASE, false),
            _ => unreachable!(),
        };
        let mode = mode;
        let paused = paused;
        let mode_steps = get_u8(&mut cursor)?;
        let _mode_duration = get_u8(&mut cursor)?;
        let curr_score = get_u16(&mut cursor)?;
        let curr_level = get_u8(&mut cursor)?;
        let curr_lives = get_u8(&mut cursor)?;

        // ghost info
        let mut ghosts = [GhostState::from_bytes(GhostColor::Red, EMPTY_LOC, 0); 4];
        for (i, g) in ghosts.iter_mut().enumerate() {
            let loc = LocationState::from_bytes([get_u8(&mut cursor)?, get_u8(&mut cursor)?])?;
            *g = GhostState::from_bytes(GHOST_NAMES[i], loc, get_u8(&mut cursor)?)
        }

        // pacman location info
        let pacman_loc = LocationState::from_bytes([get_u8(&mut cursor)?, get_u8(&mut cursor)?])?;

        // fruit location info
        let fruit_loc = LocationState::from_bytes([get_u8(&mut cursor)?, get_u8(&mut cursor)?])?;
        let fruit_steps = get_u8(&mut cursor)?;
        let _fruit_duration = get_u8(&mut cursor)?;

        // Pellet info
        let pellets = array_init(|_| get_u32(&mut cursor).unwrap_or(0));

        let mut s = Self {
            curr_ticks,
            update_period,
            mode,
            paused,
            mode_steps,
            level_steps: 0, // todo based on curr_ticks
            curr_score,
            curr_level,
            curr_lives,
            pacman_loc,
            fruit_loc,
            fruit_steps,
            ghosts,
            ghost_combo: 0, // todo
            num_pellets: pellets.iter().map(|x| x.count_ones()).sum::<u32>() as u16,
            pellets,
            walls: INIT_WALLS,
            seed,
        };
        s.plan_all_ghosts();
        Ok(s)
    }

    #[cfg(feature = "std")]
    pub fn to_bytes(&self) -> Vec<u8> {
        const FRUIT_DURATION: u8 = 30;

        let mut b = vec![];

        b.append(&mut (self.curr_ticks as u16).to_be_bytes().to_vec());
        b.push(self.update_period);
        b.push(match (self.mode, self.paused) {
            (_, true) => 0,
            (GameMode::SCATTER, false) => 1,
            (GameMode::CHASE, false) => 2,
        });
        b.push(self.mode_steps);
        // mode_duration
        b.push(match (self.mode, self.paused) {
            (GameMode::SCATTER, false) => GameMode::SCATTER.duration(),
            (GameMode::CHASE, false) => GameMode::CHASE.duration(),
            _ => 255,
        });
        b.append(&mut self.curr_score.to_be_bytes().to_vec());
        b.push(self.curr_level);
        b.push(self.curr_lives);

        // ghost info
        for g in 0..4 {
            let ghost = &self.ghosts[g];
            b.append(&mut ghost.loc.to_bytes().to_vec());
            b.push(ghost.get_aux());
        }

        // pacman location
        b.append(&mut self.pacman_loc.to_bytes().to_vec());

        // fruit location info
        b.append(&mut self.fruit_loc.to_bytes().to_vec());
        b.push(self.fruit_steps);
        b.push(FRUIT_DURATION);

        // pellet info
        for i in 0..31 {
            b.append(&mut self.pellets[i].to_be_bytes().to_vec());
        }

        b
    }

    /// Start the game engine
    pub fn step(&mut self) {
        let lives_before = self.curr_lives;
        self.next_tick();
        if self.update_ready() {
            self.update_all_ghosts();
            self.try_respawn_pacman();
            self.check_collisions();
            self.handle_step_events();
        }
        if self.update_ready() {
            self.plan_all_ghosts();
        }
        if self.curr_lives < lives_before {
            self.paused = true;
        }
    }

    #[cfg(feature = "std")]
    /// Set pacman's location
    pub fn set_pacman_location(&mut self, location: Position) {
        // Check if there is a wall at the anticipated location, and return if so
        if !self.in_bounds(location) || self.wall_at(location) {
            return;
        }
        let likely_path = self.bfs((self.pacman_loc.row, self.pacman_loc.col), location);
        if let Some(likely_path) = likely_path {
            for LocationState { dir, .. } in likely_path {
                self.move_pacman_dir(dir);
            }
            self.collect_pellet(location);
            self.check_collisions();
        } else {
            eprintln!("Could not find path to position!");
        }
    }

    #[cfg(feature = "std")]
    pub fn bfs(&self, start: Position, end: Position) -> Option<Vec<LocationState>> {
        let mut visited: HashMap<Position, Option<(Position, Direction)>> = HashMap::new();
        let mut queue: VecDeque<Position> = VecDeque::new();
        queue.push_back(start);
        visited.insert(start, None);
        while let Some(pos) = queue.pop_front() {
            let loc = LocationState {
                row: pos.0,
                col: pos.1,
                dir: Direction::Stay,
            };
            for dir in Direction::all_except_stay() {
                let next = loc.get_neighbor_coords(dir);
                if !self.wall_at(next) && !visited.contains_key(&next) {
                    if next == end {
                        let mut path = vec![LocationState {
                            row: pos.0,
                            col: pos.1,
                            dir,
                        }];
                        let mut node = pos;
                        while let Some(Some((previous, dir))) = visited.remove(&node) {
                            node = previous;
                            path.insert(
                                0,
                                LocationState {
                                    row: previous.0,
                                    col: previous.1,
                                    dir,
                                },
                            );
                        }
                        return Some(path);
                    }
                    visited.insert(next, Some((pos, dir)));
                    queue.push_back(next);
                }
            }
        }
        None
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
        #[cfg(feature = "std")]
        if std::env::var("DISABLE_PACMAN_LOGGING").is_ok() {
            println!(
                "\x1b[36mGAME: Update period changed ({} -> {}) (t = {})\x1b[0m\n",
                self.get_update_period(),
                period,
                self.curr_ticks,
            );
        }

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
        #[cfg(feature = "std")]
        if std::env::var("DISABLE_PACMAN_LOGGING").is_ok() {
            println!(
                "\x1b[32mGAME: Next level ({} -> {}) (t = {})\x1b[0m\n",
                level,
                level + 1,
                self.curr_ticks,
            );
        }

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
        #[cfg(feature = "std")]
        if std::env::var("DISABLE_PACMAN_LOGGING").is_ok() {
            println!(
                "\x1b[36mGAME: Lives changed ({} -> {})\x1b[0m\n",
                self.get_lives(),
                lives,
            );
        }

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
        #[cfg(feature = "std")]
        if std::env::var("DISABLE_PACMAN_LOGGING").is_ok() {
            println!(
                "\x1b[31mGAME: Pacman lost a life ({} -> {}) (t = {})\x1b[0m\n",
                lives,
                lives - 1,
                self.curr_ticks,
            );
        }

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
            #[cfg(feature = "std")]
            if std::env::var("DISABLE_PACMAN_LOGGING").is_ok() {
                println!("\x1b[31mGAME: Long-game penalty applied\x1b[0m");
            }

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
