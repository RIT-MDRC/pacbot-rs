use rand::prelude::SmallRng;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};

use crate::ghost_state::GhostColor;
use crate::location::Direction::*;
use crate::{
    game_modes::GameMode, game_state::GameState, ghost_state::GhostColor::*, location::*,
    variables::*,
};

pub type Position = (i8, i8);

/***************************** Bitwise Operations *****************************/

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
    pub fn update_ready(&self) -> bool {
        let update_period: u32 = self.get_update_period().into();

        // Update if the update period divides the current ticks
        self.curr_ticks % update_period == 0
    }

    /**************************** Positional Functions ****************************/

    // Determines if a position is within the bounds of the maze
    pub fn in_bounds(&self, pos: Position) -> bool {
        let (row, col) = pos;
        (row >= 0 && row < MAZE_ROWS as i8) && (col >= 0 && col < MAZE_COLS as i8)
    }

    // Determines if a pellet is at a given location
    pub fn pellet_at(&self, pos: Position) -> bool {
        let (row, col) = pos;
        if !self.in_bounds(pos) {
            return false;
        }

        // Returns the bit of the pellet row corresponding to the column
        get_bit_u32(self.pellets[row as usize], col as usize)
    }

    /*
    Collects a pellet if it is at a given location
    */
    pub fn collect_pellet(&mut self, pos: Position) {
        let (row, col) = pos;

        // Collect fruit, if applicable
        if self.fruit_exists() && self.pacman_loc.collides_with(self.fruit_loc) {
            self.set_fruit_steps(0);
            self.increment_score(FRUIT_POINTS);
        }

        // If there's no pellet, return
        if !self.pellet_at(pos) {
            return;
        }

        // If we can clear the pellet's bit, decrease the number of pellets
        modify_bit_u32(&mut self.pellets[row as usize], col as usize, false);
        self.decrement_num_pellets();

        // If the we are in particular rows and columns, it is a super pellet
        let super_pellet = is_super_pellet((row, col));

        // Make all the ghosts frightened if a super pellet is collected
        if super_pellet {
            self.frighten_all_ghosts();
        }

        // Update the score, depending on the pellet type
        if super_pellet {
            self.increment_score(SUPER_PELLET_POINTS);
        } else {
            self.increment_score(PELLET_POINTS);
        }

        // Act depending on the number of pellets left over
        let num_pellets = self.get_num_pellets();

        // Spawn fruit, if applicable
        if !self.fruit_exists()
            && (num_pellets == FRUIT_THRESHOLD1 || num_pellets == FRUIT_THRESHOLD2)
        {
            self.set_fruit_steps(FRUIT_DURATION);
        }

        // Other pellet-related events
        if num_pellets == ANGER_THRESHOLD1 || num_pellets == ANGER_THRESHOLD2 {
            // Ghosts get angry (speeding up)
            self.set_update_period(u8::max(1, self.get_update_period().saturating_sub(2)));
            self.mode = GameMode::CHASE;
            self.set_mode_steps(GameMode::CHASE.duration());
        } else if num_pellets == 0 {
            self.level_reset();
            self.increment_level();
        }
    }

    // Determines if a wall is at a given location
    pub fn wall_at(&self, pos: Position) -> bool {
        if !self.in_bounds(pos) {
            return true;
        }

        // Returns the bit of the wall row corresponding to the column
        let (row, col) = pos;
        get_bit_u32(self.walls[row as usize], col as usize)
    }

    // Determines if the ghost house is at a given location
    pub fn ghost_spawn_at(&self, pos: Position) -> bool {
        let (row, col) = pos;
        (13..=14).contains(&row) && (11..=15).contains(&col)
    }

    /***************************** Collision Handling *****************************/

    // Check collisions between Pacman and all the ghosts, and respawn ghosts/Pacman as necessary.
    pub fn check_collisions(&mut self) {
        // Loop over all the ghosts and check for collisions with Pacman.
        let mut num_ghosts_eaten = 0;
        let mut did_pacman_die = false;
        let pacman_loc = self.pacman_loc;
        for ghost in &mut self.ghosts {
            if pacman_loc.collides_with(ghost.loc) {
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
            self.death_reset();
        } else {
            for _ in 0..num_ghosts_eaten {
                // Add points corresponding to the current combo length.
                self.increment_score(COMBO_MULTIPLIER << self.ghost_combo);

                // Increment the ghost respawn combo.
                self.ghost_combo += 1;
            }
        }
    }

    /***************************** Event-Based Resets *****************************/

    // Reset the board (while leaving pellets alone) after Pacman dies
    fn death_reset(&mut self) {
        // Set Pacman to be in an empty state
        self.pacman_loc = EMPTY_LOC;

        // Decrease the number of lives Pacman has left
        self.decrement_lives();

        /*
            If the mode is not the initial mode and the ghosts aren't angry,
            change the mode back to the initial mode
        */
        if self.get_num_pellets() > ANGER_THRESHOLD1 {
            self.mode = INIT_MODE;
            self.set_mode_steps(INIT_MODE.duration());
        }

        // Set the fruit steps back to 0
        self.set_fruit_steps(0);

        // Reset all the ghosts to their original locations
        self.reset_all_ghosts();
    }

    // Reset the board (including pellets) after Pacman clears a level
    fn level_reset(&mut self) {
        // Set Pacman to be in an empty state
        self.pacman_loc = EMPTY_LOC;

        // If the mode is not the initial mode, change it
        self.mode = INIT_MODE;
        self.set_mode_steps(INIT_MODE.duration());

        // Reset the level penalty
        self.set_level_steps(LEVEL_DURATION);

        // Set the fruit steps back to 0
        self.set_fruit_steps(0);

        // Reset all the ghosts to their original locations
        self.reset_all_ghosts();

        // Reset the pellet bit array and count
        self.reset_pellets();
    }

    /************************** Motion (Pacman Location) **************************/

    // Move Pacman one space in a given direction
    pub fn move_pacman_dir(&mut self, dir: Direction) {
        // Check collisions with all the ghosts
        self.check_collisions();

        // Calculate the next row and column
        let next_loc = self.pacman_loc.get_neighbor_coords(dir);

        // Update Pacman's direction
        self.pacman_loc.dir = dir;

        // Check if there is a wall at the anticipated location, and return if so
        if self.wall_at(next_loc) {
            return;
        }

        // Move Pacman the anticipated spot
        self.pacman_loc.update_coords(next_loc);
        self.collect_pellet(next_loc);
    }

    // Move Pacman back to its spawn point, if necessary
    pub fn try_respawn_pacman(&mut self) {
        // Set Pacman to be in its original state
        if self.pacman_loc.is_empty() && self.get_lives() > 0 {
            self.pacman_loc = PACMAN_SPAWN_LOC;
        }
    }

    /******************************* Ghost Movement *******************************/

    // Frighten all ghosts at once
    fn frighten_all_ghosts(&mut self) {
        // Reset the ghost respawn combo back to 0
        self.ghost_combo = 0;

        // Loop over all the ghosts
        for ghost in &mut self.ghosts {
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
    pub fn reverse_all_ghosts(&mut self) {
        // Loop over all the ghosts
        for ghost in &mut self.ghosts {
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
    fn reset_all_ghosts(&mut self) {
        // Reset the ghost respawn combo back to 0
        self.ghost_combo = 0;

        // Reset each of the ghosts
        for ghost in &mut self.ghosts {
            ghost.reset();
        }

        // If no lives are left, set all ghosts to stare at the player, menacingly
        if self.get_lives() == 0 {
            for ghost in &mut self.ghosts {
                if ghost.color != Orange {
                    ghost.next_loc.dir = Stay;
                } else {
                    // Orange does like making eye contact, unfortunately
                    ghost.next_loc.dir = Left;
                }
            }
        }
    }

    // Update all ghosts at once
    pub fn update_all_ghosts(&mut self) {
        // Loop over the individual ghosts
        for ghost in &mut self.ghosts {
            ghost.update();
        }
    }

    // A game state function to plan all ghosts at once
    pub fn plan_all_ghosts(&mut self) {
        // Plan each ghost's next move
        for ghost_idx in 0..self.ghosts.len() {
            let chase_color = self.ghosts[ghost_idx].color;
            let chase_target = self.get_chase_target(chase_color);

            // If the location is empty (i.e. after a reset/respawn), don't plan
            if self.ghosts[ghost_idx].loc.is_empty() {
                return;
            }

            // Determine the next position based on the current direction
            let loc = self.ghosts[ghost_idx].loc;
            self.ghosts[ghost_idx].next_loc.advance_from(loc);

            // If the ghost is trapped, reverse the current direction and return
            if self.ghosts[ghost_idx].is_trapped() {
                self.ghosts[ghost_idx].next_loc.dir =
                    self.ghosts[ghost_idx].next_loc.dir.opposite();
                self.ghosts[ghost_idx].dec_trapped_steps();
                return;
            }

            // Decide on a target for this ghost, depending on the game mode.
            /*
                If the ghost is spawning in the ghost house, choose red's spawn
                location as the target to encourage it to leave the ghost house.

                Otherwise: pick chase or scatter targets, depending on the mode.
            */
            let target_loc = if self.ghosts[ghost_idx].spawning
                && !self.ghosts[ghost_idx]
                    .loc
                    .collides_with(GHOST_SPAWN_LOCS[Red as usize])
                && !self.ghosts[ghost_idx]
                    .next_loc
                    .collides_with(GHOST_SPAWN_LOCS[Red as usize])
            {
                GHOST_SPAWN_LOCS[Red as usize].get_coords()
            } else {
                match self.mode {
                    GameMode::CHASE => chase_target,
                    GameMode::SCATTER => self.ghosts[ghost_idx].scatter_target.get_coords(),
                }
            };

            // Determine which of the four neighboring moves to the next location are valid.
            let moves = Direction::all_except_stay().map(|dir| {
                (
                    dir,
                    self.ghosts[ghost_idx].next_loc.get_neighbor_coords(dir),
                )
            });
            let valid_moves = moves.into_iter().filter(|&(dir, loc)| {
                // If this move would make the ghost reverse, skip it.
                if dir == self.ghosts[ghost_idx].next_loc.dir.opposite() {
                    return false;
                }

                // Considerations when the ghost is spawning.
                if self.ghosts[ghost_idx].spawning {
                    // Determine if the move would be within the ghost house.
                    if self.ghost_spawn_at(loc) {
                        return true;
                    }

                    // Determine if the move would help the ghost escape the ghost house,
                    // and make it a valid one if so.
                    if loc == GHOST_HOUSE_EXIT_POS {
                        return true;
                    }
                }

                // Otherwise, the move is valid if it does not move into a wall.
                !self.wall_at(loc)
            });

            let chosen_move = if self.ghosts[ghost_idx].fright_steps > 1 {
                // If the ghost will still be frightened one tick later, immediately choose
                // a random valid direction and return.
                let mut rng = SmallRng::seed_from_u64(self.seed);
                let chosen_move = valid_moves.choose(&mut rng);
                self.seed = rng.gen();
                chosen_move
            } else {
                // Otherwise, pick the move that takes the ghost closest to its target.
                valid_moves.min_by_key(|&(_dir, loc)| dist_sq(loc, target_loc))
            }
            .expect("ghost has no valid moves!");

            // Once we have picked a move, set next_loc.dir to that direction.
            self.ghosts[ghost_idx].next_loc.dir = chosen_move.0;
        }
    }

    /************************ Ghost Targeting (Chase Mode) ************************/

    /*
    Returns the chase location of the red ghost
    (i.e. Pacman's exact location)
    */
    fn get_chase_target_red(&self) -> Position {
        // Return Pacman's current location
        self.pacman_loc.get_coords()
    }

    /*
    Returns the chase location of the pink ghost
    (i.e. 4 spaces ahead of Pacman's location)
    */
    fn get_chase_target_pink(&self) -> Position {
        // Return the red pink's target (4 spaces ahead of Pacman)
        self.pacman_loc.get_ahead_coords(4)
    }

    /*
    Returns the chase location of the cyan ghost
    (i.e. The red ghost's location, reflected about 2 spaces ahead of Pacman)
    */
    fn get_chase_target_cyan(&self) -> Position {
        // Get the 'pivot' square, 2 steps ahead of Pacman
        let (pivot_row, pivot_col) = self.pacman_loc.get_ahead_coords(2);

        // Get the current location of the red ghost
        let (red_row, red_col) = self.ghosts[Red as usize].loc.get_coords();

        // Return the pair of coordinates of the calculated target
        ((2 * pivot_row - red_row), (2 * pivot_col - red_col))
    }

    /*
    Returns the chase location of the orange ghost
    (i.e. Pacman's exact location, the same as red's target most of the time)
    Though, if close enough to Pacman, it should choose its scatter target
    */
    fn get_chase_target_orange(&self) -> Position {
        // Get Pacman's current location
        let pacman_pos = self.pacman_loc.get_coords();

        // Get the orange ghost's current location
        let orange_pos = self.ghosts[Orange as usize].loc.get_coords();

        // If Pacman is far enough from the ghost, return Pacman's location
        if dist_sq(orange_pos, pacman_pos) >= 64 {
            return pacman_pos;
        }

        // Otherwise, return the scatter location of orange
        self.ghosts[Orange as usize].scatter_target.get_coords()
    }

    // Returns the chase location of an arbitrary ghost color
    pub fn get_chase_target(&self, color: GhostColor) -> Position {
        match color {
            Red => self.get_chase_target_red(),
            Pink => self.get_chase_target_pink(),
            Cyan => self.get_chase_target_cyan(),
            Orange => self.get_chase_target_orange(),
        }
    }
}
