use std::fmt::Debug;

use stage_manager::StageChange;
use sui::{DynamicLayable, Layable};
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

#[derive(Debug, Clone)]
/// this is the only event type that'll get relayed to the remote controller
pub struct RemoteEvent<T>(pub T);

#[derive(Debug)]
pub struct RemoteStage<T: Send + Debug> {
	current: DynamicLayable<'static>,

	handle: tokio::task::JoinHandle<()>,

	stage_rx: Receiver<StageChange<'static>>,
	events_tx: Sender<T>,
}
impl<T: Send + Debug> RemoteStage<T> {
	pub fn new_explicit<F: Future<Output = ()> + Send + 'static>(
		layable: impl Layable + Debug + 'static,
		controller: impl FnOnce(Sender<StageChange<'static>>, Receiver<T>) -> F,
	) -> Self {
		let layable = sui::custom_only_debug(layable);

		let (stage_tx, stage_rx) = tokio::sync::mpsc::channel(5);
		let (events_tx, events_rx) = tokio::sync::mpsc::channel(10);

		let controller = controller(stage_tx, events_rx);
		let handle = tokio::task::spawn(controller);

		Self {
			current: layable,
			handle,
			stage_rx,
			events_tx,
		}
	}

	/// creates a new, remotely controlled stage
	pub fn new<F: Future<Output = ()> + Send + 'static>(
		controller: impl FnOnce(Sender<StageChange<'static>>, Receiver<T>) -> F,
	) -> Self {
		Self::new_explicit(sui::comp::Space::new(10, 10), controller)
	}

	fn try_recv(&mut self) {
		match self.stage_rx.try_recv() {
			Ok(StageChange::Simple(new_stage)) => self.current = new_stage,
			Ok(StageChange::Swapper(swap_fn)) => {
				let current_stage = std::mem::replace(
					&mut self.current,
					sui::custom(sui::comp::Space::new(10, 10)), // ! self.current replaced with a dummy stage for this block
				);
				let new_stage = swap_fn(current_stage);

				self.current = new_stage;
			}
			Err(TryRecvError::Empty) => (),
			Err(TryRecvError::Disconnected) => {
				panic!(
					"RemoteStage's receiver disconnected while waiting for results to be yielded\nyour async environment that yields the stages should probably made to run forever, as dropping the RemoteStage aborts the task anyway"
				)
			}
		}
	}
}
impl<T: Send + Debug> Drop for RemoteStage<T> {
	fn drop(&mut self) {
		self.handle.abort();
	}
}

impl<T: Send + Debug + 'static> Layable for RemoteStage<T> {
	fn size(&self) -> (i32, i32) {
		self.current.size()
	}
	fn render(&self, d: &mut sui::Handle, det: sui::Details, scale: f32) {
		self.current.render(d, det, scale)
	}

	fn tick(&mut self) {
		self.try_recv();
		self.current.tick();
	}
	fn pass_events(
		&mut self,
		events: impl Iterator<Item = sui::core::Event>,
		det: sui::Details,
		scale: f32,
	) -> impl Iterator<Item = sui::core::ReturnEvent> {
		self.try_recv();

		for event in self.current.pass_events(events, det, scale) {
			if event.can_take::<RemoteEvent<T>>() {
				let event: RemoteEvent<T> = event
					.take()
					.expect("can_take said we can take but we can't take");
				let event = event.0;

				match self.events_tx.try_send(event) {
					Ok(a) => a,
					Err(err) => {
						eprintln!(
							"RemoteStage had to drop RemoteEvent since the buffer was full\nerr: {err}"
						)
					}
				}
			}
		}
		std::iter::empty()
	}
}
