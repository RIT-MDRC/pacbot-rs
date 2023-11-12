type Position = (i8, i8);

/***************************** Bitwise Operations *****************************/

use crate::{
    game_modes::GameMode, game_state::GameState, ghost_state::*, location::*, variables::*,
};

fn get_bit_u32(num: u32, bit_idx: usize) -> bool {
    ((num >> bit_idx) & 1) == 1
}

fn modify_bit_u32(num: &mut u32, bit_idx: usize, bit_val: bool) {
    // If the bit is true, we should set the bit, otherwise we clear it
    if bit_val {
        *num |= 1 << bit_idx;
    } else {
        *num &= !(1 << bit_idx);
    }
}

/****************************** Timing Functions ******************************/

impl GameState {
    // Determines if the game state is ready to update
    pub fn updateReady(&self) -> bool {
        let update_period: u32 = self.getUpdatePeriod().into();

        // Update if the update period divides the current ticks
        self.currTicks % update_period == 0
    }

    /**************************** Positional Functions ****************************/

    // Determines if a position is within the bounds of the maze
    fn inBounds(&self, pos: Position) -> bool {
        let (row, col) = pos;
        (row >= 0 && row < MAZE_ROWS as i8) && (col >= 0 && col < MAZE_COLS as i8)
    }

    // Determines if a pellet is at a given location
    fn pelletAt(&self, pos: Position) -> bool {
        let (row, col) = pos;
        if !self.inBounds(pos) {
            return false;
        }

        // Returns the bit of the pellet row corresponding to the column
        get_bit_u32(self.pellets[row as usize], col as usize)
    }

    /*
    Collects a pellet if it is at a given location
    Returns the number of pellets that are left
    */
    fn collectPellet(&mut self, pos: Position) {
        let (row, col) = pos;

        // Collect fruit, if applicable
        if self.fruitExists() && self.pacmanLoc.collides_with(self.fruitLoc) {
            self.setFruitSteps(0);
            self.incrementScore(FRUIT_POINTS);
        }

        // If there's no pellet, return
        if !self.pelletAt(pos) {
            return;
        }

        // If we can clear the pellet's bit, decrease the number of pellets
        modify_bit_u32(&mut self.pellets[row as usize], col as usize, false);
        self.decrementNumPellets();

        // If the we are in particular rows and columns, it is a super pellet
        let superPellet = ((row == 3) || (row == 23)) && ((col == 1) || (col == 26));

        // Make all the ghosts frightened if a super pellet is collected
        if superPellet {
            self.frightenAllGhosts();
        }

        // Update the score, depending on the pellet type
        if superPellet {
            self.incrementScore(SUPER_PELLET_POINTS);
        } else {
            self.incrementScore(PELLET_POINTS);
        }

        // Act depending on the number of pellets left over
        let numPellets = self.getNumPellets();

        // Spawn fruit, if applicable
        if !self.fruitExists() && (numPellets == FRUIT_THRESHOLD1 || numPellets == FRUIT_THRESHOLD2)
        {
            self.setFruitSteps(FRUIT_DURATION);
        }

        // Other pellet-related events
        if numPellets == ANGER_THRESHOLD1 || numPellets == ANGER_THRESHOLD2 {
            // Ghosts get angry (speeding up)
            self.setUpdatePeriod(u8::max(1, self.getUpdatePeriod().saturating_sub(2)));
            self.mode = GameMode::CHASE;
            self.set_mode_steps(GameMode::CHASE.duration());
        } else if numPellets == 0 {
            self.levelReset();
            self.incrementLevel();
        }
    }

    // Determines if a wall is at a given location
    pub fn wallAt(&self, pos: Position) -> bool {
        if !self.inBounds(pos) {
            return true;
        }

        // Returns the bit of the wall row corresponding to the column
        let (row, col) = pos;
        get_bit_u32(self.walls[row as usize], col as usize)
    }

    // Determines if the ghost house is at a given location
    pub fn ghostSpawnAt(&self, pos: Position) -> bool {
        let (row, col) = pos;
        (13..=14).contains(&row) && (11..=15).contains(&col)
    }

