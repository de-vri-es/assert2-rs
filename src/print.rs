use crate::maybe_debug::MaybeDebug;
use crate::maybe_debug::wrap;
use std::os::raw::c_int;
use yansi::Paint;

extern "C" {
	fn isatty(fd: c_int) -> c_int;
}

fn stderr_is_tty() -> bool {
	unsafe { isatty(2) != 0 }
}

fn should_color() -> bool {
	// CLICOLOR not set? Check if stderr is a TTY.
	let clicolor = match std::env::var_os("CLICOLOR") {
		Some(x) => x,
		None => return stderr_is_tty(),
	};

	// CLICOLOR not ascii? Disable colors.
	let clicolor = match clicolor.to_str() {
		Some(x) => x,
		None => return false,
	};

	let force = false;
	let force = force || clicolor.eq_ignore_ascii_case("yes");
	let force = force || clicolor.eq_ignore_ascii_case("true");
	let force = force || clicolor.eq_ignore_ascii_case("always");
	let force = force || clicolor.eq_ignore_ascii_case("1");

	if force {
		true
	} else if clicolor.eq_ignore_ascii_case("auto") {
		stderr_is_tty()
	} else {
		false
	}
}

fn set_color() {
	if should_color() {
		Paint::enable()
	} else {
		Paint::disable()
	}
}

pub fn binary_failure<Left: MaybeDebug, Right: MaybeDebug>(
	left: &Left,
	right: &Right,
	op_str: &str,
	left_expr: &str,
	right_expr: &str,
	file: &str,
	line: u32,
	column: u32,
) {
	set_color();
	eprintln!("{msg} at {file}{colon}{line}{colon}{column}",
		msg    = Paint::red("Assertion failed").bold(),
		file   = Paint::default(file).bold(),
		line   = line,
		column = column,
		colon  = Paint::blue(":"),
	);
	eprintln!("  {check} {left} {op} {right} {close}",
		check = Paint::magenta("check!("),
		left  = Paint::cyan(left_expr),
		op    = Paint::blue(op_str).bold(),
		right = Paint::yellow(right_expr),
		close = Paint::magenta(")"),
	);
	eprintln!("{}", Paint::default("with expansion:").bold());
	eprintln!("  {left:?} {op} {right:?}",
		left  = Paint::cyan(wrap(left)),
		op    = Paint::blue(op_str).bold(),
		right = Paint::yellow(wrap(right)),
	);
}

pub fn bool_failure<Value: MaybeDebug>(
	value: &Value,
	expr: &str,
	file: &str,
	line: u32,
	column: u32,
) {
	set_color();
	eprintln!("{msg} at {file}{colon}{line}{colon}{column}",
		msg    = Paint::red("Assertion failed").bold(),
		file   = Paint::default(file).bold(),
		line   = line,
		column = column,
		colon  = Paint::blue(":"),
	);
	eprintln!("  {check} {expr} {close}",
		check = Paint::magenta("check!("),
		expr  = Paint::cyan(expr),
		close = Paint::magenta(")"),
	);
	eprintln!("{}", Paint::default("with expansion:").bold());
	eprintln!("  {:?}", Paint::cyan(wrap(value)));
}
