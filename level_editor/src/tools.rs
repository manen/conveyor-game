use std::fmt::Debug;

use game::world::{ETile, Tile};
use strum::IntoEnumIterator;
use sui::{Layable, LayableExt};

/// requests the changing of the tile we're placing
#[derive(Clone, Debug)]
pub struct TileChange(pub ETile);

fn tools() -> impl Iterator<Item = ETile> {
	ETile::iter()
}

/// creates the toolbar layout. listen to [SelectTool] in your component to have it working
pub fn toolbar() -> impl Layable + Clone + Debug {
	toolbar_from_tools(tools())
}
pub fn toolbar_from_tools(tools: impl Iterator<Item = ETile>) -> impl Layable + Clone + Debug {
	let toolbar = tools.map(|tool| {
		sui::Text::new(tool.name(), 24)
			.margin(4)
			.clickable(move |_| TileChange(tool.clone()))
	});

	let toolbar = sui::comp::div::SpaceBetween::new_horizontal(toolbar.collect::<Vec<_>>());

	toolbar
}
