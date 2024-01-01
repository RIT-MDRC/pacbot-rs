use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::{
    location::LocationState,
    variables::{EMPTY_LOC, GHOST_SCATTER_TARGETS, GHOST_SPAWN_LOCS, GHOST_TRAPPED_STEPS},
};

// Enum-like declaration to hold the ghost colors
pub const RED: u8 = 0;
pub const PINK: u8 = 1;
pub const CYAN: u8 = 2;
pub const ORANGE: u8 = 3;
pub const NUM_COLORS: u8 = 4;

// Names of the ghosts (not the nicknames, just the colors, for debugging)
pub const GHOST_NAMES: [&str; NUM_COLORS as usize] = ["red", "pink", "cyan", "orange"];

/*
An object to keep track of the location and attributes of a ghost
*/
#[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct GhostState {
    pub loc: LocationState,            // Current location
    pub next_loc: LocationState,       // Planned location (for next update)
    pub scatter_target: LocationState, // Position of (fixed) scatter target
    pub color: u8,
    pub trapped_steps: u8,
    pub fright_steps: u8,
    pub spawning: bool, // Flag set when spawning
    pub eaten: bool,    // Flag set when eaten and returning to ghost house
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArcMutexGhost {
    pub ghost: Arc<Mutex<GhostState>>,
}

impl PartialOrd for ArcMutexGhost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ghost.lock().unwrap().loc.partial_cmp(&other.ghost.lock().unwrap().loc)
    }
}

impl PartialEq for ArcMutexGhost {
    fn eq(&self, other: &Self) -> bool {
        self.ghost.lock().unwrap().loc == other.ghost.lock().unwrap().loc
    }
}

impl Ord for ArcMutexGhost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ghost.lock().unwrap().loc.cmp(&other.ghost.lock().unwrap().loc)
    }
}

impl Eq for ArcMutexGhost {}

impl GhostState {
    // Create a new ghost state with given location and color values
    pub fn new(color: u8) -> Self {
        Self {
            loc: EMPTY_LOC,
            next_loc: GHOST_SPAWN_LOCS[color as usize],
            scatter_target: GHOST_SCATTER_TARGETS[color as usize],
            color,
            trapped_steps: GHOST_TRAPPED_STEPS[color as usize],
            fright_steps: 0,
            spawning: true,
            eaten: false,
        }
    }

    /*************************** Ghost Frightened State ***************************/

    // Set the fright steps of a ghost
    pub fn set_fright_steps(&mut self, steps: u8) {
        self.fright_steps = steps;
    }

    // Decrement the fright steps of a ghost
    pub fn dec_fright_steps(&mut self) {
        self.fright_steps -= 1;
    }

    // Check if a ghost is frightened
    pub fn is_frightened(&self) -> bool {
        // Return whether there is at least one fright step left
        self.fright_steps > 0
    }

    /****************************** Ghost Trap State ******************************/

    // Set the trapped steps of a ghost
    pub fn set_trapped_steps(&mut self, steps: u8) {
        self.trapped_steps = steps;
    }

    // Decrement the trapped steps of a ghost
    pub fn dec_trapped_steps(&mut self) {
        self.trapped_steps -= 1;
    }

    // Check if a ghost is trapped
    pub fn is_trapped(&self) -> bool {
        // Return whether there is at least one fright step left
        self.trapped_steps > 0
    }

    /**************************** Ghost Spawning State ****************************/

    // Set the ghost spawning flag
    pub fn set_spawning(&mut self, spawning: bool) {
        self.spawning = spawning;
    }

    /// Check if a ghost is spawning.
    pub fn is_spawning(&self) -> bool {
        self.spawning
    }

    /****************************** Ghost Eaten Flag ******************************/

    /// Set the ghost eaten flag.
    pub fn set_eaten(&mut self, eaten: bool) {
        self.eaten = eaten;
    }

    /// Check if a ghost is eaten.
    pub fn is_eaten(&self) -> bool {
        self.eaten
    }
}
