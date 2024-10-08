use serde::{Deserialize, Serialize};

use GhostColor::*;

use crate::{
    location::LocationState,
    variables::{EMPTY_LOC, GHOST_SCATTER_TARGETS, GHOST_SPAWN_LOCS, GHOST_TRAPPED_STEPS},
};

/// Ghost colors
#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum GhostColor {
    Red = 0,
    Pink = 1,
    Cyan = 2,
    Orange = 3,
}

impl TryFrom<u8> for GhostColor {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Red),
            1 => Ok(Pink),
            2 => Ok(Cyan),
            3 => Ok(Orange),
            _ => Err(()),
        }
    }
}

// Names of the ghosts (not the nicknames, just the colors, for debugging)
pub const GHOST_NAMES: [GhostColor; 4] = [Red, Pink, Cyan, Orange];

/*
An object to keep track of the location and attributes of a ghost
*/
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct GhostState {
    pub loc: LocationState,            // Current location
    pub next_loc: LocationState,       // Planned location (for next update)
    pub scatter_target: LocationState, // Position of (fixed) scatter target
    pub color: GhostColor,
    pub trapped_steps: u8,
    pub fright_steps: u8,
    pub spawning: bool, // Flag set when spawning
    pub eaten: bool,    // Flag set when eaten and returning to ghost house
}

impl GhostState {
    // Create a new ghost state with given location and color values
    pub fn new(color: GhostColor) -> Self {
        Self {
            loc: EMPTY_LOC,
            next_loc: GHOST_SPAWN_LOCS[color as usize],
            scatter_target: GHOST_SCATTER_TARGETS[color as u8 as usize],
            color,
            trapped_steps: GHOST_TRAPPED_STEPS[color as u8 as usize],
            fright_steps: 0,
            spawning: true,
            eaten: false,
        }
    }

    // Update auxiliary info (fright steps and spawning flag, 1 byte)
    pub fn update_aux(&mut self, aux_info: u8) {
        self.fright_steps = aux_info & 0x3f;
        self.spawning = (aux_info >> 7) != 0;
    }

    pub fn update_aux2(&mut self, aux2: u8) {
        self.trapped_steps = aux2 & 0x3f;
        self.eaten = (aux2 >> 7) != 0;
    }

    pub fn get_aux(&self) -> u8 {
        let mut aux_info = self.fright_steps & 0x3f;
        if self.spawning {
            aux_info |= 1 << 7;
        }
        aux_info
    }

    pub fn get_aux2(&self) -> u8 {
        let mut aux_info = self.trapped_steps & 0x3f;
        if self.eaten {
            aux_info |= 1 << 7;
        }
        aux_info
    }

    /*************************** Ghost Frightened State ***************************/

    // Set the fright steps of a ghost
    pub fn set_fright_steps(&mut self, steps: u8) {
        self.fright_steps = steps;
    }

    // Decrement the fright steps of a ghost
    pub fn dec_fright_steps(&mut self) {
        self.fright_steps -= 1;
    }

    // Check if a ghost is frightened
    pub fn is_frightened(&self) -> bool {
        // Return whether there is at least one fright step left
        self.fright_steps > 0
    }

    /****************************** Ghost Trap State ******************************/

    // Set the trapped steps of a ghost
    pub fn set_trapped_steps(&mut self, steps: u8) {
        self.trapped_steps = steps;
    }

    // Decrement the trapped steps of a ghost
    pub fn dec_trapped_steps(&mut self) {
        self.trapped_steps -= 1;
    }

    // Check if a ghost is trapped
    pub fn is_trapped(&self) -> bool {
        // Return whether there is at least one fright step left
        self.trapped_steps > 0
    }

    /**************************** Ghost Spawning State ****************************/

    // Set the ghost spawning flag
    pub fn set_spawning(&mut self, spawning: bool) {
        self.spawning = spawning;
    }

    /// Check if a ghost is spawning.
    pub fn is_spawning(&self) -> bool {
        self.spawning
    }

    /****************************** Ghost Eaten Flag ******************************/

    /// Set the ghost eaten flag.
    pub fn set_eaten(&mut self, eaten: bool) {
        self.eaten = eaten;
    }

    /// Check if a ghost is eaten.
    pub fn is_eaten(&self) -> bool {
        self.eaten
    }

    pub fn from_bytes(color: GhostColor, location: LocationState, aux: u8, aux2: u8) -> Self {
        let mut s = Self {
            loc: location,
            next_loc: location, // planning happens after all ghosts are initialized
            scatter_target: GHOST_SCATTER_TARGETS[color as usize],
            color,
            trapped_steps: 0, // aux2
            fright_steps: 0,  // aux
            spawning: false,  // aux
            eaten: false,     // aux2
        };
        s.update_aux(aux);
        s.update_aux2(aux2);
        s
    }
}
