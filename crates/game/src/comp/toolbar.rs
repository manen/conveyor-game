use std::fmt::Debug;

use sui::{Layable, LayableExt};

use crate::{
	comp::{TooltipData, TooltipProvider},
	game::{Tool, tools},
	textures::Textures,
	world::buildings::Building,
};

#[derive(Clone, Debug)]
/// the ReturnEvent sent back by the component
pub struct SelectTool(pub Tool);

/// creates the toolbar layout. listen to [SelectTool] in your component to have it working
pub fn toolbar(textures: &Textures) -> impl Layable + Clone + Debug + 'static {
	toolbar_from_tools(textures, tools())
}
pub fn toolbar_from_tools(
	textures: &Textures,
	tools: impl Iterator<Item = Tool>,
) -> impl Layable + Clone + Debug + 'static {
	let tooltip_data = TooltipData::default();

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
		let texture = texture.margin(4);
		let texture = super::TooltipOnHover::new(tool.name(), tooltip_data.clone(), texture);

		// sui::Text::new(tool.name(), 24)
		texture.clickable(move |_| SelectTool(tool.clone()))
	});

	let toolbar = toolbar.collect::<Vec<_>>();

	// let toolbar = sui::comp::div::SpaceBetween::new_horizontal(toolbar);
	let toolbar = sui::div_h(toolbar);
	let toolbar = TooltipProvider::new_explicit(toolbar, tooltip_data);

	toolbar
}
