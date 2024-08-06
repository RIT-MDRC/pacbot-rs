use crate::location::Direction;
use crate::{
    ghost_state::GhostColor::*,
    ghost_state::GhostState,
    variables::{EMPTY_LOC, GHOST_SPAWN_LOCS, GHOST_TRAPPED_STEPS},
};

impl GhostState {
    /******************************** Ghost Resets ********************************/

    /// Respawn the ghost
    pub fn reset(&mut self) {
        // Set the ghost to be trapped, spawning, and not frightened
        self.set_spawning(true);
        self.set_eaten(false);
        self.set_trapped_steps(GHOST_TRAPPED_STEPS[self.color as usize]);
        self.set_fright_steps(0);

        // Set the current ghost to be at an empty location
        self.loc = EMPTY_LOC;

        // Set the current location of the ghost to be its spawn point
        self.next_loc = GHOST_SPAWN_LOCS[self.color as usize];
    }

    /****************************** Ghost Respawning ******************************/

    /// Respawn the ghost
    pub fn respawn(&mut self) {
        // Set the ghost to be eaten and spawning
        self.set_spawning(true);
        self.set_eaten(true);

        // Set the current ghost to be at an empty location
        self.loc = EMPTY_LOC;

        // Set the current location of the ghost to be its spawn point
        // (or pink's spawn location, in the case of red, so it spawns in the box)
        let spawn_loc_color = if self.color == Red { Pink } else { self.color };
        self.next_loc = GHOST_SPAWN_LOCS[spawn_loc_color as usize];
        self.next_loc.dir = Direction::Up;
    }

    /******************** Ghost Updates (before serialization) ********************/

    /// Update the ghost's position
    pub fn update(&mut self) {
        // If the ghost is at the red spawn point and not moving downwards,
        // we can mark it as done spawning.
        if self.loc.collides_with(GHOST_SPAWN_LOCS[Red as usize]) && self.loc.dir != Direction::Down
        {
            self.set_spawning(false);
        }

        // Set the ghost to be no longer eaten, if applicable
        if self.is_eaten() {
            self.set_eaten(false);
            self.set_fright_steps(0);
        }

        // Decrement the ghost's frightened steps count if necessary
        if self.is_frightened() {
            self.dec_fright_steps();
        }

        // Copy the next location into the current location
        self.loc = self.next_loc;
    }
}
