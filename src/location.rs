use crate::variables::EMPTY_LOC;

/// Directions:              U   L  D  R  None
pub const D_ROW: [i8; 5] = [-1, -0, 1, 0, 0];
pub const D_COL: [i8; 5] = [-0, -1, 0, 1, 0];

/// Enum-like declaration to hold the direction indices from above

pub const UP: u8 = 0;
pub const LEFT: u8 = 1;
pub const DOWN: u8 = 2;
pub const RIGHT: u8 = 3;
pub const NUM_DIRS: usize = 4;
pub const NONE: u8 = 4;

// Names of the directions (forr debugging)
pub const DIR_NAMES: [&str; NUM_DIRS + 1] = ["up", "left", "down", "right", "none"];

/// An object to keep track of the position and direction of an agent.
#[derive(Copy, Clone)]
pub struct LocationState {
    pub row: i8, // Row
    pub col: i8, // Col
    pub dir: u8, // Index of the direction, within the direction arrays
}

impl LocationState {
    pub fn new(row: i8, col: i8, dir: u8) -> Self {
        Self { row, col, dir }
    }

    /******************************** Read Location *******************************/

    // Determine if another location state matches with the given location
    pub fn collides_with(&self, loc2: LocationState) -> bool {
        // If any of the rows or columns is at least 32, they don't collide
        if self.row >= 32 || self.col >= 32 || loc2.row >= 32 || loc2.col >= 32 {
            return false;
        }

        // Return if both coordinates match
        return (self.row == loc2.row) && (self.col == loc2.col);
    }

    // Determine if a given location state matches with the empty location
    pub fn is_empty(&self) -> bool {
        // Return if both coordinates match
        return (self.row == EMPTY_LOC.row) && (self.col == EMPTY_LOC.col);
    }

    // Return a direction corresponding to an existing location
    pub fn get_dir(&self) -> u8 {
        self.dir
    }

    pub fn get_reversed_dir(&self) -> u8 {
        // Copy the current direction
        let dir = self.get_dir();

        // Switch between up and down, or left and right
        match dir {
            UP => DOWN,
            LEFT => RIGHT,
            DOWN => UP,
            RIGHT => LEFT,
            _ => dir,
        }
    }

    // Return a set of coordinates corresponding to an existing location
    pub fn get_coords(&self) -> (i8, i8) {
        // Return the pair of coordinates
        return (self.row, self.col);
    }

    // Create a new set of coordinates as the neighbor of an existing location
    pub fn get_neighbor_coords(&self, dir: u8) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        return (
            self.row + D_ROW[dir as usize],
            self.col + D_COL[dir as usize],
        );
    }

    /*
    Return a set of coordinates a few steps ahead (in the direction it is facing)
    of a given location state
    */
    pub fn get_ahead_coords(&self, spaces: i8) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        return (
            self.row + D_ROW[self.dir as usize] * spaces,
            self.col + D_COL[self.dir as usize] * spaces,
        );
    }

    /*
    Set the given location to be one time step after another location,
    and copy the current direction
    */
    pub fn advance_from(&mut self, loc2: LocationState) {
        // Set the next location to be one ahead of the current one
        (self.row, self.col) = loc2.get_ahead_coords(1);

        // Keep the same direction by default
        self.dir = loc2.get_dir();
    }

    // Move a given location state to specified coordinates
    pub fn update_coords(&mut self, coords: (i8, i8)) {
        (self.row, self.col) = coords;
    }
}
