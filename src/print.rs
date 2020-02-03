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

pub struct FailedCheck<'a, T> {
	pub macro_name: &'a str,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub expression: T,
}

pub trait CheckExpression {
	fn print_expression(&self);
	fn print_expansion(&self);
}

pub struct BinaryOp<'a, Left, Right> {
	pub left: &'a Left,
	pub right: &'a Right,
	pub operator: &'a str,
	pub left_expr: &'a str,
	pub right_expr: &'a str,
}

pub struct BooleanExpr<'a> {
	pub expression: &'a str,
}

pub struct MatchExpr<'a, Value> {
	pub print_let: bool,
	pub value: &'a Value,
	pub pattern: &'a str,
	pub expression: &'a str,
}

impl<'a, T: CheckExpression> FailedCheck<'a, T> {
	#[rustfmt::skip]
	pub fn print(&self) {
		set_color();
		eprintln!("{msg} at {file}:{line}:{column}:",
			msg    = Paint::red("Assertion failed").bold(),
			file   = Paint::default(self.file).bold(),
			line   = self.line,
			column = self.column,
		);
		eprint!("  {name}{open} ",
			name = Paint::magenta(self.macro_name),
			open = Paint::magenta("!("),
		);
		self.expression.print_expression();
		eprintln!(" {}", Paint::magenta(")"));
		eprintln!("with expansion:");
		eprint!("  ");
		self.expression.print_expansion();
		eprintln!();
		if let Some(msg) = self.custom_msg {
			eprintln!("with message:");
			eprintln!("  {}", Paint::default(msg).bold());
		}
		eprintln!();
	}
}

#[rustfmt::skip]
impl<Left: Debug, Right: Debug> CheckExpression for BinaryOp<'_, Left, Right> {
	fn print_expression(&self) {
		eprint!("{left} {op} {right}",
			left  = Paint::cyan(self.left_expr),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right_expr),
		);
	}
	fn print_expansion(&self) {
		eprint!("{left:?} {op} {right:?}",
			left  = Paint::cyan(self.left),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right),
		);
	}
}

#[rustfmt::skip]
impl CheckExpression for BooleanExpr<'_> {
	fn print_expression(&self) {
		eprint!("{}", Paint::cyan(self.expression));
	}
	fn print_expansion(&self) {
		eprint!("{:?}", Paint::cyan(false));
	}
}

#[rustfmt::skip]
impl<Value: Debug> CheckExpression for MatchExpr<'_, Value> {
	fn print_expression(&self) {
		if self.print_let {
			eprint!("{} ", Paint::blue("let").bold());
		}
		eprint!("{pat} {eq} {expr}",
			pat  = Paint::cyan(self.pattern),
			eq   = Paint::blue("=").bold(),
			expr = Paint::yellow(self.expression),
		);
	}
	fn print_expansion(&self) {
		eprint!("{:?}", Paint::yellow(self.value));
	}
}
