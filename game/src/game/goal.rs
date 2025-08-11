use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::DefaultHasher;

use anyhow::anyhow;
use rust_i18n::t;
use stage_manager_remote::RemoteStageChange;
use sui::{Layable, LayableExt};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{
	textures::{TextureID, Textures},
	world::{EResource, Resource},
};

#[derive(Debug)]
/// defines what the goal is for the level
pub struct Goal {
	resources: HashMap<EResource, i32>,
}
impl Goal {
	pub fn new<I: IntoIterator<Item = (EResource, i32)>>(iter: I) -> Self {
		Self {
			resources: iter.into_iter().collect(),
		}
	}
	pub fn from_hashmap(resources: HashMap<EResource, i32>) -> Self {
		Self { resources }
	}
}

#[derive(Debug)]
struct RenderData {
	textures: Textures,
	display_tx: mpsc::Sender<RemoteStageChange>,
}

#[derive(Debug)]
/// the wrapper for the resource receiver to make checking for the goal easier
pub struct ResourceCounter {
	goal: Goal,

	resource_rx: mpsc::Receiver<EResource>,
	resources: HashMap<EResource, i32>,

	render_data: Option<RenderData>,
}
impl ResourceCounter {
	pub fn enable_display_tx(
		&mut self,
		textures: Textures,
		display_tx: mpsc::Sender<RemoteStageChange>,
	) {
		self.render_data = Some(RenderData {
			textures,
			display_tx,
		});
	}

	/// should be called post-tick
	fn progress_into_component(&self) -> Option<impl Layable + Debug + Clone + 'static> {
		match &self.render_data {
			Some(render_data) => {
				let rows = self.goal.resources.iter().map(|(resource, target)| {
					let received = self.resources.get(resource).copied().unwrap_or_default();

					let texture = render_data.textures.texture_for(resource.texture_id());
					let texture = match texture {
						Some(a) => Some(a.clone()),
						None => render_data
							.textures
							.texture_for(TextureID::Transparent)
							.cloned(),
					};
					let texture = texture.fix_wh_square(16);
					let texture = texture.margin(2);

					let count = format!("{received} / {target}");
					let count = sui::Text::new(count, 18);
					let count = count.centered().margin(2);

					let row = sui::div_h([sui::custom(texture), sui::custom(count)]);
					row
				});
				let rows = rows.collect::<Vec<_>>();
				let rows = sui::div(rows);
				let rows = rows.margin(4);

				let title = sui::Text::new(t!("ui.resource-bank"), 24)
					.centered()
					.margin(4);
				let display = sui::div([sui::custom(title), sui::custom(rows)]);

				let display = display.margin(4);
				let display = display.center_y();

				Some(display)
			}
			None => None,
		}
	}
	pub async fn render_tick(&mut self) -> anyhow::Result<()> {
		match &self.render_data {
			Some(render_data) => {
				let progress_component = match self.progress_into_component() {
					Some(a) => a,
					None => return Ok(()),
				};

				render_data
					.display_tx
					.send(RemoteStageChange::simple(progress_component))
					.await
					.map_err(|err| anyhow!("{err}"))?;
				Ok(())
			}
			None => return Ok(()),
		}
	}
}
impl ResourceCounter {
	pub fn new(goal: Goal, resource_rx: mpsc::Receiver<EResource>) -> Self {
		Self {
			goal,
			resource_rx,

			resources: Default::default(),
			render_data: None,
		}
	}

	pub fn tick(&mut self) -> anyhow::Result<()> {
		loop {
			match self.resource_rx.try_recv() {
				Ok(resource) => {
					// add the received resource to the counter
					let count = self.resources.get(&resource).copied().unwrap_or_default();
					self.resources.insert(resource, count + 1);
				}
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => {
					return Err(anyhow!(
						"ResourceCounter's receiver disconnected: ChannelConsumer buildings destroyed or game got dropped"
					));
				}
			}
		}
		Ok(())
	}
	pub async fn tick_next(&mut self) -> anyhow::Result<()> {
		let next = self
			.resource_rx
			.recv()
			.await
			.ok_or_else(|| anyhow!("resource sender probably got dropped"))?;

		let count = self.resources.get(&next).copied().unwrap_or_default();
		self.resources.insert(next, count + 1);

		Ok(())
	}

	pub fn set_goal(&mut self, goal: Goal) {
		self.goal = goal;
	}

	/// does not tick the underlying counter so call self.tick or else it won't
	/// work and it'll be confusing
	pub fn is_reached(&self) -> bool {
		self.goal
			.resources
			.iter()
			.filter(|(resource, required_count)| {
				let count = self.resources.get(resource).copied().unwrap_or_default();

				**required_count <= count
			})
			.count() > 0
	}
}
