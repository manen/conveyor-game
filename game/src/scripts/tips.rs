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
		let button = sui::text(self.name, 16).clickable(move |_| RemoteEvent(self.page.clone()));
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
	let text = sui::comp::WrappedText::new(text, 24).margin(4);

	let actions = actions
		.into_iter()
		.map(|action| action.into_button().margin_h(4))
		.map(sui::custom_only_debug);
	let actions = sui::div(actions.collect::<Vec<_>>());

	let div = sui::div([sui::custom(text), sui::custom_only_debug(actions)]);
	RemoteStageChange::simple(div)
}
