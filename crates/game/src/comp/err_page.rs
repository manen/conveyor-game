use std::fmt::{Debug, Display};

use stage_manager::StageChange;
use sui::{DynamicLayable, Layable};

use crate::scripts::main::main_menu;

pub fn err_page<E: Debug + Display>(err: E) -> impl Layable + Debug + 'static {
	let main_menu = main_menu();
	let main_menu = stage_manager_loaders::Loader::new_invisible(main_menu, |a| {
		StageChange::simple_only_debug(a)
	});

	comp_extra::err_page_customizable(err, Some(main_menu))
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
