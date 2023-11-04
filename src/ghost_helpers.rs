use rand::Rng;

use crate::{
    game_modes::{CHASE, SCATTER},
    ghost_state::{GhostState, GHOST_NAMES, PINK, RED},
    location::{DIR_NAMES, DOWN, NUM_DIRS, UP},
    variables::{
        EMPTY_LOC, GHOST_HOUSE_EXIT_COL, GHOST_HOUSE_EXIT_ROW, GHOST_SPAWN_LOCS,
        GHOST_TRAPPED_STEPS,
    },
};

impl GhostState {
    /******************************** Ghost Resets ********************************/

    /// Respawn the ghost
    pub fn reset(&self) {
        // Set the ghost to be trapped, spawning, and not frightened
        self.set_spawning(true);
        self.set_trapped_steps(GHOST_TRAPPED_STEPS[self.color]);
        self.set_fright_steps(0);

        // Set the current ghost to be at an empty location
        self.loc = EMPTY_LOC;

        /*
            Set the current location of the ghost to be its spawn point
            (or pink's spawn location, in the case of red, so it spawns in the box)
        */
        self.next_loc = GHOST_SPAWN_LOCS[self.color as usize];
    }

    /****************************** Ghost Respawning ******************************/

    /// Respawn the ghost
    pub fn respawn(&self) {
        // Set the ghost to be eaten and spawning
        self.set_spawning(true);
        self.set_eaten(true);

        // Set the current ghost to be at an empty location
        self.loc = EMPTY_LOC;

        /*
            Set the current location of the ghost to be its spawn point
            (or pink's spawn location, in the case of red, so it spawns in the box)
        */
        if self.color == RED {
            self.next_loc
                .update_coords(GHOST_SPAWN_LOCS[PINK as usize].get_coords())
        } else {
            self.next_loc
                .update_coords(GHOST_SPAWN_LOCS[self.color as usize].get_coords())
        }
        self.next_loc.dir = UP;
    }

    /******************** Ghost Updates (before serialization) ********************/

    /// Update the ghost's position
    pub fn update(&self) {
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
    pub fn plan(&self) {
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

        // Keep local copies of the fright steps and spawning variables
        let frightSteps = self.get_fright_steps();
        let spawning = self.is_spawning();

        // Capture the last unpaused current game mode (could be the current mode)
        let mode = self.game.getLastUnpausedMode();

        // Decide on a target for this ghost, depending on the game mode.
        /*
            If the ghost is spawning in the ghost house, choose red's spawn
            location as the target to encourage it to leave the ghost house

            Otherwise: pick chase or scatter targets, depending on the mode
        */
        let (targetRow, targetCol) = if spawning
            && !self.loc.collides_with(GHOST_SPAWN_LOCS[RED as usize])
            && !self.next_loc.collides_with(GHOST_SPAWN_LOCS[RED as usize])
        {
            GHOST_SPAWN_LOCS[RED as usize].get_coords()
        } else {
            match mode {
                CHASE => self.game.getChaseTarget(self.color),
                SCATTER => self.scatter_target.get_coords(),
                _ => unreachable!(),
            }
        };

        /*
            Determine whether each of the four neighboring moves to the next
            location is valid, and count how many are good
        */
        let numValidMoves = 0;
        let moveValid = [false; NUM_DIRS];
        let moveDistSq = [0; NUM_DIRS];
        for dir in 0..NUM_DIRS {
            // Get the neighboring cell in that location
            let (row, col) = self.next_loc.get_neighbor_coords(dir as u8);

            // Calculate the distance from the target to the move location
            moveDistSq[dir] = self.game.distSq(row, col, targetRow, targetCol);

            // Determine if that move is valid
            moveValid[dir] = !self.game.wallAt(row, col);

            // Considerations when the ghost is spawning
            if spawning {
                // Determine if the move would be within the ghost house
                if self.game.ghostSpawnAt(row, col) {
                    moveValid[dir] = true;
                }

                // Determine if the move would help the ghost escape the ghost house,
                // and make it a valid one if so
                if row == GHOST_HOUSE_EXIT_ROW && col == GHOST_HOUSE_EXIT_COL {
                    moveValid[dir] = true;
                }
            }

            // If this move would make the ghost reverse, skip it
            if dir as u8 == self.next_loc.get_reversed_dir() {
                moveValid[dir] = false;
            }

            // Increment the valid moves counter if necessary
            if moveValid[dir] {
                numValidMoves += 1;
            }
        }

        // Debug statement, in case a ghost somehow is surrounded by all walls
        if numValidMoves == 0 {
            let (row, col) = self.next_loc.get_coords();
            let dir = self.next_loc.get_dir();
            println!(
                "\x1b[2m\x1b[36mWARN: {} has nowhere to go (row = {row}, col = {col}, dir = {}, spawning = {spawning})\n\x1b[0m",
                GHOST_NAMES[self.color as usize], DIR_NAMES[dir as usize]
            );
            return;
        }

        // If the ghost will still frightened one tick later, immediately choose
        // a random valid direction and return
        if frightSteps > 1 {
            // Generate a random index out of the valid moves
            let randomNum = rand::thread_rng().gen_range(0..numValidMoves);

            // Loop over all directions
            let mut count = 0;
            for dir in 0..NUM_DIRS {
                // Skip any invalid moves
                if !moveValid[dir] {
                    continue;
                }

                // If we have reached the correct move, update the direction and return
                if count == randomNum {
                    self.next_loc.dir = dir as u8;
                    return;
                }

                // Update the count of valid moves so far
                count += 1;
            }
        }

        let bestDir = (0..NUM_DIRS)
            .filter(|&dir| moveValid[dir])
            .max_by_key(|&dir| moveDistSq[dir])
            .unwrap() as u8;

        // Once we have picked the best direction, update it
        self.next_loc.dir = bestDir;
    }
}
