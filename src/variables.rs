use crate::{game_modes::GameMode, location::Direction::*, location::LocationState};

/// The number of rows in the pellets and walls states
pub const MAZE_ROWS: usize = 31;

/// The number of columns in the pellets and walls states
pub const MAZE_COLS: usize = 28;

/// The update period that the game starts with by default
pub const INIT_UPDATE_PERIOD: u8 = 12;

/// The number of steps (update periods) that pass before the level speeds up
pub const LEVEL_DURATION: u16 = 960; // 8 minutes at 24 fps, update period = 12

/// The number of steps (update periods) before a level speeds up further
pub const LEVEL_PENALTY_DURATION: u16 = 240; // 2 min (24fps, update period = 12)

/// The mode that the game starts on by default
pub const INIT_MODE: GameMode = GameMode::SCATTER;

/// The level that Pacman starts on by default
pub const INIT_LEVEL: u8 = 1;

/// The number of lives that Pacman starts with
pub const INIT_LIVES: u8 = 3;

/// The coordinates where the ghost house exit is located
pub const GHOST_HOUSE_EXIT_POS: (i8, i8) = (12, 13);

/// Spawn position for Pacman
pub const PACMAN_SPAWN_LOC: LocationState = LocationState::new(23, 13, Up);

/// Spawn position for the fruit
pub const FRUIT_SPAWN_LOC: LocationState = LocationState::new(17, 13, Stay);

// The number of steps that the fruit stays on the maze for
pub const FRUIT_DURATION: u8 = 30;

// The points earned upon collecting a fruit
pub const FRUIT_POINTS: u16 = 100;

// "Invalid" location - serializes to 0x00100000 0x00100000
pub const EMPTY_LOC: LocationState = LocationState::new(32, 32, Stay);

// Spawn positions for the ghosts
pub const GHOST_SPAWN_LOCS: [LocationState; 4] = [
    LocationState::new(11, 13, Left), // red
    LocationState::new(13, 13, Down), // pink
    LocationState::new(14, 11, Up),   // cyan
    LocationState::new(14, 15, Up),   // orange
];

// Scatter targets for the ghosts - should remain constant
pub const GHOST_SCATTER_TARGETS: [LocationState; 4] = [
    LocationState::new(-3, 25, Stay), // red
    LocationState::new(-3, 2, Stay),  // pink
    LocationState::new(31, 27, Stay), // cyan
    LocationState::new(31, 0, Stay),  // orange
];

// The number of steps that the ghosts stay in the trapped state for
pub const GHOST_TRAPPED_STEPS: [u8; 4] = [
    0,  // red
    5,  // pink
    16, // cyan
    32, // orange
];

// The number of steps that the ghosts stay in the frightened state for
pub const GHOST_FRIGHT_STEPS: u8 = 40;

// The number of pellets in a typical game of Pacman
pub const INIT_PELLET_COUNT: u16 = 244;

// The number of pellets at which to spawn the first fruit
pub const FRUIT_THRESHOLD1: u16 = 174;

// The number of pellets at which to spawn the second fruit
pub const FRUIT_THRESHOLD2: u16 = 74;

// The number of pellets at which to make the ghosts angry
pub const ANGER_THRESHOLD1: u16 = 20;

// The number of pellets at which to make the ghosts angrier
pub const ANGER_THRESHOLD2: u16 = 10;

// The points earned when collecting a pellet
pub const PELLET_POINTS: u16 = 10;

// The points earned when collecting a pellet
pub const SUPER_PELLET_POINTS: u16 = 50;

// The multiplier for the combo from catching successive frightened ghosts
pub const COMBO_MULTIPLIER: u16 = 200;

// Column-wise, this may look backwards; column 0 is at bit 0 on the right
// (Tip: Ctrl+F '1' to see the initial pellet locations)
pub const INIT_PELLETS: [u32; 31] = [
    //                middle
    // col:             vv    8 6 4 2 0
    0b0000_0000000000000000000000000000, // row 0
    0b0000_0111111111111001111111111110, // row 1
    0b0000_0100001000001001000001000010, // row 2
    0b0000_0100001000001001000001000010, // row 3
    0b0000_0100001000001001000001000010, // row 4
    0b0000_0111111111111111111111111110, // row 5
    0b0000_0100001001000000001001000010, // row 6
    0b0000_0100001001000000001001000010, // row 7
    0b0000_0111111001111001111001111110, // row 8
    0b0000_0000001000000000000001000000, // row 9
    0b0000_0000001000000000000001000000, // row 10
    0b0000_0000001000000000000001000000, // row 11
    0b0000_0000001000000000000001000000, // row 12
    0b0000_0000001000000000000001000000, // row 13
    0b0000_0000001000000000000001000000, // row 14
    0b0000_0000001000000000000001000000, // row 15
    0b0000_0000001000000000000001000000, // row 16
    0b0000_0000001000000000000001000000, // row 17
    0b0000_0000001000000000000001000000, // row 18
    0b0000_0000001000000000000001000000, // row 19
    0b0000_0111111111111001111111111110, // row 20
    0b0000_0100001000001001000001000010, // row 21
    0b0000_0100001000001001000001000010, // row 22
    0b0000_0111001111111001111111001110, // row 23
    0b0000_0001001001000000001001001000, // row 24
    0b0000_0001001001000000001001001000, // row 25
    0b0000_0111111001111001111001111110, // row 26
    0b0000_0100000000001001000000000010, // row 27
    0b0000_0100000000001001000000000010, // row 28
    0b0000_0111111111111111111111111110, // row 29
    0b0000_0000000000000000000000000000, // row 30
];

// Column-wise, this may look backwards; column 0 is at bit 0 on the right
// (Tip: Ctrl+F '0' to see the valid Pacman locations)
pub const INIT_WALLS: [u32; 31] = [
    //                middle
    // col:             vv    8 6 4 2 0
    0b0000_1111111111111111111111111111, // row 0
    0b0000_1000000000000110000000000001, // row 1
    0b0000_1011110111110110111110111101, // row 2
    0b0000_1011110111110110111110111101, // row 3
    0b0000_1011110111110110111110111101, // row 4
    0b0000_1000000000000000000000000001, // row 5
    0b0000_1011110110111111110110111101, // row 6
    0b0000_1011110110111111110110111101, // row 7
    0b0000_1000000110000110000110000001, // row 8
    0b0000_1111110111110110111110111111, // row 9
    0b0000_1111110111110110111110111111, // row 10
    0b0000_1111110110000000000110111111, // row 11
    0b0000_1111110110111111110110111111, // row 12
    0b0000_1111110110111111110110111111, // row 13
    0b0000_1111110000111111110000111111, // row 14
    0b0000_1111110110111111110110111111, // row 15
    0b0000_1111110110111111110110111111, // row 16
    0b0000_1111110110000000000110111111, // row 17
    0b0000_1111110110111111110110111111, // row 18
    0b0000_1111110110111111110110111111, // row 19
    0b0000_1000000000000110000000000001, // row 20
    0b0000_1011110111110110111110111101, // row 21
    0b0000_1011110111110110111110111101, // row 22
    0b0000_1000110000000000000000110001, // row 23
    0b0000_1110110110111111110110110111, // row 24
    0b0000_1110110110111111110110110111, // row 25
    0b0000_1000000110000110000110000001, // row 26
    0b0000_1011111111110110111111111101, // row 27
    0b0000_1011111111110110111111111101, // row 28
    0b0000_1000000000000000000000000001, // row 29
    0b0000_1111111111111111111111111111, // row 30
];
