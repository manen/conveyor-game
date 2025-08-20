use std::fmt::{Debug, Display};

use stage_manager::StageChange;
use sui::{Layable, LayableExt};

pub fn err_page_customizable<E: Debug + Display>(
	err: E,
	mut return_to_menu: Option<StageChange<'static>>,
) -> impl Layable + Debug + 'static {
	let display = format!("{err}");
	let debug = format!("{err:#?}");

	let err_info = sui::div([
		sui::custom(sui::text(display, 32).margin(16).centered()),
		sui::custom(sui::text(debug, 24)),
	]);

	let return_to_menu = match return_to_menu {
		Some(_) => vec![sui::text("return to main menu", 24).clickable(move |_| {
			return_to_menu
				.take()
				.expect("the main menu stagechange was taken already")
		})],
		None => vec![],
	};
	let return_to_menu = sui::div(return_to_menu).to_bottom();
	err_info.overlay(return_to_menu)
}
