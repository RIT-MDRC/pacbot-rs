use core2::io;
use core2::io::ErrorKind;
use serde::{Deserialize, Serialize};

use Direction::*;

use crate::game_helpers::Position;
use crate::variables::EMPTY_LOC;

pub fn is_super_pellet(position: Position) -> bool {
    let (row, col) = position;
    ((row == 3) || (row == 23)) && ((col == 1) || (col == 26))
}

pub const SUPER_PELLETS: [Position; 4] = [(3, 1), (3, 26), (23, 1), (23, 26)];

/// Directions
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Left = 1,
    Down = 2,
    Right = 3,
    Stay = 4,
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Up),
            1 => Ok(Left),
            2 => Ok(Down),
            3 => Ok(Right),
            4 => Ok(Stay),
            _ => Err(()),
        }
    }
}

impl Direction {
    pub const fn all_except_stay() -> [Direction; 4] {
        [Up, Left, Down, Right]
    }

    /// Get the opposite direction, ex. Up -> Down
    pub const fn opposite(&self) -> Direction {
        match self {
            Up => Down,
            Left => Right,
            Down => Up,
            Right => Left,
            Stay => Stay,
        }
    }

    /// Get the direction vector, ex. Up -> (-1, 0)
    pub const fn vector(&self) -> (i8, i8) {
        match self {
            Up => (-1, 0),
            Down => (1, 0),
            Left => (0, -1),
            Right => (0, 1),
            Stay => (0, 0),
        }
    }
}

/// An object to keep track of the position and direction of an agent.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq, Ord, Eq)]
pub struct LocationState {
    pub row: i8,
    pub col: i8,
    pub dir: Direction,
}

impl LocationState {
    pub const fn new(row: i8, col: i8, dir: Direction) -> Self {
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

    // Return a set of coordinates corresponding to an existing location
    pub fn get_coords(&self) -> (i8, i8) {
        // Return the pair of coordinates
        (self.row, self.col)
    }

    // Create a new set of coordinates as the neighbor of an existing location
    pub fn get_neighbor_coords(&self, dir: Direction) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        (self.row + dir.vector().0, self.col + dir.vector().1)
    }

    /*
    Return a set of coordinates a few steps ahead (in the direction it is facing)
    of a given location state
    */
    pub fn get_ahead_coords(&self, spaces: i8) -> (i8, i8) {
        // Add the deltas to the coordinates and return the pair
        (
            self.row + self.dir.vector().0 * spaces,
            self.col + self.dir.vector().1 * spaces,
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
        self.dir = loc2.dir;
    }

    // Move a given location state to specified coordinates
    pub fn update_coords(&mut self, coords: (i8, i8)) {
        (self.row, self.col) = coords;
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        let i8s = [
            (self.dir.vector().0 << 6) | self.row,
            (self.dir.vector().1 << 6) | self.col,
        ];
        i8s.map(|x| x.to_be_bytes()[0])
    }

    pub fn from_bytes(bytes: [u8; 2]) -> io::Result<Self> {
        let bytes = bytes.map(|x| i8::from_be_bytes([x]));

        // Get the row and column bytes
        let row: i8 = bytes[0] & 0b00111111;
        let col: i8 = bytes[1] & 0b00111111;

        let d_row: i8 = bytes[0] >> 6;
        let d_col: i8 = bytes[0] >> 6;

        let dir = match (d_row, d_col) {
            (0, 0) => Stay,
            (-1, 0) => Up,
            (1, 0) => Down,
            (0, -1) => Left,
            (0, 1) => Right,
            _ => return Err(io::Error::from(ErrorKind::InvalidInput)),
        };

        Ok(Self { row, col, dir })
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
