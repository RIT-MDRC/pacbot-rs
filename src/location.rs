use serde::{Deserialize, Serialize};
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
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct LocationState {
    pub row: i8, // Row
    pub col: i8, // Col
    pub dir: u8, // Index of the direction, within the direction arrays
}

impl LocationState {
    pub const fn new(row: i8, col: i8, dir: u8) -> Self {
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
        (self.row == loc2.row) && (self.col == loc2.col)
    }

    // Determine if a given location state matches with the empty location
    pub fn is_empty(&self) -> bool {
        // Return if both coordinates match
        (self.row == EMPTY_LOC.row) && (self.col == EMPTY_LOC.col)
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
        (self.row, self.col)
    }

    // Create a new set of coordinates as the neighbor of an existing location
    pub fn get_neighbor_coords(&self, dir: u8) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        (
            self.row + D_ROW[dir as usize],
            self.col + D_COL[dir as usize],
        )
    }

    /*
    Return a set of coordinates a few steps ahead (in the direction it is facing)
    of a given location state
    */
    pub fn get_ahead_coords(&self, spaces: i8) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        (
            self.row + D_ROW[self.dir as usize] * spaces,
            self.col + D_COL[self.dir as usize] * spaces,
        )
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

    pub fn update(&mut self, loc_uint16: u16) {
        // Get the row and column bytes
        let row_uint8: u8 = (loc_uint16 >> 8) as u8;
        let col_uint8: u8 = (loc_uint16 & 0xff) as u8;

        // Get the row direction (2's complement of first 2 bits)
        // TODO I don't know why this is different
        // self.rowDir = (row_uint8 >> 6) as i8;
        // if self.rowDir >= 2 {
        //     self.rowDir -= 4;
        // }

        // Get the row value (last 6 bits)
        self.row = (row_uint8 & 0x3f) as i8;

        // Get the col direction (2's complement of first 2 bits)
        // TODO I don't know why this is different
        // self.colDir = (col_uint8 >> 6) as i8;
        // if self.colDir >= 2 {
        //     self.colDir -= 4;
        // }

        // Get the column value (last 6 bits)
        self.col = (col_uint8 & 0x3f) as i8;
    }
}

// Returns the squared Euclidean distance between two points.
pub fn dist_sq(p1: (i8, i8), p2: (i8, i8)) -> u32 {
    let row1: i32 = p1.0.into();
    let col1: i32 = p1.1.into();
    let row2: i32 = p2.0.into();
    let col2: i32 = p2.1.into();

    let dx = row2 - row1;
    let dy = col2 - col1;
    (dx * dx + dy * dy) as u32
}
