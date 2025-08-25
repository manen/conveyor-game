use std::fmt::Display;

pub trait SilentUnwrap {
	fn silent_unwrap(self);
}
impl<T, E: Display> SilentUnwrap for Result<T, E> {
	fn silent_unwrap(self) {
		match self {
			Ok(_) => {}
			Err(err) => {
				mklogger::eprintln!("silent error: {err}")
			}
		}
	}
}