    /***************************** Collision Handling *****************************/

    // Check collisions between Pacman and all the ghosts, and respawn ghosts/Pacman as necessary.
    pub fn checkCollisions(&mut self) {
        // Loop over all the ghosts and check for collisions with Pacman.
        let mut num_ghosts_eaten = 0;
        let mut did_pacman_die = false;
        for mut ghost in self.ghosts_mut() {
            if self.pacmanLoc.collides_with(ghost.loc) {
                // If the ghost was already eaten, skip it.
                if ghost.is_eaten() {
                    continue;
                }

                // If the ghost is frightened, Pacman eats it, otherwise Pacman dies.
                if ghost.is_frightened() {
                    // Respawn the ghost.
                    ghost.respawn();

                    num_ghosts_eaten += 1;
                } else {
                    did_pacman_die = true;
                    break;
                }
            }
        }

        if did_pacman_die {
            self.deathReset();
        } else {
            for _ in 0..num_ghosts_eaten {
                // Add points corresponding to the current combo length.
                self.incrementScore(COMBO_MULTIPLIER << self.ghostCombo);

                // Increment the ghost respawn combo.
                self.ghostCombo += 1;
            }
        }
    }

    /***************************** Event-Based Resets *****************************/

    // Reset the board (while leaving pellets alone) after Pacman dies
    fn deathReset(&mut self) {
        // Set Pacman to be in an empty state
        self.pacmanLoc = EMPTY_LOC;

        // Decrease the number of lives Pacman has left
        self.decrementLives();

        /*
            If the mode is not the initial mode and the ghosts aren't angry,
            change the mode back to the initial mode
        */
        if self.getNumPellets() > ANGER_THRESHOLD1 {
            self.mode = INIT_MODE;
            self.set_mode_steps(INIT_MODE.duration());
        }

        // Set the fruit steps back to 0
        self.setFruitSteps(0);

        // Reset all the ghosts to their original locations
        self.resetAllGhosts();
    }

    // Reset the board (including pellets) after Pacman clears a level
    fn levelReset(&mut self) {
        // Set Pacman to be in an empty state
        self.pacmanLoc = EMPTY_LOC;

        // If the mode is not the initial mode, change it
        self.mode = INIT_MODE;
        self.set_mode_steps(INIT_MODE.duration());

        // Reset the level penalty
        self.setLevelSteps(LEVEL_DURATION);

        // Set the fruit steps back to 0
        self.setFruitSteps(0);

        // Reset all the ghosts to their original locations
        self.resetAllGhosts();

        // Reset the pellet bit array and count
        self.resetPellets();
    }

    /************************** Motion (Pacman Location) **************************/

    // Move Pacman one space in a given direction
    pub fn movePacmanDir(&mut self, dir: u8) {
        // Check collisions with all the ghosts
        self.checkCollisions();

        // Calculate the next row and column
        let next_loc = self.pacmanLoc.get_neighbor_coords(dir);

        // Update Pacman's direction
        self.pacmanLoc.dir = dir;

        // Check if there is a wall at the anticipated location, and return if so
        if self.wallAt(next_loc) {
            return;
        }

        // Move Pacman the anticipated spot
        self.pacmanLoc.update_coords(next_loc);
        self.collectPellet(next_loc);
    }

    // Move Pacman back to its spawn point, if necessary
    pub fn tryRespawnPacman(&mut self) {
        // Set Pacman to be in its original state
        if self.pacmanLoc.is_empty() && self.getLives() > 0 {
            self.pacmanLoc = PACMAN_SPAWN_LOC;
        }
    }

    /******************************* Ghost Movement *******************************/

    // Frighten all ghosts at once
    fn frightenAllGhosts(&mut self) {
        // Reset the ghost respawn combo back to 0
        self.ghostCombo = 0;

        // Loop over all the ghosts
        for mut ghost in self.ghosts_mut() {
            /*
                To frighten a ghost, set its fright steps to a specified value
                and trap it for one step (to force the direction to reverse)
            */
            ghost.set_fright_steps(GHOST_FRIGHT_STEPS);
            if !ghost.is_trapped() {
                ghost.set_trapped_steps(1);
            }
        }
    }

