#[macro_export]
macro_rules! debug {
	($($arg:tt)*) => {{
		if DEBUG {
			let args: std::fmt::Arguments<'_> = std::format_args!($($arg)*);

			std::println!("{}:{}:{} | {}", std::file!(), std::line!(), std::column!(), args)
		}
	}};
}

#[macro_export]
macro_rules! println {
	($($arg:tt)*) => {{
		let args: std::fmt::Arguments<'_> = std::format_args!($($arg)*);
		std::println!("{}:{}:{} | {}", std::file!(), std::line!(), std::column!(), args)
	}};
}

#[macro_export]
macro_rules! eprintln {
	($($arg:tt)*) => {{
		let args: std::fmt::Arguments<'_> = std::format_args!($($arg)*);
		std::eprintln!("{}:{}:{} | {}", std::file!(), std::line!(), std::column!(), args)
	}};
}
