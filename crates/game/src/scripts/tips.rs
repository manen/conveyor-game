use std::{borrow::Cow, fmt::Debug};

use stage_manager_remote::{RemoteEvent, RemoteStageChange};
use sui::{Layable, LayableExt};

#[derive(Clone, Debug)]
pub struct Action<P: Clone + 'static> {
	name: Cow<'static, str>,
	page: P,
}
impl<P: Clone + 'static> Action<P> {
	fn into_button(self) -> impl Layable + Debug {
		let button = 
			// crate::utils::ShowMouse::new
		// (
			(sui::text(self.name, 16)).clickable_fallback(move |_| RemoteEvent(self.page.clone()))
		// )
		;
		button
	}
}
pub fn action<P: Clone + 'static>(name: impl Into<Cow<'static, str>>, page: P) -> Action<P> {
	Action {
		name: name.into(),
		page,
	}
}

pub fn text_with_actions<P: Clone + 'static>(
	text: impl Into<Cow<'static, str>>,
	actions: impl IntoIterator<Item = Action<P>>,
) -> RemoteStageChange {
	let l = text_with_actions_l(text, actions);
	RemoteStageChange::simple(l)
}
pub fn text_with_actions_l<P: Clone + 'static>(
	text: impl Into<Cow<'static, str>>,
	actions: impl IntoIterator<Item = Action<P>>,
) -> impl Layable + Debug + Clone + 'static {
	let text = sui::comp::WrappedText::new(text, 24).margin(4);

	let actions = actions
		.into_iter()
		.map(|action| action.into_button().margin_h(4))
		.map(sui::custom_only_debug);
	let actions = sui::div(actions.collect::<Vec<_>>());

	let div = sui::div([sui::custom(text), sui::custom_only_debug(actions)]);

	div
}

/// doesn't use wrapped text
pub fn text_with_actions_fullscreen<P: Clone + 'static>(
	text: impl Into<Cow<'static, str>>,
	actions: impl IntoIterator<Item = Action<P>>,
) -> RemoteStageChange {
	let l = text_with_actions_fullscreen_l(text, actions);
	RemoteStageChange::simple(l)
}
pub fn text_with_actions_fullscreen_l<P: Clone + 'static>(
	text: impl Into<Cow<'static, str>>,
	actions: impl IntoIterator<Item = Action<P>>,
) -> impl Layable + Debug + Clone + 'static {
	let text = sui::comp::text::wrapped_text::CenteredWrappedText::new(text, 24).margin(4);

	let actions = actions
		.into_iter()
		.map(|action| {
			// action.into_button().margin(2).margin_h(2).centered();
			let button = action.into_button();
			let button = button.margin(2).margin_h(2);
			let button = button.centered();

			button
		})
		.map(sui::custom_only_debug);
	let actions = sui::div(actions.collect::<Vec<_>>()).margin_v(8);

	let div = sui::div([sui::custom(text), sui::custom_only_debug(actions)]);

	let div = crate::comp::FullscreenWrap::new(div.center_y());
	let div = div.margin(16);

	let div = div.with_background(sui::comp::Color::new(sui::color(0, 0, 0, 200)));
	div
}
