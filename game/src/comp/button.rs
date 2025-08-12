use std::{borrow::Cow, fmt::Debug};

use sui::{
	Layable, LayableExt,
	comp::Clickable,
	core::{Event, MouseEvent, ReturnEvent},
};

// #[derive(Clone, Debug)]
// pub struct Button<'a> {
// 	text: Cow<'a, str>,
// 	disabled: bool,
// }

pub fn button_explicit<'a, F: FnMut() -> ReturnEvent>(
	text: impl Into<Cow<'a, str>>,
	disabled: bool,
	mut gen_f: F,
) -> impl Layable + Debug {
	let text_color = if !disabled {
		sui::comp::text::DEFAULT_COLOR
	} else {
		sui::Color::DARKGRAY
	};
	let overlay_color = if !disabled {
		sui::color(0, 0, 0, 0)
	} else {
		sui::Color::GRAY.alpha(0.3)
	};
	let border_color = if !disabled {
		sui::color(25, 25, 25, 255)
	} else {
		sui::Color::BLACK
	};

	let text = sui::Text::new_colored(text, 24, text_color);

	let text = text.margin(4);
	let text = text.margin_h(8).margin_v(2);
	let text = text.with_background(sui::comp::Color::new(sui::Color::BLACK));

	let text = text
		.margin(1)
		.with_background(sui::comp::Color::new(border_color));

	let text = text.overlay(sui::comp::Color::new(overlay_color));
	let text = text.margin(5);

	let clickable = text.clickable_optional(move || if !disabled { Some(gen_f()) } else { None });
	let clickable = clickable;

	clickable
}
