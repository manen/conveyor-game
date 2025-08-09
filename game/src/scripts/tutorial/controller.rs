use std::{borrow::Cow, fmt::Debug, time::Duration};

use anyhow::{Context, anyhow};
use stage_manager_remote::{RemoteEvent, RemoteStageChange};
use sui::{Layable, LayableExt};
use tokio::sync::{
	broadcast,
	mpsc::{self},
};

use crate::{
	game::Tool,
	scripts::tips::{action, text_with_actions, text_with_actions_fullscreen},
	world::buildings::EBuilding,
};

#[derive(Clone, Debug)]
pub enum TooltipPage {
	Reset,
	WhatIsThis,
	GetStarted,
	Continue,
	WhatAmISupposedToDo,
}

#[derive(Debug)]
pub struct Channels {
	pub stage_tx: mpsc::Sender<RemoteStageChange>,
	pub stage_rx: mpsc::Receiver<TooltipPage>,
	pub tool_use_rx: broadcast::Receiver<((i32, i32), Tool)>,
	pub game_tx: mpsc::Sender<crate::game::GameCommand>,
}
impl Channels {
	pub async fn send_stage<L: Layable + Debug + 'static>(
		&mut self,
		new_stage: L,
	) -> anyhow::Result<()> {
		self.send_stage_change(RemoteStageChange::simple_only_debug(new_stage))
			.await
	}
	pub async fn send_stage_change(
		&mut self,
		stage_change: RemoteStageChange,
	) -> anyhow::Result<()> {
		self.stage_tx
			.send(stage_change)
			.await
			.map_err(|err| anyhow!("error while sending stage change:\n{err}"))
	}

	pub async fn receive_stage_event(&mut self) -> anyhow::Result<TooltipPage> {
		drain_mpsc(&mut self.stage_rx);
		self.stage_rx
			.recv()
			.await
			.ok_or_else(|| anyhow!("expected to receive TooltipPage from stage_rx"))
	}
}

pub async fn controller(mut channels: Channels) {
	loop {
		match welcome(&mut channels).await {
			Ok(a) => a,
			Err(err) => {
				eprintln!("tooltip thread caught an error: {err}")
			}
		}
	}
}

pub async fn welcome(channels: &mut Channels) -> anyhow::Result<()> {
	channels
		.stage_tx
		.send(text_with_actions_fullscreen(
			"welcome to conveyor-game!",
			[
				action("what is this", TooltipPage::WhatIsThis),
				action("let's get started!", TooltipPage::GetStarted),
			],
		))
		.await
		.map_err(|err| anyhow!("{err}"))?;

	let event = channels
		.stage_rx
		.recv()
		.await
		.with_context(|| format!("welcome didn't receive anything"))?;

	match event {
		TooltipPage::WhatIsThis => what_is_this(channels).await?,
		TooltipPage::GetStarted => get_started(channels).await?,
		_ => {
			return Err(anyhow!(
				"unexpected page {event:?} received in what_is_this"
			));
		}
	}
	Ok(())
}

pub async fn what_is_this(channels: &mut Channels) -> anyhow::Result<()> {
	channels.send_stage_change(text_with_actions_fullscreen("conveyor-game is a game about factories and resources.\nuse the buildings available to extract, smelt, and produce resources.\nthe rest of the tutorial will teach you all about playing the game.", [
		action("okay sure", TooltipPage::Reset)
	])).await?;

	let event = channels
		.stage_rx
		.recv()
		.await
		.with_context(|| format!("what_is_this didn't receive anything"))?;

	match event {
		TooltipPage::Reset => Ok(()),
		_ => Err(anyhow!(
			"unexpected page {event:?} received in what_is_this"
		)),
	}
}

