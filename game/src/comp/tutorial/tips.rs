use std::{borrow::Cow, fmt::Debug, time::Duration};

use anyhow::{Context, anyhow};
use stage_manager_remote::{RemoteEvent, RemoteStageChange};
use sui::{Layable, LayableExt};
use tokio::sync::{
	broadcast,
	mpsc::{Receiver, Sender},
};

use crate::world::{buildings::EBuilding, tool::Tool};

#[derive(Clone, Debug)]
pub enum TooltipPage {
	Reset,
	WhatIsThis,
	GetStarted,
}

type Tx = Sender<stage_manager_remote::RemoteStageChange>;
type Rx = Receiver<TooltipPage>;

type ToolUse = broadcast::Receiver<((i32, i32), Tool)>;

pub async fn controller(mut tx: Tx, mut rx: Rx, mut tool_use: ToolUse) {
	loop {
		match welcome(&mut tx, &mut rx, &mut tool_use).await {
			Ok(a) => a,
			Err(err) => {
				eprintln!("tooltip thread caught an error: {err}")
			}
		}
	}
}

#[derive(Clone, Debug)]
pub struct Action {
	name: Cow<'static, str>,
	page: TooltipPage,
}
impl Action {
	fn into_button(self) -> impl Layable + Debug {
		let button = sui::text(self.name, 16).clickable(move |_| RemoteEvent(self.page.clone()));
		button
	}
}
pub fn action(name: impl Into<Cow<'static, str>>, page: TooltipPage) -> Action {
	Action {
		name: name.into(),
		page,
	}
}

pub fn text_with_actions(
	text: impl Into<Cow<'static, str>>,
	actions: impl IntoIterator<Item = Action>,
) -> RemoteStageChange {
	let text = sui::comp::WrappedText::new(text, 24).margin(4);

	let actions = actions
		.into_iter()
		.map(|action| action.into_button().margin_h(4))
		.map(sui::custom_only_debug);
	let actions = sui::div(actions.collect::<Vec<_>>());

	let div = sui::div([sui::custom(text), sui::custom_only_debug(actions)]);
	RemoteStageChange::simple(div)
}

pub async fn welcome(tx: &mut Tx, rx: &mut Rx, tool_use: &mut ToolUse) -> anyhow::Result<()> {
	tx.send(text_with_actions(
		"welcome to conveyor-game!",
		[
			action("what is this", TooltipPage::WhatIsThis),
			action("let's get started!", TooltipPage::GetStarted),
		],
	))
	.await
	.map_err(|err| anyhow!("{err}"))?;

	let event = rx
		.recv()
		.await
		.with_context(|| format!("welcome didn't receive anything"))?;

	match event {
		TooltipPage::WhatIsThis => what_is_this(tx, rx).await?,
		TooltipPage::GetStarted => get_started(tx, rx, tool_use).await?,
		_ => {
			return Err(anyhow!(
				"unexpected page {event:?} received in what_is_this"
			));
		}
	}
	Ok(())
}

pub async fn what_is_this(tx: &mut Tx, rx: &mut Rx) -> anyhow::Result<()> {
	tx.send(text_with_actions("this is the tutorial, and these tooltips are going to help you get the gist of the game. if you've played conveyor/factory games before, it'll feel familiar.", [
		action("okay sure", TooltipPage::Reset)
	])).await.map_err(|err| anyhow!("{err}"))?;

	let event = rx
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

pub async fn get_started(tx: &mut Tx, rx: &mut Rx, tool_use: &mut ToolUse) -> anyhow::Result<()> {
	tx.send(text_with_actions(
		"to begin extracting resources, you'll need to place a small extractor. select the small extractor from the toolbar at the top, and place it (left click) over any resource",
		[action("go back", TooltipPage::Reset)],
	))
	.await
	.map_err(|err| anyhow!("{err}"))?;

	let back_pressed = async {
		loop {
			match rx.recv().await {
				Some(TooltipPage::Reset) => return true,
				_ => continue,
			}
		}
	};
	let extractor_placed = async {
		loop {
			match tool_use.recv().await {
				Ok((_, Tool::PlaceBuilding(EBuilding::SmallExtractor(_)))) => return true,
				Err(broadcast::error::RecvError::Closed) => return false,
				_ => continue,
			}
		}
	};

	tokio::select! {
		res = back_pressed => if res {
			return Ok(())
		},
		res = extractor_placed => if res {
			tx.send(text_with_actions("just like that bro", [])).await.map_err(|err| anyhow!("{err}"))?;
			tokio::time::sleep(Duration::from_millis(5000)).await;
		}
	};
	Ok(())
}
