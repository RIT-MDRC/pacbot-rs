use game_state::GameState;

/// A game engine object, to act as an intermediary between the web broker
/// and the internal game state - its responsibility is to read responses from
/// clients and routinely send serialized copies of the game state to them.
pub struct GameEngine {
	state: GameState,
	// serves as the game clock
	ticker: time.Ticker,
}

impl GameEngine {
	/// Create a new game engine, casting channels to be uni-directional
	pub fn new(clock_rate: i32)  -> Self {

		// Time between ticks
		let _tick_time = 1000000 * time::Microsecond / time::Duration(clock_rate);
		Self {
			state: GameState::new(),
			ticker: time.NewTicker(_tick_time),
		}
	}

	/// Start the game engine - should be launched as a go-routine.
	pub fn step() {
		if ge.state.updateReady() {
			ge.state.updateAllGhosts();
			ge.state.tryRespawnPacman();
			if ge.state.getPauseOnUpdate() {
				ge.state.pause();
				ge.state.setPauseOnUpdate(false);
			}
			ge.state.checkCollisions();
			ge.state.handleStepEvents();
		}
		if ge.state.updateReady() {
			ge.state.planAllGhosts();
		}
	}
}