pub async fn get_started(channels: &mut Channels) -> anyhow::Result<()> {
	channels
		.send_stage_change(text_with_actions_fullscreen(
			"the main challenge of this game is the timer. the game starts with the timer stopped, and it's recommended to only start the timer once you've finished building.",
			[action("continue", TooltipPage::Continue)],
		))
		.await?;

	let event = channels.receive_stage_event().await?;
	match event {
		TooltipPage::Continue => {}
		_ => return Err(anyhow!("incorrect tooltippage received")),
	}

	channels.send_stage_change(text_with_actions_fullscreen("let's get started!\nwhen you click continue, the tutorial will be moved to the bottom-left corner of the screen.", [
		action("continue", TooltipPage::Continue)
	])).await?;

	let event = channels.receive_stage_event().await?;
	match event {
		TooltipPage::Continue => {}
		_ => return Err(anyhow!("incorrect tooltippage received")),
	}

	start_extracting(channels).await?;

	Ok(())
}

pub async fn start_extracting(channels: &mut Channels) -> anyhow::Result<()> {
	channels
		.send_stage_change(text_with_actions(
			"resources are extracted using extractors. you can see the blocks available to you on the toolbar at the top.",
			[action("continue", TooltipPage::Continue)],
		))
		.await?;

	let event = channels.receive_stage_event().await?;
	match event {
		TooltipPage::Continue => {}
		_ => return Err(anyhow!("incorrect tooltippage received")),
	}

	channels.send_stage_change(text_with_actions(
		"select the small extractor from the toolbar at the top, and place it (left click) over any resource",
		[action("go back", TooltipPage::Reset), action("what am i supposed to do again?", TooltipPage::WhatAmISupposedToDo)],
	))
	.await?;
	loop {
		let back_pressed = async {
			loop {
				match channels.stage_rx.recv().await {
					Some(TooltipPage::Reset) => return 1,
					Some(TooltipPage::WhatAmISupposedToDo) => return 2,
					_ => continue,
				}
			}
		};

		drain_broadcast(&mut channels.tool_use_rx);
		let extractor_placed = async {
			loop {
				match channels.tool_use_rx.recv().await {
					Ok((_, Tool::PlaceBuilding(EBuilding::SmallExtractor(_)))) => return true,
					Err(broadcast::error::RecvError::Closed) => return false,
					_ => continue,
				}
			}
		};

		tokio::select! {
			res = back_pressed => match res {
				1 => return Ok(()),
				2 => {
					channels.send_stage_change(text_with_actions("to make anything, we need to start by extracting raw materials from the ground. we can do that using the small extractor, found on the toolbar at the top of the screen.", [
							action("continue", TooltipPage::Continue)
						])).await?;

					let event = channels.receive_stage_event().await?;
					match event {
						TooltipPage::Continue => {}
						_ => return Err(anyhow!("invalid tooltippage received"))
					}

					channels.send_stage_change(text_with_actions("to continue the tutorial, select the small extractor from the toolbar and place it over a resource you'd like to mine", [
						action("go back", TooltipPage::Reset)
					])).await?;

					continue;
				}
				_ => return Err(anyhow!("invalid return value from back_pressed"))
			},
			res = extractor_placed => if res {
				return mined(channels).await;
			}
		};
	}
}

async fn mined(channels: &mut Channels) -> anyhow::Result<()> {
	channels
		.send_stage_change(text_with_actions::<TooltipPage>("just like that bro", []))
		.await
		.map_err(|err| anyhow!("{err}"))?;

	tokio::time::sleep(Duration::from_millis(5000)).await;
	Ok(())
}

fn drain_broadcast<T: Clone>(rx: &mut broadcast::Receiver<T>) {
	loop {
		match rx.try_recv() {
			Ok(_) => (),
			Err(broadcast::error::TryRecvError::Closed) => return,
			Err(_) => break,
		}
	}
}
fn drain_mpsc<T>(rx: &mut mpsc::Receiver<T>) {
	loop {
		match rx.try_recv() {
			Ok(_) => (),
			Err(mpsc::error::TryRecvError::Disconnected) => return,
			Err(_) => break,
		}
	}
}
