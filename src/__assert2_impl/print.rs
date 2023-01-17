use atty::Stream;
use std::fmt::Debug;
use yansi::Paint;
use std::fmt::Write;

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
	fn print_expression(&self, print_message: &mut  String);
	fn print_expansion(&self, print_message: &mut String);
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
		let mut print_message = String::new();
		writeln!(&mut print_message, "{msg} at {file}:{line}:{column}:",
			msg    = Paint::red("Assertion failed").bold(),
			file   = Paint::default(self.file).bold(),
			line   = self.line,
			column = self.column,
		).unwrap();
		write!(&mut print_message, "  {name}{open} ",
			name = Paint::magenta(self.macro_name),
			open = Paint::magenta("!("),
		).unwrap();
		self.expression.print_expression(&mut print_message);
		writeln!(&mut print_message, " {}", Paint::magenta(")")).unwrap();
		if !self.fragments.is_empty() {
			writeln!(&mut print_message, "with:").unwrap();
			for (name, expansion) in self.fragments {
				writeln!(&mut print_message, "  {} {} {}", Paint::magenta(name), Paint::blue("=").bold(), expansion).unwrap();
			}
		}
		writeln!(&mut print_message, "with expansion:").unwrap();
		write!(&mut print_message, "  ").unwrap();
		self.expression.print_expansion(&mut print_message);
		writeln!(&mut print_message, ).unwrap();
		if let Some(msg) = self.custom_msg {
			writeln!(&mut print_message, "with message:").unwrap();
			writeln!(&mut print_message, "  {}", Paint::default(msg).bold()).unwrap();
		}
		writeln!(&mut print_message).unwrap();
		
		eprint!("{}", print_message);
	}
}

#[rustfmt::skip]
impl<Left: Debug, Right: Debug> CheckExpression for BinaryOp<'_, Left, Right> {
	fn print_expression(&self, print_message: &mut  String) {
		write!(print_message, "{left} {op} {right}",
			left  = Paint::cyan(self.left_expr),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right_expr),
		).unwrap();
	}
	fn print_expansion(&self, print_message: &mut  String) {
		write!(print_message, "{left:?} {op} {right:?}",
			left  = Paint::cyan(self.left),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right),
		).unwrap();
	}
}

#[rustfmt::skip]
impl CheckExpression for BooleanExpr<'_> {
	fn print_expression(&self, print_message: &mut  String) {
		write!(print_message, "{}", Paint::cyan(self.expression)).unwrap();
	}
	fn print_expansion(&self, print_message: &mut String) {
		write!(print_message, "{:?}", Paint::cyan(false)).unwrap();
	}
}

#[rustfmt::skip]
impl<Value: Debug> CheckExpression for MatchExpr<'_, Value> {
	fn print_expression(&self, print_message: &mut String) {
		if self.print_let {
			write!(print_message, "{} ", Paint::blue("let").bold()).unwrap();
		}
		write!(print_message, "{pat} {eq} {expr}",
			pat  = Paint::cyan(self.pattern),
			eq   = Paint::blue("=").bold(),
			expr = Paint::yellow(self.expression),
		).unwrap();
	}
	fn print_expansion(&self, print_message: &mut String) {
		write!(print_message, "{:?}", Paint::yellow(self.value)).unwrap();
	}
}
