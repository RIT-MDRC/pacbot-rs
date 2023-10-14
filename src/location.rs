/// Directions:                U   L   D   R  None
const D_ROW: [i8; 5] = [-1, -0, 1, 0, 0];
const D_COL: [i8; 5] = [-0, -1, 0, 1, 0];

/// Enum-like declaration to hold the direction indices from above

const UP: u8 = 0;
const LEFT: u8 = 1;
const DOWN: u8 = 2;
const RIGHT: u8 = 3;
const NUM_DIRS: usize = 4;
const NONE: usize = NUM_DIRS;

// Names of the directions (forr debugging)
const DIR_NAMES: [&str; NUM_DIRS + 1] = ["up", "left", "down", "right", "none"];

/*
An object to keep track of the position and direction of an agent
*/
#[derive(Copy, Clone)]
struct LocationState {
    row: i8, // Row
    col: i8, // Col
    dir: u8, // Index of the direction, within the direction arrays
}

impl LocationState {
    pub fn new(row: i8, col: i8, dir: u8) -> Self {
        Self { row, col, dir }
    }

	/******************************** Read Location *******************************/

// Determine if another location state matches with the given location
fn collides_with(&self, loc2: LocationState) -> bool {
	// If any of the rows or columns is at least 32, they don't collide
	if loc.row >= 32 || loc.col >= 32 || loc2.row >= 32 || loc2.col >= 32 {
		return false
	}

	// Return if both coordinates match
	return ((loc.row == loc2.row) && (loc.col == loc2.col))
}

// Determine if a given location state matches with the empty location
fn is_empty() -> bool {
	// Return if both coordinates match
	return ((loc.row == emptyLoc.row) && (loc.col == emptyLoc.col))
}

// Return a direction corresponding to an existing location
fn get_dir() -> u8 {
	return loc.dir
}

fn get_reversed_dir() -> u8 {

	// Copy the current direction
	dir := loc.getDir()

	// Switch between up and down, or left and right
	switch dir {
	case up:
		return down
	case left:
		return right
	case down:
		return up
	case right:
		return left
	default:
		return dir
	}
}

// Return a set of coordinates corresponding to an existing location
fn get_coords(&self) -> (i8, i8) {

	// Return the pair of coordinates
	return (loc.row),
		(loc.col)
}

// Create a new set of coordinates as the neighbor of an existing location
fn get_neighbor_coords(&self, dir u8) (i8, i8) {

	// Add the deltas to the coordinates and return the pair
	return (loc.row + D_ROW[dir]),
		(loc.col + D_COL[dir])
}

/*
Return a set of coordinates a few steps ahead (in the direction it is facing)
of a given location state
*/
fn get_ahead_coords(&self, spaces: i8)  -> (i8, i8) {

	// Add the deltas to the coordinates and return the pair
	return (self.row + D_ROW[self.dir]*spaces),
		(self.col + D_COL[self.dir]*spaces)
}

/*
Set the given location to be one time step after another location,
and copy the current direction
*/
fn advance_from(&self, loc2 *locationState) {

	// Set the next location to be one ahead of the current one
	loc.updateCoords(loc2.getAheadCoords(1))

	// Keep the same direction by default
	loc.updateDir(loc2.getDir())
}

// Move a given location state to specified coordinates
fn update_coords(&self, row i8, col i8) {
	// Update the values
	loc.row = row
	loc.col = col
}
}
