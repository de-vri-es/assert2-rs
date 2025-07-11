use std::fmt::Debug;
use std::fmt::Write;

mod diff;
use self::diff::{MultiLineDiff, SingleLineDiff};

mod options;
use self::options::{AssertOptions, ExpansionFormat};

const ERROR_STYLE: yansi::Style = yansi::Style::new().red().bold();
const MACRO_STYLE: yansi::Style = yansi::Style::new().magenta();
const OP_STYLE: yansi::Style = yansi::Style::new().blue().bold();
const LEFT_STYLE: yansi::Style = yansi::Style::new().cyan();
const RIGHT_STYLE: yansi::Style = yansi::Style::new().yellow();
const DIMMED_STYLE: yansi::Style = yansi::Style::new().bright_black();
const NOTE_STYLE: yansi::Style = yansi::Style::new().bold();

pub struct FailedCheck<'a> {
	pub macro_name: &'a str,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub predicates: &'a [Predicate<'a>],
	pub failed: usize,
	pub expansion: Expansion<'a>,
	pub fragments: &'a [(&'a str, &'a str)],
}

pub trait CheckExpression {
	fn write_expansion(&self, buffer: &mut String);
}

pub enum Predicate<'a> {
	Binary {
		left: &'a str,
		operator: &'a str,
		right: &'a str,
	},
	Let {
		pattern: &'a str,
		expression: &'a str,
	},
	Bool {
		expression: &'a str,
	},
}

pub enum Expansion<'a> {
	Binary {
		left: &'a dyn Debug,
		operator: &'a str,
		right: &'a dyn Debug,
	},
	Let {
		expression: &'a dyn Debug,
	},
	Bool,
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

impl<'a> FailedCheck<'a> {
	#[rustfmt::skip]
	pub fn print(&self) {
		let mut print_message = String::new();
		writeln!(&mut print_message, "{msg} at {file}:{line}:{column}:",
			msg    = style("Assertion failed", ERROR_STYLE),
			file   = style(self.file, NOTE_STYLE),
			line   = self.line,
			column = self.column,
		).unwrap();
		write!(&mut print_message, "  {name}{open} ",
			name = style(self.macro_name, MACRO_STYLE),
			open = style("!(", MACRO_STYLE),
		).unwrap();
		for (i, predicate) in self.predicates.iter().enumerate() {
			if i > 0 {
				write!(print_message, " && ").unwrap();
			}
			predicate.write(&mut print_message, i == self.failed);
		}
		writeln!(&mut print_message, " {}", style(")", MACRO_STYLE)).unwrap();
		if !self.fragments.is_empty() {
			writeln!(&mut print_message, "with:").unwrap();
			for (name, expansion) in self.fragments {
				writeln!(
					&mut print_message,
					"  {} {} {}",
					style(name, MACRO_STYLE),
					style("=", OP_STYLE),
					expansion
				).unwrap();
			}
		}
		self.expansion.write(&mut print_message);
		writeln!(&mut print_message, ).unwrap();
		if let Some(msg) = self.custom_msg {
			writeln!(&mut print_message, "with message:").unwrap();
			writeln!(&mut print_message, "  {}", style(msg, NOTE_STYLE)).unwrap();
		}
		writeln!(&mut print_message).unwrap();

		eprint!("{print_message}");
	}
}

impl Predicate<'_> {
	fn write(&self, print_message: &mut String, failed: bool) {
		let op_style = match failed {
			true => OP_STYLE,
			false => yansi::Style::new(),
		};
		let left_style = match failed {
			true => LEFT_STYLE,
			false => yansi::Style::new(),
		};
		let right_style = match failed {
			true => RIGHT_STYLE,
			false => yansi::Style::new(),
		};

		match self {
			Self::Binary { left, operator, right } => {
				write!(print_message, "{left} {op} {right}",
					left  = style(left, left_style),
					op    = style(operator, op_style),
					right = style(right, right_style),
				).unwrap();
			},
			Self::Let { pattern, expression } => {
				write!(print_message, "{let} {pat} {eq} {expr}",
					let  = style("let", op_style),
					pat  = style(pattern, left_style),
					eq   = style("=", op_style),
					expr = style(expression, right_style),
				).unwrap();
			},
			Self::Bool { expression } => {
				write!(print_message, "{expr}",
					expr = style(expression, right_style),
				).unwrap();
			},
		}
	}
}

impl Expansion<'_> {
	fn write(&self, print_message: &mut String) {
		match self {
			Self::Binary { left, operator, right } => Self::write_binary(print_message, left, operator, right),
			Self::Let { expression } => Self::write_let(print_message, expression),
			Self::Bool => Self::write_bool(print_message),
		}
	}

	fn write_binary(print_message: &mut String, left: &dyn Debug, operator: &str, right: &dyn Debug) {
		let style = AssertOptions::get();

		if !style.expand.force_pretty() {
			let left = format!("{left:?}");
			let right = format!("{right:?}");
			if style.expand.force_compact() || ExpansionFormat::is_compact_good(&[&left, &right]) {
				writeln!(print_message, "with expansion:").unwrap();
				let diff = SingleLineDiff::new(&left, &right);
				print_message.push_str("  ");
				diff.write_left(print_message);
				write!(print_message, " {} ", self::style(operator, OP_STYLE)).unwrap();
				diff.write_right(print_message);
				if left == right {
					if operator == "==" {
						write!(print_message, "\n{}", self::style("Note: Left and right compared as unequal, but the Debug output of left and right is identical!", ERROR_STYLE)).unwrap();
					} else {
						write!(print_message, "\n{}", self::style("Note: Debug output of left and right is identical.", NOTE_STYLE)).unwrap();
					}
				}
				return
			}
		}

		// Compact expansion was disabled or not compact enough, so go full-on pretty debug format.
		let left = format!("{left:#?}");
		let right = format!("{right:#?}");
		writeln!(print_message, "with diff:").unwrap();
		MultiLineDiff::new(&left, &right)
			.write_interleaved(print_message);
	}

	fn write_bool(print_message: &mut String) {
		writeln!(print_message, "with expansion:").unwrap();
		write!(print_message, "  {:?}", style(false, RIGHT_STYLE)).unwrap();
	}

	fn write_let(print_message: &mut String, expression: &dyn Debug) {
		writeln!(print_message, "with expansion:").unwrap();
		let [value] = AssertOptions::get().expand.expand_all([expression]);
		let message = style(value, RIGHT_STYLE).to_string();
		for line in message.lines() {
			writeln!(print_message, "  {line}").unwrap();
		}
		// Remove last newline.
		print_message.pop();
	}
}


fn style<T: std::fmt::Display>(value: T, style: yansi::Style) -> yansi::Painted<T> {
	yansi::Painted { value, style }
}
