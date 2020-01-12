use std::os::raw::c_int;
use yansi::Paint;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

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

pub trait Diagnostic: Display {
	fn print(&self) {
		set_color();
		eprintln!("{}", self);
	}
}

impl<Left: Debug, Right: Debug> Diagnostic for BinaryOp<'_, Left, Right> {}
impl<Value: Debug> Diagnostic for BooleanExpr<'_, Value> {}
impl<Value: Debug> Diagnostic for MatchExpr<'_, Value> {}

pub struct BinaryOp<'a, Left, Right> {
	pub macro_name: &'a str,
	pub left: &'a Left,
	pub right: &'a Right,
	pub operator: &'a str,
	pub left_expr: &'a str,
	pub right_expr: &'a str,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
}

pub struct BooleanExpr<'a, Value> {
	pub macro_name: &'a str,
	pub value: &'a Value,
	pub expression: &'a str,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
}

pub struct MatchExpr<'a, Value> {
	pub macro_name: &'a str,
	pub value: &'a Value,
	pub pattern: &'a str,
	pub expression: &'a str,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
}

#[rustfmt::skip]
fn write_assertion_failed(f: &mut Formatter, file: &str, line: u32, column: u32) -> std::fmt::Result {
	write!(f, "{msg} at {file}{colon}{line}{colon}{column}:",
		msg    = Paint::red("Assertion failed").bold(),
		file   = Paint::default(file).bold(),
		line   = line,
		column = column,
		colon  = Paint::blue(":"),
	)
}

#[rustfmt::skip]
impl<Left: Debug, Right: Debug> Display for BinaryOp<'_, Left, Right> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write_assertion_failed(f, self.file, self.line, self.column)?;
		write!(f, "\n  {name}{open} {left} {op} {right} {close}",
			name  = Paint::magenta(self.macro_name),
			open  = Paint::magenta("!("),
			close = Paint::magenta(")"),
			left  = Paint::cyan(self.left_expr),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right_expr),
		)?;
		write!(f, "\n{}", Paint::default("with expansion:").bold())?;
		write!(f, "\n  {left:?} {op} {right:?}",
			left  = Paint::cyan(self.left),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right),
		)?;
		write_custom_message(f, &self.custom_msg)
	}
}

#[rustfmt::skip]
impl<Value: Debug> Display for BooleanExpr<'_, Value> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write_assertion_failed(f, self.file, self.line, self.column)?;
		write!(f, "\n  {name}{open} {expr} {close}",
			name  = Paint::magenta(self.macro_name),
			open  = Paint::magenta("!("),
			close = Paint::magenta(")"),
			expr = Paint::cyan(self.expression),
		)?;
		write!(f, "\n{}", Paint::default("with expansion:").bold())?;
		write!(f, "\n  {:?}", Paint::cyan(self.value))?;
		write_custom_message(f, &self.custom_msg)
	}
}

#[rustfmt::skip]
impl<Value: Debug> Display for MatchExpr<'_, Value> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write_assertion_failed(f, self.file, self.line, self.column)?;
		write!(f, "\n  {name}{open} {let_} {pat} {eq} {expr} {close}",
			name  = Paint::magenta(self.macro_name),
			open  = Paint::magenta("!("),
			close = Paint::magenta(")"),
			let_  = Paint::blue("let").bold(),
			pat   = Paint::cyan(self.pattern),
			eq    = Paint::blue("=").bold(),
			expr  = Paint::yellow(self.expression),
		)?;
		write!(f, "\n{}", Paint::default("with expansion:").bold())?;
		write!(f, "\n  {:?}", Paint::yellow(self.value))?;
		write_custom_message(f, &self.custom_msg)
	}
}

#[rustfmt::skip]
fn write_custom_message(f: &mut Formatter, msg: &Option<std::fmt::Arguments>) -> std::fmt::Result {
	if let Some(msg) = msg {
		write!(f, "\n{prefix}\n  {msg}",
			prefix = Paint::default("with message").bold(),
			msg    = Paint::default(msg),
		)
	} else {
		Ok(())
	}
}
