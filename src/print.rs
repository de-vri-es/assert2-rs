use std::os::raw::c_int;
use yansi::Paint;

use std::fmt::Debug;

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

fn print_assertion_failed(file: &str, line: u32, column: u32) {
	eprintln!("{msg} at {file}{colon}{line}{colon}{column}{bcolon}",
		msg    = Paint::red("Assertion failed").bold(),
		file   = Paint::default(file).bold(),
		line   = line,
		column = column,
		colon  = Paint::blue(":"),
		bcolon = Paint::default(":").bold(),
	);
}

#[allow(clippy::too_many_arguments)]
pub fn binary_failure<Left: Debug, Right: Debug>(
	name: &str,
	left: Left,
	right: Right,
	op_str: &str,
	left_expr: &str,
	right_expr: &str,
	file: &str,
	line: u32,
	column: u32,
) {
	set_color();
	print_assertion_failed(file, line, column);
	eprintln!("  {name}{open} {left} {op} {right} {close}",
		name  = Paint::magenta(name),
		open  = Paint::magenta("!("),
		left  = Paint::cyan(left_expr),
		op    = Paint::blue(op_str).bold(),
		right = Paint::yellow(right_expr),
		close = Paint::magenta(")"),
	);
	eprintln!("{}", Paint::default("with expansion:").bold());
	eprintln!("  {left:?} {op} {right:?}",
		left  = Paint::cyan(left),
		op    = Paint::blue(op_str).bold(),
		right = Paint::yellow(right),
	);
}

pub fn bool_failure<Value: Debug>(
	name: &str,
	value: Value,
	expr: &str,
	file: &str,
	line: u32,
	column: u32,
) {
	set_color();
	print_assertion_failed(file, line, column);
	eprintln!("  {name}{open} {expr} {close}",
		name  = Paint::magenta(name),
		open  = Paint::magenta("!("),
		expr  = Paint::cyan(expr),
		close = Paint::magenta(")"),
	);
	eprintln!("{}", Paint::default("with expansion:").bold());
	eprintln!("  {:?}", Paint::cyan(value));
}

pub fn match_failure<Value: Debug>(
	name: &str,
	value: Value,
	pat: &str,
	expr: &str,
	file: &str,
	line: u32,
	column: u32,
) {
	set_color();
	print_assertion_failed(file, line, column);
	eprintln!("  {name}{open} {let_} {pat} {eq} {expr} {close}",
		name  = Paint::magenta(name),
		open  = Paint::magenta("!("),
		let_  = Paint::blue("let").bold(),
		pat   = Paint::cyan(pat),
		eq    = Paint::blue("=").bold(),
		expr  = Paint::yellow(expr),
		close = Paint::magenta(")"),
	);
	eprintln!("{}", Paint::default("with expansion:").bold());
	eprintln!("  {:?}", Paint::yellow(value));
}

pub fn user_message_prefix() {
	eprint!("{}\n  ", Paint::default("with message:").bold())
}
