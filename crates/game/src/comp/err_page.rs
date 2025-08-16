use std::fmt::{Debug, Display};

use stage_manager::StageChange;
use sui::{DynamicLayable, Layable, LayableExt};

use crate::scripts::main::main_menu;

pub fn err_page_customizable<E: Debug + Display>(
	err: E,
	mut return_to_menu: Option<StageChange<'static>>,
) -> impl Layable + Debug + 'static {
	let display = format!("{err}");
	let debug = format!("{err:?}");

	let err_info = sui::div([
		sui::custom(sui::text(display, 32).centered()),
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

pub fn err_page<E: Debug + Display>(err: E) -> impl Layable + Debug + 'static {
	let main_menu = main_menu();
	let main_menu = stage_manager_loaders::Loader::new_invisible(main_menu, |a| {
		StageChange::simple_only_debug(a)
	});

	err_page_customizable(err, Some(main_menu))
}

pub fn handle_result<E: Debug + Display, T: Layable + Debug + 'static>(
	res: Result<T, E>,
) -> DynamicLayable<'static> {
	handle_result_dyn(res.map(DynamicLayable::new_only_debug))
}
pub fn handle_result_dyn<E: Debug + Display>(
	res: Result<DynamicLayable<'static>, E>,
) -> DynamicLayable<'static> {
	match res {
		Ok(a) => DynamicLayable::new_only_debug(a),
		Err(err) => DynamicLayable::new_only_debug(err_page(err)),
	}
}

pub fn handle_err<E: Debug + Display, T: Layable + Debug + 'static, F: FnOnce() -> Result<T, E>>(
	f: F,
) -> DynamicLayable<'static> {
	handle_result(f())
}
pub async fn handle_err_async<
	E: Debug + Display,
	T: Layable + Debug + 'static,
	F: Future<Output = Result<T, E>>,
	Fn: FnOnce() -> F,
>(
	f: Fn,
) -> DynamicLayable<'static> {
	handle_result(f().await)
}

pub fn handle_err_dyn<E: Debug + Display, F: FnOnce() -> Result<DynamicLayable<'static>, E>>(
	f: F,
) -> DynamicLayable<'static> {
	handle_result_dyn(f())
}
pub async fn handle_err_async_dyn<
	E: Debug + Display,
	F: Future<Output = Result<DynamicLayable<'static>, E>>,
	Fn: FnOnce() -> F,
>(
	f: Fn,
) -> DynamicLayable<'static> {
	handle_result_dyn(f().await)
}
