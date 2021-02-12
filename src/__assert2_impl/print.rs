use std::os::raw::c_int;
use yansi::Paint;
use atty::Stream;

use std::fmt::Debug;

fn should_color() -> bool {
	if std::env::var_os("CLICOLOR").map(|x| x == "0").unwrap_or(false) {
		false
	} else if std::env::var_os("CLICOLOR_FORCE").map(|x| x != "0").unwrap_or(false) {
		true
	} else {
		atty::is(Stream::Stderr)
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
	pub fragments: &'a [(&'a str, &'a str)],
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
		if !self.fragments.is_empty() {
			eprintln!("with:");
			for (name, expansion) in self.fragments {
				eprintln!("  {} {} {}", Paint::magenta(name), Paint::blue("=").bold(), expansion);
			}
		}
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
