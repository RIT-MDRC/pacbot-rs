/***************************** Bitwise Operations *****************************/

use crate::{game_state::GameState, location::{LEFT, NONE}};

fn get_bit_u8(num: u8, bit_idx: usize) -> bool {
	((num >> bit_idx) & 1) == 1
}

fn get_bit_u16(num: u16, bit_idx: usize) -> bool {
	((num >> bit_idx) & 1) == 1
}

fn get_bit_u32(num: u32, bit_idx: usize) -> bool {
	((num >> bit_idx) & 1) == 1
}

fn modify_bit_u8(num: &mut u8, bit_idx: usize, bit_val: bool) {
	// If the bit is true, we should set the bit, otherwise we clear it
	if bit_val {
		*num |= (1 << bit_idx);
	} else {
		*num &= (!(1 << bit_idx));
	}
}

/****************************** Timing Functions ******************************/

impl GameState {

// Determines if the game state is ready to update
fn updateReady (&self,) -> bool {

	// Get the current ticks value
	let currTicks = self.getCurrTicks();

	// Get the update period (u16 to match the type of current ticks)
	let updatePeriod = self.getUpdatePeriod();

	// Update if the update period divides the current ticks
	currTicks%updatePeriod == 0
}

/**************************** Positional Functions ****************************/

// Determines if a position is within the bounds of the maze
fn inBounds (&self,row: i8, col: i8) -> bool {
 ((row >= 0 && row < mazeRows) && (col >= 0 && col < mazeCols))
}

// Determines if a pellet is at a given location
fn pelletAt (&self,row i8, col i8) -> bool {
	if !self.inBounds(row, col) {
		return false;
	}

	// Returns the bit of the pellet row corresponding to the column
	getBit(self.pellets[row], col)
}

/*
Collects a pellet if it is at a given location
Returns the number of pellets that are left
*/
fn collectPellet (&self,row: i8, col: i8) -> {

	// Collect fruit, if applicable
	if self.fruitExists() && self.pacmanLoc.collidesWith(self.fruitLoc) {
		self.setFruitSteps(0);
		self.incrementScore(fruitPoints);
	}

	// If there's no pellet, return
	if !self.pelletAt(row, col) {
		return;
	}

	// If we can clear the pellet's bit, decrease the number of pellets
	modifyBit(&(self.pellets[row]), col, false);
	self.decrementNumPellets();

	// If the we are in particular rows and columns, it is a super pellet
	let superPellet = ((row == 3) || (row == 23)) && ((col == 1) || (col == 26));

	// Make all the ghosts frightened if a super pellet is collected
	if superPellet {
		self.frightenAllGhosts();
	}

	// Update the score, depending on the pellet type
	if superPellet {
		self.incrementScore(superPelletPoints);
	} else {
		self.incrementScore(pelletPoints);
	}

	// Act depending on the number of pellets left over
let 	numPellets = self.getNumPellets();

	// Spawn fruit, if applicable
	if (numPellets == fruitThreshold1) && !self.fruitExists() {
		self.setFruitSteps(fruitDuration);
	} else if (numPellets == fruitThreshold2) && !self.fruitExists() {
		self.setFruitSteps(fruitDuration);
	}

	// Other pellet-related events
	if numPellets == angerThreshold1 { // Ghosts get angry (speeding up)
		self.setUpdatePeriod(u8(max(1, int(self.getUpdatePeriod())-2)))
		self.setMode(chase);
		self.setModeSteps(0xff);
	} else if numPellets == angerThreshold2 { // Ghosts get angrier
		self.setUpdatePeriod(u8(max(1, int(self.getUpdatePeriod())-2)))
		self.setMode(chase);
		self.setModeSteps(0xff);
	} else if numPellets == 0 {
		self.levelReset();
		self.incrementLevel();
	}
}

// Determines if a wall is at a given location
fn wallAt (&self,row: i8, col: i8) -> bool {
	if !self.inBounds(row, col) {
		return true
	}

	// Returns the bit of the wall row corresponding to the column
	return getBit(self.walls[row], col)
}

// Determines if the ghost house is at a given location
fn ghostSpawnAt (&self,row: i8, col: i8) -> bool {
	if !self.inBounds(row, col) {
		return false
	}

	// Returns the bit of the wall row corresponding to the column
	return ((row >= 13) && (row <= 14)) && ((col >= 11) && (col <= 15))
}

// Calculates the squared Euclidean distance between two points
fn distSq (&self,row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
	let dx = int(row2 - row1);
	let dy = int(col2 - col1);
	dx*dx + dy*dy
}

/***************************** Collision Handling *****************************/

// Check collisions between Pacman and all the ghosts
fn checkCollisions (&self,) {

	// Flag to decide which ghosts should respawn
	let ghostRespawnFlag = 0;

	// Keep track of how many ghosts need to respawn
	let numGhostRespawns = 0;

	// Loop over all the ghosts
	for ghost in self.ghosts {

		// Check each collision individually
		if self.pacmanLoc.collidesWith(ghost.loc) {

			// If the ghost was already eaten, skip it
			if ghost.isEaten() {
				continue;
			}

			// If the ghost is frightened, Pacman eats it, otherwise Pacman dies
			if ghost.isFrightened() {
				modifyBit(&ghostRespawnFlag, ghost.color, true);
				numGhostRespawns+= 1;
			} else {
				self.deathReset();
				return;
			}
		}
	}

	// If no ghosts need to respawn, there's no more work to do
	if numGhostRespawns == 0 {
		return
	}

	// Lock the motion mutex to synchronize with other ghost update routines
	self.respawnGhosts(numGhostRespawns, ghostRespawnFlag)
}

/***************************** Event-Based Resets *****************************/

// Reset the board (while leaving pellets alone) after Pacman dies
fn deathReset (&self,) {
	// Set the game to be paused at the next update
	self.setPauseOnUpdate(true);

	// Set Pacman to be in an empty state
	self.pacmanLoc.copyFrom(emptyLoc);

	// Decrease the number of lives Pacman has left
	self.decrementLives();

	/*
		If the mode is not the initial mode and the ghosts aren't angry,
		change the mode back to the initial mode
	*/
	if self.getNumPellets() > angerThreshold1 {
		self.setMode(initMode);
		self.setModeSteps(modeDurations[initMode]);
	}

	// Set the fruit steps back to 0
	self.setFruitSteps(0);

	// Reset all the ghosts to their original locations
	self.resetAllGhosts();
}

// Reset the board (including pellets) after Pacman clears a level
fn levelReset (&self,) {

	// Set the game to be paused at the next update
	self.setPauseOnUpdate(true);

	// Set Pacman to be in an empty state
	self.pacmanLoc.copyFrom(emptyLoc);

	// If the mode is not the initial mode, change it
	self.setMode(initMode);
	self.setModeSteps(modeDurations[initMode]);

	// Reset the level penalty
	self.setLevelSteps(levelDuration);

	// Set the fruit steps back to 0
	self.setFruitSteps(0);

	// Reset all the ghosts to their original locations
	self.resetAllGhosts();

	// Reset the pellet bit array and count
	self.resetPellets();
}

/************************** Motion (Pacman Location) **************************/

// Move Pacman one space in a given direction
fn movePacmanDir (&self,dir: u8) {

	// Check collisions with all the ghosts
	self.checkCollisions();

	// Ignore the command if the game is paused
	if self.isPaused() || self.getPauseOnUpdate() {
		return
	}

	// Shorthand to make computation simpler
	let pLoc = self.pacmanLoc;

	// Calculate the next row and column
	let (nextRow, nextCol) = pLoc.getNeighborCoords(dir);

	// Update Pacman's direction
	pLoc.updateDir(dir);

	// Check if there is a wall at the anticipated location, and return if so
	if self.wallAt(nextRow, nextCol) {
		return
	}

	// Move Pacman the anticipated spot
	pLoc.updateCoords(nextRow, nextCol);
	self.collectPellet(nextRow, nextCol);
}

// Move Pacman back to its spawn point, if necessary
fn tryRespawnPacman (&self,) {

	// Set Pacman to be in its original state
	if self.pacmanLoc.isEmpty() && self.getLives() > 0 {
		self.pacmanLoc.copyFrom(pacmanSpawnLoc);
	}
}

/******************************* Ghost Movement *******************************/

// Frighten all ghosts at once
fn frightenAllGhosts (&self,) {
	// Reset the ghost respawn combo back to 0
	self.ghostCombo = 0;

	// Loop over all the ghosts
	for ghost in self.ghosts {

		/*
			To frighten a ghost, set its fright steps to a specified value
			and trap it for one step (to force the direction to reverse)
		*/
		ghost.setFrightSteps(ghostFrightSteps);
		if !ghost.isTrapped() {
			ghost.setTrappedSteps(1);
		}
	}
}

// Reverse all ghosts at once (similar to frightenAllGhosts)
fn reverseAllGhosts (&self,) {

	// Loop over all the ghosts
	for ghost in self.ghosts {

		/*
			To change the direction a ghost, trap it for one step
			(to force the direction to reverse)
		*/
		if !ghost.isTrapped() {
			ghost.setTrappedSteps(1);
		}
	}
}

// Reset all ghosts at once
fn resetAllGhosts (&self,) {
	// Reset the ghost respawn combo back to 0
	self.ghostCombo = 0;

	// Add relevant ghosts to a wait group
	self.wgGhosts.Add(int(numColors));

	// Reset each of the ghosts
	for ghost in self.ghosts {
		ghost.reset();
	}

	// Wait for the resets to finish
	self.wgGhosts.Wait();

	// If no lives are left, set all ghosts to stare at the player, menacingly
	if self.getLives() == 0 {
		for ghost in self.ghosts {
			if ghost.color != orange {
				ghost.nextLoc.updateDir(NONE);
			} else { // Orange does like making eye contact, unfortunately
				ghost.nextLoc.updateDir(LEFT);
			}
		}
	}
}

// Respawn some ghosts, according to a flag
fn respawnGhosts (&self,
	numGhostRespawns: i32, ghostRespawnFlag: u8) {

	// Add relevant ghosts to a wait group
	self.wgGhosts.Add(numGhostRespawns);

	// Loop over the ghost colors again, to decide which should respawn
	for ghost in self.ghosts {

		// If the ghost should respawn, do so and increase the score and combo
		if getBit(ghostRespawnFlag, ghost.color) {

			// Respawn the ghost
			ghost.respawn();

			// Add points corresponding to the current combo length
			self.incrementScore(comboMultiplier << self.ghostCombo);

			// Increment the ghost respawn combo
			self.ghostCombo += 1;
		}
	}

	// Wait for the respawns to finish
	self.wgGhosts.Wait();
}

// Update all ghosts at once
fn updateAllGhosts (&self,) {
	// Add relevant ghosts to a wait group
	self.wgGhosts.Add(int(numColors))

	// Loop over the individual ghosts
	for ghost in &self.ghosts {
		ghost.update();
	}

	// Wait for the respawns to finish
	self.wgGhosts.Wait()
}

// A game state function to plan all ghosts at once
fn planAllGhosts (&self,) {
	// Add pending ghost plans
	self.wgGhosts.Add(int(numColors));

	// Plan each ghost's next move concurrently
	for ghost in self.ghosts {
		ghost.plan();
	}

	// Wait until all pending ghost plans are complete
	self.wgGhosts.Wait();
}

/************************ Ghost Targeting (Chase Mode) ************************/

/*
Returns the chase location of the red ghost
(i.e. Pacman's exact location)
*/
fn getChaseTargetRed (&self,) -> (i8, i8) {

	// Return Pacman's current location
	self.pacmanLoc.getCoords()
}

/*
Returns the chase location of the pink ghost
(i.e. 4 spaces ahead of Pacman's location)
*/
fn getChaseTargetPink (&self,) -> (i8, i8) {

	// Return the red pink's target (4 spaces ahead of Pacman)
	self.pacmanLoc.getAheadCoords(4)
}

/*
Returns the chase location of the cyan ghost
(i.e. The red ghost's location, reflected about 2 spaces ahead of Pacman)
*/
fn getChaseTargetCyan (&self,) -> (i8, i8) {

	// Get the 'pivot' square, 2 steps ahead of Pacman
	let (pivotRow, pivotCol) = self.pacmanLoc.getAheadCoords(2);

	// Get the current location of the red ghost
	let (redRow, redCol) = self.ghosts[RED].loc.getCoords();

	// Return the pair of coordinates of the calculated target
	((2*pivotRow - redRow),(2*pivotCol - redCol))
}

/*
Returns the chase location of the orange ghost
(i.e. Pacman's exact location, the same as red's target most of the time)
Though, if close enough to Pacman, it should choose its scatter target
*/
fn getChaseTargetOrange (&self,) -> (i8, i8) {

	// Get Pacman's current location
	let (pacmanRow, pacmanCol) = self.pacmanLoc.getCoords();

	// Get the orange ghost's current location
	 let (orangeRow, orangeCol) = self.ghosts[ORANGE].loc.getCoords();

	// If Pacman is far enough from the ghost, return Pacman's location
	if self.distSq(orangeRow, orangeCol, pacmanRow, pacmanCol) >= 64 {
		return ((pacmanRow),
			(pacmanCol));
	}

	// Otherwise, return the scatter location of orange
	self.ghosts[ORANGE].scatterTarget.getCoords()
}

// Returns the chase location of an arbitrary ghost color
fn getChaseTarget (&self,color: u8) -> (i8, i8) {
	match color {
		RED => self.getChaseTargetRed(),
		PINK => self.getChaseTargetPink(),
		CYAN => self.getChaseTargetCyan(),
		ORANGE => self.getChaseTargetOrange(),
		_ => emptyLoc.getCoords(),
	}
}
}