use std::fmt::Debug;

mod diff;
use self::diff::{MultiLineDiff, SingleLineDiff};

mod options;
use self::options::{AssertOptions, ExpansionFormat};

mod writer;

const DEFAULT_STYLE: yansi::Style = yansi::Style::new();
const ERROR_STYLE: yansi::Style = yansi::Style::new().red().bold();
const MACRO_STYLE: yansi::Style = yansi::Style::new().magenta();
const OP_STYLE: yansi::Style = yansi::Style::new().blue().bold();
const LEFT_STYLE: yansi::Style = yansi::Style::new().cyan();
const RIGHT_STYLE: yansi::Style = yansi::Style::new().yellow();
const NOTE_STYLE: yansi::Style = yansi::Style::new().bold();
const DIMMED_STYLE: yansi::Style = yansi::Style::new().dim();

pub struct FailedCheck<'a> {
	pub macro_name: &'a str,
	pub file: &'a str,
	pub line: u32,
	pub column: u32,
	pub custom_msg: Option<std::fmt::Arguments<'a>>,
	pub predicates: &'a [(&'a str, Predicate<'a>)],
	pub multiline: bool,
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
		left: &'a (dyn Debug + 'a),
		operator: &'a str,
		right: &'a (dyn Debug + 'a),
	},
	Let {
		expression: &'a (dyn Debug + 'a),
	},
	Bool,
}

fn terminal_width() -> usize {
	#[cfg(any(unix, windows))]
	{
		use terminal_size::{Width, terminal_size_of};
		if let Some((Width(width), _height)) = terminal_size_of(std::io::stderr()) {
			return width as usize;
		}
	}

	// TODO: Is this fallback a good idea?
	// Or is it better to disable features that misbehave when the terminal width is unknown?
	80
}

impl<'a> FailedCheck<'a> {
	#[rustfmt::skip]
	pub fn print(&self) {
		let mut buffer = String::new();
		let options = options::AssertOptions::get();
		let term_width = terminal_width();
		let mut writer = writer::WrappingWriter::new(&mut buffer, term_width, options.color);
		self.print_assertion(&mut writer);
		if !self.fragments.is_empty() {
			writer.write("with:\n");
			for (name, expansion) in self.fragments {
				writer.write("  ");
				writer.write_styled(name, MACRO_STYLE);
				writer.write(" ");
				writer.write_styled("=", OP_STYLE);
				writer.write(" ");
				writer.write_styled(expansion, MACRO_STYLE);
				writer.flush_line();
			}
		}
		self.expansion.write(&mut writer);
		writer.flush_line();
		if let Some(msg) = self.custom_msg {
			writer.write("with message:\n  ");
			writer.write_styled(&format!("{msg}"), NOTE_STYLE);
			writer.flush_line();
		}
		writer.flush_line();
		drop(writer);

		eprint!("{buffer}");
	}

	fn print_assertion(&self, writer: &mut writer::WrappingWriter) {
		writer.write_styled("Assertion failed", ERROR_STYLE);
		writer.write(" at ");
		writer.write_styled(self.file, NOTE_STYLE);
		writer.write(&format!(":{}:{}", self.line, self.column));
		writer.flush_line();
		writer.write("  ");
		writer.write_styled(self.macro_name, MACRO_STYLE);
		writer.write_styled("!( ", MACRO_STYLE);

		writer.set_indent(2);
		// Print all the predicates up to and including the failed one.
		for (i, (glue, predicate)) in self.predicates[..=self.failed].iter().enumerate() {
			writer.write_styled(glue, DIMMED_STYLE);
			predicate.write(writer, i == self.failed, self.predicates.len() > 1);
		}

		if let Some((glue, _next)) = self.predicates.get(self.failed + 1) {
			writer.write_styled(glue, DIMMED_STYLE);
			if glue.trim_end() == *glue {
				writer.write(" ");
			}
			writer.write_styled("...", DIMMED_STYLE);
		}
		if self.multiline {
			writer.flush_line();
		}

		if writer.buffer_mut().ends_with('\n') {
			writer.write_styled(")", MACRO_STYLE);
		} else {
			writer.write_styled(" )", MACRO_STYLE);
		}
		writer.set_indent(0);
		writer.flush_line();
	}
}

impl Predicate<'_> {
	fn write(&self, writer: &mut writer::WrappingWriter, failed: bool, undercurl: bool) {
		fn make_snippet(data: &str, style: yansi::Style, failed: bool, undercurl: bool) -> writer::Snippet<'_> {
			let mut snippet = writer::Snippet::new(data);
			if failed {
				snippet = snippet.style(style);
				if undercurl {
					snippet = snippet.undercurl_error();
				}
			} else {
				snippet = snippet.style(DIMMED_STYLE);
			}
			snippet
		}

		match self {
			Self::Binary { left, operator, right } => {
				writer.write_snippet(&make_snippet(left, LEFT_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(" ", DEFAULT_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(operator, OP_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(" ", DEFAULT_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(right, RIGHT_STYLE, failed, undercurl));
			},
			Self::Let { pattern, expression } => {
				writer.write_snippet(&make_snippet("let ", OP_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(pattern, LEFT_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(" = ", OP_STYLE, failed, undercurl));
				writer.write_snippet(&make_snippet(expression, RIGHT_STYLE, failed, undercurl));
			},
			Self::Bool { expression } => {
				writer.write_snippet(&make_snippet(expression, LEFT_STYLE, failed, undercurl));
			}
		}
	}
}

impl Expansion<'_> {
	fn write(&self, writer: &mut writer::WrappingWriter) {
		match self {
			Self::Binary { left, operator, right } => Self::write_binary(writer, left, operator, right),
			Self::Let { expression } => Self::write_let(writer, expression),
			Self::Bool => Self::write_bool(writer),
		}
	}

	fn write_binary(writer: &mut writer::WrappingWriter, left: &dyn Debug, operator: &str, right: &dyn Debug) {
		let style = AssertOptions::get();

		if !style.expand.force_pretty() {
			let left = format!("{left:?}");
			let right = format!("{right:?}");
			if style.expand.force_compact() || ExpansionFormat::is_compact_good(&[&left, &right]) {
				writer.write("with expansion:\n");
				let diff = SingleLineDiff::new(&left, &right);
				writer.write("  ");
				diff.write_left(writer);
				writer.write(" ");
				writer.write_styled(operator, OP_STYLE);
				writer.write(" ");
				diff.write_right(writer);
				if left == right {
					writer.flush_line();
					if operator == "==" {
						writer.write_styled("Note: Left and right compared as unequal, but the Debug output of left and right is identical!", ERROR_STYLE);
					} else {
						writer.write_styled("Note: Debug output of left and right is identical.", NOTE_STYLE);
					}
				}
				return
			}
		}

		// Compact expansion was disabled or not compact enough, so go full-on pretty debug format.
		let left = format!("{left:#?}");
		let right = format!("{right:#?}");
		writer.write("with diff:\n");
		MultiLineDiff::new(&left, &right)
			.write_interleaved(writer);
	}

	fn write_bool(writer: &mut writer::WrappingWriter) {
		writer.write("with expansion:\n");
		writer.write("  ");
		writer.write_styled("false", LEFT_STYLE);
	}

	fn write_let(writer: &mut writer::WrappingWriter, expression: &dyn Debug) {
		writer.write("with expansion:\n");
		let [value] = AssertOptions::get().expand.expand_all([expression]);
		for line in value.lines() {
			writer.write("  ");
			writer.write_styled(line, RIGHT_STYLE);
			writer.flush_line();
		}
		// Remove last newline.
		writer.buffer_mut().pop();
	}
}
