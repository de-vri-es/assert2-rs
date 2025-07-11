use std::fmt::Write;
use yansi::Paint;

use super::{style, LEFT_STYLE, RIGHT_STYLE};

/// A line diff between two inputs.
pub struct MultiLineDiff<'a> {
	/// The actual diff results from the [`diff`] crate.
	line_diffs: Vec<LineDiff<'a>>,
}

impl<'a> MultiLineDiff<'a> {
	/// Create a new diff between a left and right input.
	pub fn new(left: &'a str, right: &'a str) -> Self {
		let line_diffs = LineDiff::from_diff(diff::lines(left, right));
		Self {
			line_diffs
		}
	}

	/// Write the left and right input interleaved with eachother, highlighting the differences between the two.
	pub fn write_interleaved(&self, buffer: &mut String) {
		for diff in &self.line_diffs {
			match *diff {
				LineDiff::LeftOnly(left) => {
					writeln!(buffer, "{}", style(&format_args!("< {left}"), LEFT_STYLE)).unwrap();
				},
				LineDiff::RightOnly(right) => {
					writeln!(buffer, "{}", style(&format_args!("> {right}"), RIGHT_STYLE)).unwrap();
				},
				LineDiff::Different(left, right) => {
					let diff = SingleLineDiff::new(left, right);
					write!(buffer, "{} ", "<".paint(diff.left_highlights.normal)).unwrap();
					diff.write_left(buffer);
					write!(buffer, "\n{} ", ">".paint(diff.right_highlights.normal)).unwrap();
					diff.write_right(buffer);
					buffer.push('\n');
				},
				LineDiff::Equal(text) => {
					writeln!(buffer, "  {}", text.primary().on_primary().dim()).unwrap();
				},
			}
		}
		// Remove last newline.
		buffer.pop();
	}
}

