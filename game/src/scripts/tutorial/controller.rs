use std::{borrow::Cow, fmt::Debug, sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use rust_i18n::t;
use stage_manager_remote::{RemoteEvent, RemoteStageChange};
use sui::{Layable, LayableExt};
use tokio::sync::{
	Mutex, broadcast,
	mpsc::{self},
	oneshot,
};

use crate::{
	game::{Game, GameCommand, Tool},
	scripts::tips::{action, text_with_actions, text_with_actions_fullscreen},
	utils::CheckConnection,
	world::{EResource, Resource, buildings::EBuilding, maps::BuildingsMap},
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

	pub stage_size: (usize, usize),
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

	/// displays a simple page with a single action: continue
	pub async fn simple_page_with_continue(
		&mut self,
		text: impl Into<Cow<'static, str>>,
	) -> anyhow::Result<()> {
		self.simple_page_with_named_continue(text, t!("tutorial.continue"))
			.await
	}
	pub async fn simple_page_with_named_continue(
		&mut self,
		text: impl Into<Cow<'static, str>>,
		continue_name: impl Into<Cow<'static, str>>,
	) -> anyhow::Result<()> {
		self.send_stage_change(text_with_actions(
			text,
			[action(continue_name, TooltipPage::Continue)],
		))
		.await
		.map_err(|err| anyhow!("{err}"))?;

		let event = self.receive_stage_event().await?;
		match event {
			TooltipPage::Continue => {}
			_ => return Err(anyhow!("invalid tooltippage received in mined: {event:?}")),
		}
		Ok(())
	}

	/// does NOT wait for the GameRunner to execute the Fn
	pub async fn game<F: FnOnce(&mut Game) + Send + 'static>(
		&mut self,
		f: F,
	) -> anyhow::Result<()> {
		let command = GameCommand::new(f);
		self.game_tx
			.send(command)
			.await
			.map_err(|err| anyhow!("channel.game failed: {err}"))?;
		Ok(())
	}
	/// waits for the GameRunner to actually execute the Fn sent
	pub async fn game_with_return<
		R: Debug + Send + 'static,
		F: FnOnce(&mut Game) -> R + Send + 'static,
	>(
		&mut self,
		f: F,
	) -> anyhow::Result<R> {
		let (command, rx) = GameCommand::new_return(f);
		self.game_tx
			.send(command)
			.await
			.map_err(|err| anyhow!("channel.game_with_return failed: {err}"))?;

		let ret = rx.await?;
		Ok(ret)
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
			t!("tutorial.welcome-to-conveyor-game"),
			[
				action(t!("tutorial.what-is-this"), TooltipPage::WhatIsThis),
				action(t!("tutorial.lets-get-started"), TooltipPage::GetStarted),
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
	channels
		.send_stage_change(text_with_actions_fullscreen(
			t!("tutorial.game-about"),
			[action(t!("tutorial.okay-sure"), TooltipPage::Reset)],
		))
		.await?;

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
			t!("tutorial.the-main-challenge-timer"),
			[action(t!("tutorial.continue"), TooltipPage::Continue)],
		))
		.await?;

	let event = channels.receive_stage_event().await?;
	match event {
		TooltipPage::Continue => {}
		_ => return Err(anyhow!("incorrect tooltippage {event:?} received")),
	}

	channels
		.send_stage_change(text_with_actions_fullscreen(
			t!("tutorial.lets-get-started-end-fullscreen"),
			[action(t!("tutorial.continue"), TooltipPage::Continue)],
		))
		.await?;

	let event = channels.receive_stage_event().await?;
	match event {
		TooltipPage::Continue => {}
		_ => return Err(anyhow!("incorrect tooltippage received")),
	}

	channels
		.game(|game| {
			game.enable_timer(Duration::from_secs(60 * 5));
			game.pause_time();
		})
		.await?;

	start_extracting(channels).await?;

	Ok(())
}

