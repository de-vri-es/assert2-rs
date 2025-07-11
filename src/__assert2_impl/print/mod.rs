use std::fmt::Debug;
use yansi::Paint;
use std::fmt::Write;

mod diff;
use self::diff::{MultiLineDiff, SingleLineDiff};

mod options;
use self::options::{AssertOptions, ExpansionFormat};

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
	fn write_expression(&self, buffer: &mut  String);
	fn write_expansion(&self, buffer: &mut String);
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
		let mut print_message = String::new();
		writeln!(&mut print_message, "{msg} at {file}:{line}:{column}:",
			msg    = "Assertion failed".red().bold(),
			file   = self.file.bold(),
			line   = self.line,
			column = self.column,
		).unwrap();
		write!(&mut print_message, "  {name}{open} ",
			name = Paint::magenta(self.macro_name),
			open = Paint::magenta("!("),
		).unwrap();
		self.expression.write_expression(&mut print_message);
		writeln!(&mut print_message, " {}", Paint::magenta(")")).unwrap();
		if !self.fragments.is_empty() {
			writeln!(&mut print_message, "with:").unwrap();
			for (name, expansion) in self.fragments {
				writeln!(
					&mut print_message,
					"  {} {} {}",
					Paint::magenta(name), Paint::blue("=").bold(),
					expansion
				).unwrap();
			}
		}
		self.expression.write_expansion(&mut print_message);
		writeln!(&mut print_message, ).unwrap();
		if let Some(msg) = self.custom_msg {
			writeln!(&mut print_message, "with message:").unwrap();
			writeln!(&mut print_message, "  {}", msg.bold()).unwrap();
		}
		writeln!(&mut print_message).unwrap();

		eprint!("{print_message}");
	}
}

#[rustfmt::skip]
impl<Left: Debug, Right: Debug> CheckExpression for BinaryOp<'_, Left, Right> {
	fn write_expression(&self, print_message: &mut  String) {
		write!(print_message, "{left} {op} {right}",
			left  = Paint::cyan(self.left_expr),
			op    = Paint::blue(self.operator).bold(),
			right = Paint::yellow(self.right_expr),
		).unwrap();
	}

	fn write_expansion(&self, print_message: &mut String) {
		let style = AssertOptions::get();

		if !style.expand.force_pretty() {
			let left = format!("{:?}", self.left);
			let right = format!("{:?}", self.right);
			if style.expand.force_compact() || ExpansionFormat::is_compact_good(&[&left, &right]) {
				writeln!(print_message, "with expansion:").unwrap();
				let diff = SingleLineDiff::new(&left, &right);
				print_message.push_str("  ");
				diff.write_left(print_message);
				write!(print_message, " {} ", Paint::blue(self.operator)).unwrap();
				diff.write_right(print_message);
				if left == right {
					if self.operator == "==" {
						write!(print_message, "\n{}", "Note: Left and right compared as unequal, but the Debug output of left and right is identical!".red()).unwrap();
					} else {
						write!(print_message, "\n{}", "Note: Debug output of left and right is identical.".bold()).unwrap();
					}
				}
				return
			}
		}

		// Compact expansion was disabled or not compact enough, so go full-on pretty debug format.
		let left = format!("{:#?}", self.left);
		let right = format!("{:#?}", self.right);
		writeln!(print_message, "with diff:").unwrap();
		MultiLineDiff::new(&left, &right)
			.write_interleaved(print_message);
	}
}

#[rustfmt::skip]
impl CheckExpression for BooleanExpr<'_> {
	fn write_expression(&self, print_message: &mut  String) {
		write!(print_message, "{}", Paint::cyan(self.expression)).unwrap();
	}

	fn write_expansion(&self, print_message: &mut String) {
		writeln!(print_message, "with expansion:").unwrap();
		write!(print_message, "  {:?}", false.cyan()).unwrap();
	}
}

#[rustfmt::skip]
impl<Value: Debug> CheckExpression for MatchExpr<'_, Value> {
	fn write_expression(&self, buffer: &mut String) {
		if self.print_let {
			write!(buffer, "{} ", Paint::blue("let").bold()).unwrap();
		}
		write!(buffer, "{pat} {eq} {expr}",
			pat  = Paint::cyan(self.pattern),
			eq   = Paint::blue("=").bold(),
			expr = Paint::yellow(self.expression),
		).unwrap();
	}

	fn write_expansion(&self, print_message: &mut String) {
		writeln!(print_message, "with expansion:").unwrap();
		let [value] = AssertOptions::get().expand.expand_all([&self.value]);
		let message = value.yellow().to_string();
		for line in message.lines() {
			writeln!(print_message, "  {line}").unwrap();
		}
		// Remove last newline.
		print_message.pop();
	}
}
