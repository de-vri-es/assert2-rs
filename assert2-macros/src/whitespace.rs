use proc_macro2::Span;
use quote::ToTokens as _;
use syn::spanned::Spanned as _;

/// Get the operator of a binary expression with surrounding whitespace (if possible).
pub fn operator_with_whitespace(expr: &syn::ExprBinary) -> Option<String> {
	let left_spacing = whitespace_between(expr.left.span(), expr.op.span())?;
	let right_spacing = whitespace_between(expr.op.span(), expr.right.span())?;
	Some(format!("{}{}{}", left_spacing, expr.op.into_token_stream(), right_spacing))
}

/// Get the whitespace inside a group, between the delimiters and the contents.
#[cfg(feature = "span-locations")]
pub fn whitespace_inside(group: &proc_macro2::Group) -> Option<(String, String)> {
	#[cfg(not(feature = "span-locations"))]
	{
		_ = group;
		None
	}

	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for comptabile compilers
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
					return Some((Whitespace { lines: 0, spaces }.into(), String::new()))
				} else {
					let lines = group_end.line().checked_sub(group_start.line())?;
					// Columns are 1 based and the group_end includes the delimiter, so shrink spaces by 2.
					let spaces = group_end.column().checked_sub(2)?;
					return Some((Whitespace { lines, spaces }.into(), String::new()));
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
			Whitespace { lines: 0, spaces }.into()
		} else {
			let lines = content_start.line().checked_sub(group_start.line())?;
			// Columns are 1 based, so shrink spaces by 1.
			let spaces = content_start.column() - 1;
			Whitespace { lines, spaces }.into()
		};

		let spacing_end = if content_end.line() == group_end.line() {
			// Group end includes the delimiter, so shrink spacing by 1.
			let spaces = group_end.column().checked_sub(content_end.column() + 1)?;
			Whitespace { lines: 0, spaces }.into()
		} else {
			let lines = group_end.line().checked_sub(content_end.line())?;
			// Columns are 1 based and the group_end includes the delimiter, so shrink spaces by 2.
			let spaces = group_end.column().checked_sub(2)?;
			Whitespace { lines, spaces }.into()
		};

		Some((spacing_start, spacing_end))
	}
}

/// Get the source code between two spans (if possible).
pub fn whitespace_between(a: Span, b: Span) -> Option<String> {
	#[cfg(not(feature = "span-locations"))]
	{
		let _ = (a, b);
		None
	}
	#[cfg(feature = "span-locations")]
	#[allow(clippy::incompatible_msrv)] // code enabled only for comptabile compilers
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
			return Some(Whitespace { lines: 0, spaces}.into());
		}

		// Otherwise, we count the line difference and we only look at the column of span b.
		// Span A must still come before span B.
		let lines = span_b.line().checked_sub(span_a.line())?;

		// Column is 1 based, so subtract 1.
		let spaces = span_b.column().saturating_sub(1);
		Some(Whitespace { lines, spaces}.into())
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
struct Whitespace {
	lines: usize,
	spaces: usize,
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