    // Reverse all ghosts at once (similar to frightenAllGhosts)
    pub fn reverseAllGhosts(&mut self) {
        // Loop over all the ghosts
        for mut ghost in self.ghosts_mut() {
            /*
                To change the direction a ghost, trap it for one step
                (to force the direction to reverse)
            */
            if !ghost.is_trapped() {
                ghost.set_trapped_steps(1);
            }
        }
    }

    // Reset all ghosts at once
    fn resetAllGhosts(&mut self) {
        // Reset the ghost respawn combo back to 0
        self.ghostCombo = 0;

        // Reset each of the ghosts
        for mut ghost in self.ghosts_mut() {
            ghost.reset();
        }

        // If no lives are left, set all ghosts to stare at the player, menacingly
        if self.getLives() == 0 {
            for mut ghost in self.ghosts_mut() {
                if ghost.color != ORANGE {
                    ghost.next_loc.dir = NONE;
                } else {
                    // Orange does like making eye contact, unfortunately
                    ghost.next_loc.dir = LEFT;
                }
            }
        }
    }

    // Update all ghosts at once
    pub fn updateAllGhosts(&mut self) {
        // Loop over the individual ghosts
        for mut ghost in self.ghosts_mut() {
            ghost.update();
        }
    }

    // A game state function to plan all ghosts at once
    pub fn planAllGhosts(&mut self) {
        // Plan each ghost's next move concurrently
        for mut ghost in self.ghosts_mut() {
            ghost.plan(self);
        }
    }

    /************************ Ghost Targeting (Chase Mode) ************************/

    /*
    Returns the chase location of the red ghost
    (i.e. Pacman's exact location)
    */
    fn getChaseTargetRed(&self) -> Position {
        // Return Pacman's current location
        self.pacmanLoc.get_coords()
    }

    /*
    Returns the chase location of the pink ghost
    (i.e. 4 spaces ahead of Pacman's location)
    */
    fn getChaseTargetPink(&self) -> Position {
        // Return the red pink's target (4 spaces ahead of Pacman)
        self.pacmanLoc.get_ahead_coords(4)
    }

    /*
    Returns the chase location of the cyan ghost
    (i.e. The red ghost's location, reflected about 2 spaces ahead of Pacman)
    */
    fn getChaseTargetCyan(&self) -> Position {
        // Get the 'pivot' square, 2 steps ahead of Pacman
        let (pivotRow, pivotCol) = self.pacmanLoc.get_ahead_coords(2);

        // Get the current location of the red ghost
        let red_ghost = self.ghosts[RED as usize].borrow();
        let (redRow, redCol) = red_ghost.loc.get_coords();

        // Return the pair of coordinates of the calculated target
        ((2 * pivotRow - redRow), (2 * pivotCol - redCol))
    }

    /*
    Returns the chase location of the orange ghost
    (i.e. Pacman's exact location, the same as red's target most of the time)
    Though, if close enough to Pacman, it should choose its scatter target
    */
    fn getChaseTargetOrange(&self) -> Position {
        // Get Pacman's current location
        let pacman_pos = self.pacmanLoc.get_coords();

        // Get the orange ghost's current location
        let orange_ghost = self.ghosts[ORANGE as usize].borrow();
        let orange_pos = orange_ghost.loc.get_coords();

        // If Pacman is far enough from the ghost, return Pacman's location
        if dist_sq(orange_pos, pacman_pos) >= 64 {
            return pacman_pos;
        }

        // Otherwise, return the scatter location of orange
        orange_ghost.scatter_target.get_coords()
    }

    // Returns the chase location of an arbitrary ghost color
    pub fn getChaseTarget(&self, color: u8) -> Position {
        match color {
            RED => self.getChaseTargetRed(),
            PINK => self.getChaseTargetPink(),
            CYAN => self.getChaseTargetCyan(),
            ORANGE => self.getChaseTargetOrange(),
            _ => unreachable!(), // TODO: convert color to a proper enum
        }
    }
}
