use proc_macro2::Span;
use quote::ToTokens as _;

/// Get the operator of a binary expression with surrounding whitespace (if possible).
pub fn operator_with_whitespace(expr: &syn::ExprBinary) -> Option<OperatorWithSpacing> {
	let (_left_start, left_end) = stream_start_end_spans(expr.left.to_token_stream())?;
	let (op_start, op_end) = stream_start_end_spans(expr.op.to_token_stream())?;
	let (right_start, _right_end) = stream_start_end_spans(expr.right.to_token_stream())?;
	let left_spacing = whitespace_between(left_end, op_start)?;
	let right_spacing = whitespace_between(op_end, right_start)?;
	Some(OperatorWithSpacing {
		before: left_spacing,
		operator: expr.op,
		after: right_spacing,
	})
}

pub struct OperatorWithSpacing {
	pub before: Whitespace,
	pub operator: syn::BinOp,
	pub after: Whitespace,
}

impl OperatorWithSpacing {
	/// Make a new operator with one space on each side.
	pub fn new(operator: syn::BinOp) -> Self {
		Self {
			before: Whitespace::new().with_spaces(1),
			operator,
			after: Whitespace::new().with_spaces(1),
		}
	}

	/// Make a new logical AND operator (`&&`) with one space on each side.
	pub fn new_logical_and() -> Self {
		Self::new(syn::BinOp::And(syn::token::AndAnd(Span::call_site())))
	}

	/// Get the total amount of newlines in the whitespace (before and after the operator).
	pub fn total_newlines(&self) -> usize {
		self.before.lines + self.after.lines
	}

	/// Get the minimum indentation in the whitespace (before or after).
	///
	/// Returns `None` if neither `before` or `after` has indentation.
	pub fn min_indent(&self) -> Option<usize> {
		[self.before.indentation(), self.after.indentation()]
			.into_iter()
			.flatten()
			.min()
	}

	/// Adjust the indentation of the spacing around the operator.
	pub fn adjust_indent(&mut self, adjust: isize) {
		self.before.adjust_indent(adjust);
		self.after.adjust_indent(adjust);
	}
}

impl std::fmt::Display for OperatorWithSpacing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Self {
			before,
			operator,
			after,
		} = self;
		let operator = operator.into_token_stream();
		write!(f, "{before}{operator}{after}")
	}
}

/// Get the whitespace inside a group, between the delimiters and the contents.
#[cfg(feature = "span-locations")]
pub fn whitespace_inside(group: &proc_macro2::Group) -> Option<(Whitespace, Whitespace)> {
	#[cfg(not(feature = "span-locations"))]
	{
		_ = group;
		None
	}

	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for compatible compilers
	{
		let group_start = group.span().unwrap().start();
		let group_end = group.span().unwrap().end();

		let (content_start, content_end) = match stream_start_end_spans(group.stream()) {
			// Group is not empty, use the content start and end spans.
			Some((start_span, end_span)) => (start_span.unwrap().start(), end_span.unwrap().end()),

			// Group is empty, get the spacing between the group start and end spans.
			None => {
				if group_end.line() == group_start.line() {
					// Group span includes delimiter, so shrink spacing by 2 (one per delimiter).
					let spaces = group_end.column().checked_sub(group_start.column() + 2)?;
					return Some((Whitespace { lines: 0, spaces }, Whitespace::new()))
				} else {
					let lines = group_end.line().checked_sub(group_start.line())?;
					// Columns are 1 based and the group_end includes the delimiter, so shrink spaces by 2.
					let spaces = group_end.column().checked_sub(2)?;
					return Some((Whitespace { lines, spaces }, Whitespace::new()));
				};
			}
		};

		// The start spans must be in the same file and end spans must be in the same file.
		if group_start.file() != content_start.file() || group_end.file() != content_end.file() {
			return None;
		}

		let spacing_start = if group_start.line() == content_start.line() {
			// Shrink spacing by 1 to account for the end delimiter in group_end.
			let spaces = content_start.column().checked_sub(group_start.column() + 1)?;
			Whitespace { lines: 0, spaces }
		} else {
			let lines = content_start.line().checked_sub(group_start.line())?;
			// Columns are 1 based, so shrink spaces by 1.
			let spaces = content_start.column().checked_sub(1).expect("column not one based");
			Whitespace { lines, spaces }
		};

		let spacing_end = if content_end.line() == group_end.line() {
			// Group end includes the delimiter, so shrink spacing by 1.
			let spaces = group_end.column().checked_sub(content_end.column() + 1)?;
			Whitespace { lines: 0, spaces }
		} else {
			let lines = group_end.line().checked_sub(content_end.line())?;
			// Columns are 1 based and the group_end includes the delimiter, so shrink spaces by 2.
			let spaces = group_end.column().checked_sub(2)?;
			Whitespace { lines, spaces }
		};

		Some((spacing_start, spacing_end))
	}
}