enum LineDiff<'a> {
	// There is only a left line.
	LeftOnly(&'a str),
	// There is only a right line.
	RightOnly(&'a str),
	// There is a left and a right line, but they are different.
	Different(&'a str, &'a str),
	// There is a left and a right line, and they are equal.
	Equal(&'a str),
}

impl<'a> LineDiff<'a> {
	fn from_diff(diffs: Vec<diff::Result<&'a str>>) -> Vec<Self> {
		let mut output = Vec::with_capacity(diffs.len());

		let mut seen_left = 0;
		for item in diffs {
			match item {
				diff::Result::Left(l) => {
					output.push(LineDiff::LeftOnly(l));
					seen_left += 1;
				},
				diff::Result::Right(r) => {
					if let Some(last) = output.last_mut() {
						match last {
							// If we see exactly one left line followed by a right line,
							// make it a `Self::Different` entry so we perform word diff later.
							Self::LeftOnly(old_l) if seen_left == 1 => {
								*last = Self::Different(old_l, r);
								seen_left = 0;
								continue;
							},
							// If we see another right line, turn the `Self::Different` back into individual lines.
							// This way, we dont do word diffs when one left line was replaced by multiple right lines.
							Self::Different(old_l, old_r) => {
								let old_r = *old_r;
								*last = Self::LeftOnly(old_l);
								output.push(Self::RightOnly(old_r));
								output.push(Self::RightOnly(r));
								seen_left = 0;
								continue;
							},
							// In other cases, just continue to the default behaviour of adding a `RightOnly` entry.
							Self::LeftOnly(_) => (),
							Self::RightOnly(_) => (),
							Self::Equal(_) => (),
						}
					}
					output.push(LineDiff::RightOnly(r));
					seen_left = 0;
				},
				diff::Result::Both(l, _r) => {
					output.push(Self::Equal(l));
					seen_left = 0;
				}
			}
		}

		output
	}
}

/// A character/word based diff between two single-line inputs.
pub struct SingleLineDiff<'a> {
	/// The left line.
	left: &'a str,

	/// The right line.
	right: &'a str,

	/// The highlighting for the left line.
	left_highlights: Highlighter,

	/// The highlighting for the right line.
	right_highlights: Highlighter,
}

impl<'a> SingleLineDiff<'a> {
	/// Create a new word diff between two input lines.
	pub fn new(left: &'a str, right: &'a str) -> Self {
		let left_words = Self::split_words(left);
		let right_words = Self::split_words(right);
		let diffs = diff::slice(&left_words, &right_words);

		let mut left_highlights = Highlighter::new(yansi::Color::Cyan);
		let mut right_highlights = Highlighter::new(yansi::Color::Yellow);
		for diff in &diffs {
			match diff {
				diff::Result::Left(left) => {
					left_highlights.push(left.len(), true);
				},
				diff::Result::Right(right) => {
					right_highlights.push(right.len(), true);
				},
				diff::Result::Both(left, right) => {
					left_highlights.push(left.len(), false);
					right_highlights.push(right.len(), false);
				}
			}
		}

		Self {
			left,
			right,
			left_highlights,
			right_highlights,
		}
	}

	/// Write the left line with highlighting.
	///
	/// This does not write a line break to the buffer.
	pub fn write_left(&self, buffer: &mut String) {
		self.left_highlights.write_highlighted(buffer, self.left);
	}

	/// Write the right line with highlighting.
	///
	/// This does not write a line break to the buffer.
	pub fn write_right(&self, buffer: &mut String) {
		self.right_highlights.write_highlighted(buffer, self.right);
	}

	/// Split an input line into individual words.
	fn split_words(mut input: &str) -> Vec<&str> {
		/// Check if there should be a word break between character `a` and `b`.
		fn is_break_point(a: char, b: char) -> bool {
			if a.is_alphabetic() {
				!b.is_alphabetic() || (a.is_lowercase() && !b.is_lowercase())
			} else if a.is_ascii_digit() {
				!b.is_ascii_digit()
			} else if a.is_whitespace() {
				!b.is_whitespace()
			} else {
				true
			}
		}

		let mut output = Vec::new();
		while !input.is_empty() {
			let split = input.chars()
				.zip(input.char_indices().skip(1))
				.find_map(|(a, (pos, b))| Some(pos).filter(|_| is_break_point(a, b)))
				.unwrap_or(input.len());
			let (head, tail) = input.split_at(split);
			output.push(head);
			input = tail;
		}
		output
	}
}

/// Highlighter that incrementaly builds a range of alternating styles.
struct Highlighter {
	/// The ranges of alternating highlighting.
	///
	/// If the boolean is true, the range should be printed with the `highlight` style.
	/// If the boolean is false, the range should be printed with the `normal` style.
	ranges: Vec<(bool, std::ops::Range<usize>)>,

	/// The total length of the highlighted ranges (in bytes, not characters or terminal cells).
	total_highlighted: usize,

	/// The style for non-highlighted words.
	normal: yansi::Style,

	/// The style for highlighted words.
	highlight: yansi::Style,
}

impl Highlighter {
	/// Create a new highlighter with the given color.
	fn new(color: yansi::Color) -> Self {
		let normal = yansi::Style::new().fg(color);
		let highlight = yansi::Style::new().fg(yansi::Color::Black).bg(color).bold();
		Self {
			ranges: Vec::new(),
			total_highlighted: 0,
			normal,
			highlight,
		}
	}

	/// Push a range to the end of the highlighter.
	fn push(&mut self, len: usize, highlight: bool) {
		if highlight {
			self.total_highlighted += len;
		}
		if let Some(last) = self.ranges.last_mut() {
			if last.0 == highlight {
				last.1.end += len;
			} else {
				let start = last.1.end;
				self.ranges.push((highlight, start..start + len));
			}
		} else {
			self.ranges.push((highlight, 0..len))
		}
	}

	/// Write the data using the highlight ranges.
	fn write_highlighted(&self, buffer: &mut String, data: &str) {
		let not_highlighted = data.len() - self.total_highlighted;
		if not_highlighted < div_ceil(self.total_highlighted, 2) {
			write!(buffer, "{}", data.paint(self.normal)).unwrap();
		} else {
			for (highlight, range) in self.ranges.iter().cloned() {
				let piece = if highlight {
					data[range].paint(self.highlight)
				} else {
					data[range].paint(self.normal)
				};
				write!(buffer, "{piece}").unwrap();
			}
		}
	}
}

fn div_ceil(a: usize, b: usize) -> usize {
	if b == 0 {
		a / b
	} else {
		let d = a / b;
		let r = a % b;
		if r > 0 {
			d + 1
		} else {
			d
		}
	}
}

#[test]
fn test_div_ceil() {
	use crate::assert;
	assert!(div_ceil(0, 2) == 0);
	assert!(div_ceil(1, 2) == 1);
	assert!(div_ceil(2, 2) == 1);
	assert!(div_ceil(3, 2) == 2);
	assert!(div_ceil(4, 2) == 2);

	assert!(div_ceil(20, 7) == 3);
	assert!(div_ceil(21, 7) == 3);
	assert!(div_ceil(22, 7) == 4);
	assert!(div_ceil(27, 7) == 4);
	assert!(div_ceil(28, 7) == 4);
	assert!(div_ceil(29, 7) == 5);
}
