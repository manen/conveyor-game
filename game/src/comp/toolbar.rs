use std::fmt::Debug;

use sui::{Layable, LayableExt};

use crate::{
	textures::Textures,
	utils::Direction,
	world::{
		buildings::{Building, EBuilding},
		tool::Tool,
	},
};

#[derive(Clone, Debug)]
/// the ReturnEvent sent back by the component
pub struct SelectTool(pub Tool);

fn tools() -> impl Iterator<Item = Tool> {
	use std::iter;

	iter::once(Tool::PlaceBuilding(EBuilding::nothing()))
		.chain(Direction::all().map(|dir| Tool::PlaceBuilding(EBuilding::conveyor(dir))))
		.chain([
			Tool::PlaceBuilding(EBuilding::small_extractor()),
			Tool::PlaceBuilding(EBuilding::debug_consumer()),
		])
}

/// creates the toolbar layout. listen to [SelectTool] in your component to have it working
pub fn toolbar(textures: &Textures) -> impl Layable + Clone + Debug + 'static {
	toolbar_from_tools(textures, tools())
}
pub fn toolbar_from_tools(
	textures: &Textures,
	tools: impl Iterator<Item = Tool>,
) -> impl Layable + Clone + Debug + 'static {
	let toolbar = tools.map(|tool| {
		let texture = match tool.clone() {
			Tool::PlaceBuilding(building) => sui::custom(building.tool_icon_render(textures)),

			#[allow(unreachable_patterns)]
			_ => sui::custom(
				textures
					.texture_for(tool.texture_id())
					.expect("no texture for tool texture")
					.clone(),
			),
		};

		let texture = texture.fix_wh_square(64);
		let texture = super::TooltipOnHover::new(tool.name(), texture);

		// sui::Text::new(tool.name(), 24)
		texture
			.margin(4)
			.clickable(move |_| SelectTool(tool.clone()))
	});

	let toolbar = toolbar.collect::<Vec<_>>();

	// let toolbar = sui::comp::div::SpaceBetween::new_horizontal(toolbar);
	let toolbar = sui::div_h(toolbar);

	toolbar
}
