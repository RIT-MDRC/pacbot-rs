use rand::seq::IteratorRandom;

use crate::{
    game_modes::GameMode,
    game_state::GameState,
    ghost_state::{GhostState, PINK, RED},
    location::{dist_sq, DOWN, NUM_DIRS, UP},
    variables::{EMPTY_LOC, GHOST_HOUSE_EXIT_POS, GHOST_SPAWN_LOCS, GHOST_TRAPPED_STEPS},
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
        let spawn_loc_color = if self.color == RED { PINK } else { self.color };
        self.next_loc = GHOST_SPAWN_LOCS[spawn_loc_color as usize];
        self.next_loc.dir = UP;
    }

    /******************** Ghost Updates (before serialization) ********************/

    /// Update the ghost's position
    pub fn update(&mut self) {
        // If the ghost is at the red spawn point and not moving downwards,
        // we can mark it as done spawning.
        if self.loc.collides_with(GHOST_SPAWN_LOCS[RED as usize]) && self.loc.dir != DOWN {
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

    /******************** Ghost Planning (after serialization) ********************/

    /// Plan the ghost's next move
    pub fn plan(&mut self, game_state: &GameState) {
        // If the location is empty (i.e. after a reset/respawn), don't plan
        if self.loc.is_empty() {
            return;
        }

        // Determine the next position based on the current direction
        self.next_loc.advance_from(self.loc);

        // If the ghost is trapped, reverse the current direction and return
        if self.is_trapped() {
            self.next_loc.dir = self.next_loc.get_reversed_dir();
            self.dec_trapped_steps();
            return;
        }

        // Decide on a target for this ghost, depending on the game mode.
        /*
            If the ghost is spawning in the ghost house, choose red's spawn
            location as the target to encourage it to leave the ghost house.

            Otherwise: pick chase or scatter targets, depending on the mode.
        */
        let target_loc = if self.spawning
            && !self.loc.collides_with(GHOST_SPAWN_LOCS[RED as usize])
            && !self.next_loc.collides_with(GHOST_SPAWN_LOCS[RED as usize])
        {
            GHOST_SPAWN_LOCS[RED as usize].get_coords()
        } else {
            match game_state.mode {
                GameMode::CHASE => game_state.get_chase_target(self.color),
                GameMode::SCATTER => self.scatter_target.get_coords(),
            }
        };

        // Determine which of the four neighboring moves to the next location are valid.
        let moves = (0..NUM_DIRS as u8).map(|dir| (dir, self.next_loc.get_neighbor_coords(dir)));
        let valid_moves = moves.filter(|&(dir, loc)| {
            // If this move would make the ghost reverse, skip it.
            if dir == self.next_loc.get_reversed_dir() {
                return false;
            }

            // Considerations when the ghost is spawning.
            if self.spawning {
                // Determine if the move would be within the ghost house.
                if game_state.ghost_spawn_at(loc) {
                    return true;
                }

                // Determine if the move would help the ghost escape the ghost house,
                // and make it a valid one if so.
                if loc == GHOST_HOUSE_EXIT_POS {
                    return true;
                }
            }

            // Otherwise, the move is valid if it does not move into a wall.
            !game_state.wall_at(loc)
        });

        let chosen_move = if self.fright_steps > 1 {
            // If the ghost will still be frightened one tick later, immediately choose
            // a random valid direction and return.
            valid_moves.choose(&mut rand::thread_rng())
        } else {
            // Otherwise, pick the move that takes the ghost closest to its target.
            valid_moves.max_by_key(|&(_dir, loc)| dist_sq(loc, target_loc))
        }
        .expect("ghost has no valid moves!");

        // Once we have picked a move, set next_loc.dir to that direction.
        self.next_loc.dir = chosen_move.0;
    }
}
