use std::sync::Arc;

use arc_swap::ArcSwap;
use game_core::{GAME_TICK_FREQUENCY, GameData, GameProvider, tool::Tool};
use tokio::{sync::broadcast, task::JoinHandle};

#[derive(Debug)]
/// essentially a GameData on another thread, ticked asynchronously \
/// uses the tool_use_rx from game to do tool events
pub struct MultithreadedGame {
	handle: JoinHandle<()>,
	data: Arc<ArcSwap<GameData>>,
}
impl GameProvider for MultithreadedGame {
	fn data<'a>(&'a self) -> impl std::ops::Deref<Target = GameData> + 'a {
		self.data.load().clone()
	}
	fn standard_tick(&mut self) {}

	fn tool_use(&mut self, _: &game_core::tool::Tool, _: (i32, i32)) {
		// tool use handled separately with tool_use_rx
	}
}

impl MultithreadedGame {
	pub fn new(game_data: GameData, tool_use_rx: broadcast::Receiver<(Tool, (i32, i32))>) -> Self {
		let data = ArcSwap::new(Arc::new(game_data));
		let data = Arc::new(data);

		let data = data.clone();

		// the main task that drives the game forward
		let tick_task = {
			let data = data.clone();

			async move {
				let mut interval = tokio::time::interval(GAME_TICK_FREQUENCY);
				loop {
					interval.tick().await;

					let old_game = data.load_full();
					let mut game = GameData::clone(&old_game);
					game.tick();

					// TODO: check to see if game actually changed before swapping it out
					data.swap(Arc::new(game));
				}
			}
		};

		// the task that reacts to tool uses
		let tool_task = {
			let mut tool_use_rx = tool_use_rx;
			let data = data.clone();
			async move {
				loop {
					let (tool, pos) = match tool_use_rx.recv().await {
						Ok(a) => a,
						Err(broadcast::error::RecvError::Closed) => {
							mklogger::eprintln!("MultithreadedGame's tool_use received broke");
							break;
						}
						Err(_) => continue,
					};
					let old_game = data.load_full();
					let mut game = GameData::clone(&old_game);
					tool.r#use(&mut game, pos);

					// TODO: check to see if game actually changed before swapping it out
					data.swap(Arc::new(game));
				}
			}
		};

		let master_task = async move {
			let _joined = tokio::join!(tick_task, tool_task);
			mklogger::println!(
				"master_task finished execution. you'll probably never see this but if u do then hi"
			);
		};
		let handle = tokio::spawn(master_task);

		Self { handle, data }
	}
}
impl Drop for MultithreadedGame {
	fn drop(&mut self) {
		self.handle.abort();
	}
}