pub async fn start_extracting(channels: &mut Channels) -> anyhow::Result<()> {
	channels
		.simple_page_with_continue(t!("tutorial.resources-are-extracted-using-extractors"))
		.await?;

	channels
		.send_stage_change(text_with_actions(
			t!("tutorial.select-the-small-extractor"),
			[
				action(t!("tutorial.go-back"), TooltipPage::Reset),
				action(
					t!("tutorial.what-am-i-supposed-to-do"),
					TooltipPage::WhatAmISupposedToDo,
				),
			],
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
					Ok((pos, Tool::PlaceBuilding(EBuilding::SmallExtractor(_)))) => {
						return Some(pos);
					}
					Err(broadcast::error::RecvError::Closed) => return None,
					_ => continue,
				}
			}
		};

		tokio::select! {
			res = back_pressed => match res {
				1 => return Ok(()),
				2 => {
					channels.simple_page_with_continue(t!("tutorial.to-make-anything")).await?;

					channels.send_stage_change(text_with_actions(t!("tutorial.to-continue-the-tutorial"), [
						action(t!("tutorial.go-back"), TooltipPage::Reset)
					])).await?;

					continue;
				}
				invalid => return Err(anyhow!("invalid return value {invalid} received from back_pressed"))
			},
			res = extractor_placed => if let Some(pos) = res {
				let again = mined(channels, pos).await?;
				if again {
					continue;
				} else {
					break Ok(());
				}
			}
		};
	}
}

/// returns true if the extractor should be placed again
async fn mined(channels: &mut Channels, pos: (i32, i32)) -> anyhow::Result<bool> {
	let tile_resource = channels
		.game_with_return(move |game| game.tile_resource_at(pos))
		.await?;
	let tile_resource_name = match tile_resource {
		None => {
			// player put the extractor over fucking stone
			channels
				.send_stage_change(text_with_actions::<TooltipPage>(
					t!("tutorial.extractors-placed-over-stone"),
					[],
				))
				.await?;

			return Ok(true);
		}
		Some(res) => res.name(),
	};

	channels
		.simple_page_with_continue(t!(
			"tutorial.this-extractor-will-begin-mining",
			resource_name = tile_resource_name
		))
		.await?;

	channels
		.simple_page_with_continue(t!("tutorial.before-we-do-that"))
		.await?;
	channels
		.simple_page_with_named_continue(
			t!("tutorial.in-the-middle-of-the-screen"),
			t!("tutorial.yes-what-about-it"),
		)
		.await?;
	channels
		.simple_page_with_continue(t!("tutorial.this-is-the-central-building"))
		.await?;

	channels
		.simple_page_with_named_continue(
			t!("tutorial.resources-are-moved-using-conveyors"),
			t!("tutorial.okay-im-ready"),
		)
		.await?;

	channels
		.send_stage_change(text_with_actions::<TooltipPage>(
			t!("tutorial.place-conveyors"),
			[],
		))
		.await?;

	{
		// this block contains the code used to check if the miner's connected to the center building or nah

		let mut test_buildmap = BuildingsMap::new(channels.stage_size.0, channels.stage_size.1);
		super::place_at_center(&mut test_buildmap, EBuilding::debug_consumer());

		let targets = test_buildmap
			.iter()
			.filter_map(|(c, b)| match b {
				EBuilding::DebugConsumer(_) => Some(c),
				_ => None,
			})
			.collect::<Vec<_>>();
		std::mem::drop(test_buildmap);

		loop {
			tokio::time::sleep(Duration::from_millis(750)).await;

			let data = channels
				.game_with_return(|game| game.data().clone())
				.await
				.with_context(|| {
					format!("while cloning the Game's data to the controller thread")
				})?;

			let is_connected = data.is_connected(pos, targets.as_ref());
			let is_connected = match is_connected {
				Ok(a) => a,
				Err(err) => {
					eprintln!("connection checker error: {err:?}");
					false
				}
			};
			println!("targets were: {targets:?}");

			if is_connected {
				break;
			}
		}
	}

	channels
		.simple_page_with_named_continue(
			"fuck yeah",
			"this is the end for now so pressing this will take you back to the start",
		)
		.await?;

	// TODO connection checker utility
	// we should poll the game like twice a second to check if the miner we placed has a way to go into any of the collector buildings
	// if yes, continue

	tokio::time::sleep(Duration::from_millis(5000)).await;
	Ok(false)
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