/// Get the source code between two spans (if possible).
pub fn whitespace_between(a: Span, b: Span) -> Option<Whitespace> {
	#[cfg(not(feature = "span-locations"))]
	{
		let _ = (a, b);
		None
	}
	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for compatible compilers
	{
		let span_a = a.unwrap().end();
		let span_b = b.unwrap().start();

		// If spans have different files, we can't say anything sensible about whitespace between them.
		if span_a.file() != span_b.file() {
			return None;
		}

		// If they are on the same line, we only care about the difference in columns.
		if span_a.line() == span_b.line() {
			// Span a must come before span b in order to determine the whitespace between them.
			let spaces = span_b.column().checked_sub(span_a.column())?;
			return Some(Whitespace { lines: 0, spaces});
		}

		// Otherwise, we count the line difference and we only look at the column of span b.
		// Span A must still come before span B.
		let lines = span_b.line().checked_sub(span_a.line())?;

		// Column is 1 based, so subtract 1.
		let spaces = span_b.column().saturating_sub(1);
		Some(Whitespace { lines, spaces})
	}
}

fn stream_start_end_spans(stream: proc_macro2::TokenStream) -> Option<(Span, Span)> {
	let mut tokens = stream.into_iter();
	let start_span = tokens.next()?.span();
	let mut end_span = start_span;
	for token in tokens {
		end_span = token.span();
	}
	Some((start_span, end_span))
}

#[derive(Copy, Clone)]
pub struct Whitespace {
	pub lines: usize,
	pub spaces: usize,
}

impl Whitespace {
	pub fn new() -> Self {
		Self {
			lines: 0,
			spaces: 0,
		}
	}

	#[must_use]
	pub fn with_lines(self, lines: usize) -> Self {
		Self {
			lines,
			spaces: self.spaces,
		}
	}

	#[must_use]
	pub fn with_spaces(self, spaces: usize) -> Self {
		Self {
			lines: self.lines,
			spaces,
		}
	}

	/// Get the amount of indentation this whitespace represents.
	///
	/// Only whitespace after a newline counts as indentation,
	/// so if `self.lines == 0`, this function returns `None`.
	pub fn indentation(&self) -> Option<usize> {
		if self.lines == 0 {
			None
		} else {
			Some(self.spaces)
		}
	}

	/// Adjust the indentation of the whitespace.
	///
	/// Only whitespace after a newline counts as indentation,
	/// so if `self.lines == 0`, this function does nothing.
	pub fn adjust_indent(&mut self, adjust: isize) {
		if self.lines > 0 {
			self.spaces = (self.spaces as isize + adjust) as usize;
		}
	}
}

impl From<Whitespace> for String {
	fn from(value: Whitespace) -> Self {
		let mut output = String::with_capacity(value.lines + value.spaces);
		for _ in 0..value.lines {
			output.push('\n');
		}
		for _ in 0..value.spaces {
			output.push(' ');
		}
		output
	}
}

impl std::fmt::Display for Whitespace {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for _ in 0..self.lines {
			f.write_str("\n")?;
		}
		for _ in 0..self.spaces {
			f.write_str(" ")?;
		}
		Ok(())
	}
}
