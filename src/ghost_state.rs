use crate::location::LocationState;

// Enum-like declaration to hold the ghost colors
const RED: u8 = 0;
const PINK: u8 = 1;
const CYAN: u8 = 2;
const ORANGE: u8 = 3;
const NUM_COLORS: u8 = 4;

/*
The number of "active" ghosts (the others are invisible and don't affect
the progression of the game)
*/
const NUM_ACTIVE_GHOSTS: u8 = 4;

// Configure the number of active ghosts
fn config_num_active_ghosts(num_active_ghosts: u8) {
    NUM_ACTIVE_GHOSTS = num_active_ghosts;
}

// Names of the ghosts (not the nicknames, just the colors, for debugging)
// var ghostNames [numColors]string = [...]string{
// 	"red",
// 	"pink",
// 	"cyan",
// 	"orange",
// }

/*
An object to keep track of the location and attributes of a ghost
*/
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

impl GhostState {
    // Create a new ghost state with given location and color values
    pub fn new(color: u8) -> Self {
        // Ghost state object
        let mut g = Self {
            loc: new_location_state_copy(empty_loc),
            next_loc: new_location_state_copy(ghost_spawn_locs[color]),
            scatter_target: new_location_state_copy(ghost_scatter_targets[color]),
            color,
            trapped_steps: ghost_trapped_steps[color],
            fright_steps: 0,
            spawning: true,
            eaten: false,
        };

        // If the color is greater than the number of active ghosts, hide this ghost
        if color >= NUM_ACTIVE_GHOSTS {
            g.next_loc = new_location_state_copy(empty_loc);
        }

        // Return the ghost state
        new
    }

    /*************************** Ghost Frightened State ***************************/

    // Set the fright steps of a ghost
    pub fn set_fright_steps(steps: u8) {
        g.frightSteps = steps;
    }

    // Decrement the fright steps of a ghost
    pub fn dec_fright_steps() {
        g.frightSteps -= 1;
    }

    // Get the fright steps of a ghost
    pub fn get_fright_steps() -> u8 {
        g.frightSteps
    }

    // Check if a ghost is frightened
    pub fn is_frightened() -> bool {
        // Return whether there is at least one fright step left
        g.frightSteps > 0
    }

    /****************************** Ghost Trap State ******************************/

    // Set the trapped steps of a ghost
    pub fn set_trapped_steps(steps: u8) {
        g.trapped_steps = steps;
    }

    // Decrement the trapped steps of a ghost
    pub fn dec_trapped_steps() {
        g.trapped_steps -= 1;
    }

    // Check if a ghost is trapped
    pub fn is_trapped() -> bool {
        // Return whether there is at least one fright step left
        g.trapped_steps > 0
    }

    /**************************** Ghost Spawning State ****************************/

    // Set the ghost spawning flag
    pub fn set_spawning(spawning: bool) {
        g.spawning = spawning;
    }

    /// Check if a ghost is spawning.
    pub fn is_spawning() -> bool {
        g.spawning
    }

    /****************************** Ghost Eaten Flag ******************************/

    /// Set the ghost eaten flag.
    pub fn set_eaten(eaten: bool) {
        g.eaten = eaten;
    }

    /// Check if a ghost is eaten.
    pub fn is_eaten() -> bool {
        g.eaten
    }
}
