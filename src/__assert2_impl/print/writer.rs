use std::borrow::Cow;
use unicode_width::UnicodeWidthChar;

/// Writer that supports styling, wrapping and marking with `^^^`.
pub struct WrappingWriter<'a> {
	buffer: &'a mut String,
	width: usize,
	styling_enabled: bool,
	current_line_width: usize,
	current_line_undercurl: Vec<(std::ops::Range<usize>, yansi::Style)>,
	need_flush: bool,
	indent: usize,
}

impl<'a> WrappingWriter<'a> {
	pub fn new(buffer: &'a mut String, width: usize, styling_enabled: bool) -> Self {
		Self {
			buffer,
			width,
			styling_enabled,
			current_line_width: 0,
			current_line_undercurl: Vec::new(),
			need_flush: false,
			indent: 0,
		}
	}

	pub fn reserve(&mut self, additional: usize) {
		self.buffer.reserve(additional);
	}

	pub fn buffer_mut(&mut self) -> &mut String {
		self.buffer
	}

	pub fn write(&mut self, data: &str) {
		self.write_snippet(&Snippet::new(data));
	}

	pub fn write_styled(&mut self, data: &str, style: yansi::Style) {
		self.write_snippet(&Snippet::new(data).style(style));
	}

	pub fn set_indent(&mut self, indent: usize) {
		self.indent = indent;
	}

	pub fn write_snippet(&mut self, snippet: &Snippet<'_>) {
		let mut content = snippet.content.as_ref();
		let style = snippet.style;
		let undercurl = snippet.undercurl;

		while !content.is_empty() {
			let mut content_width = 0;
			let mut end_index = None;
			for (i, c) in content.char_indices() {
				if c == '\n' {
					end_index = Some(i);
					break;
				}
				let char_width = match c {
					'\t' => 4,
					c => c.width().unwrap_or(0),
				};
				if self.current_line_width + content_width + char_width > self.width {
					end_index = Some(i);
					break
				} else {
					content_width += char_width;
				}
			}

			let end_line = end_index.is_some();
			let end_index = end_index.unwrap_or(content.len());
			let (head, tail) = content.split_at(end_index);
			let tail = tail.strip_prefix('\n').unwrap_or(tail);
			self.write_piece(head, content_width, style, undercurl);
			content = tail;
			if end_line {
				self.flush_line();
			}
		}
	}

	fn write_piece(&mut self, content: &str, width: usize, style: yansi::Style, undercurl: Option<yansi::Style>) {
		// Skip all work if the content is empty.
		if content.is_empty() {
			return;
		}

		// Reserve space in the buffer.
		let mut reserve = content.len() + 1;
		if !self.need_flush {
			reserve += self.indent;
		}
		if self.styling_enabled {
			reserve += style.prefix().len() + style.suffix().len();
		}
		self.reserve(reserve);

		// Add indentation if needed.
		if !self.need_flush {
			for _ in 0..self.indent {
				self.buffer.push(' ');
			}
			self.current_line_width += self.indent;
		}

		// Add stryling prefix.
		if self.styling_enabled {
			self.buffer.push_str(&style.prefix());
		}

		// Add actual contents.
		self.buffer.push_str(content);

		// Add styling suffix.
		if self.styling_enabled {
			self.buffer.push_str(&style.suffix());
		}

		// Mark added content for undercurling.
		if let Some(undercurl) = undercurl {
			self.current_line_undercurl.push((self.current_line_width..self.current_line_width + width, undercurl));
		}

		// Update bookkeeping.
		self.need_flush = true;
		self.current_line_width += width;
	}

	pub fn flush_line(&mut self) {
		self.need_flush = false;
		self.buffer.push('\n');
		self.current_line_width = 0;

		// Write the undercurl, if any.
		let mut end_index = 0;
		for (range, style) in std::mem::take(&mut self.current_line_undercurl) {
			let skip = range.start - end_index;
			let count = range.len();
			end_index = range.end;
			for _ in 0..skip {
				self.buffer.push(' ');
			}
			if self.styling_enabled {
				self.buffer.push_str(&style.prefix());
			}
			for _ in 0..count {
				self.buffer.push('^');
			}
			if self.styling_enabled {
				self.buffer.push_str(&style.suffix());
			}
		}
		if !self.buffer.ends_with('\n') {
			self.buffer.push('\n');
		}
	}
}

impl Drop for WrappingWriter<'_> {
	fn drop(&mut self) {
		if self.need_flush {
			self.flush_line();
		}
	}
}

pub struct Snippet<'a> {
	content: Cow<'a, str>,
	style: yansi::Style,
	undercurl: Option<yansi::Style>,
}

impl<'a> Snippet<'a> {
	pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
		Self {
			content: content.into(),
			style: yansi::Style::new(),
			undercurl: None,
		}
	}

	pub fn style(mut self, style:  yansi::Style) -> Self {
		self.style = style;
		self
	}

	pub fn undercurl(mut self, style: yansi::Style) -> Self {
		self.undercurl = Some(style);
		self
	}

	pub fn undercurl_error(self) -> Self {
		self.undercurl(super::ERROR_STYLE)
	}
}

#[cfg(test)]
mod test {
	use super::{WrappingWriter, Snippet};

	#[test]
	fn styles_are_applied() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, true);
		writer.write_snippet(&Snippet::new("Hello").style(yansi::Style::new().yellow()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("dear").undercurl(yansi::Style::new().red().bold()));
		writer.write_snippet(&Snippet::new("!"));
		drop(writer);
		assert_eq!(buffer, concat!(
			"\u{1b}[33mHello\u{1b}[0m dear!\n",
			"      \u{1b}[1;31m^^^^\u{1b}[0m\n",
		));
	}

	#[test]
	fn styles_are_stripped() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, false);
		writer.write_snippet(&Snippet::new("Hello").style(yansi::Style::new().yellow()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("dear").undercurl(yansi::Style::new().red().bold()));
		writer.write_snippet(&Snippet::new("!"));
		drop(writer);
		assert_eq!(buffer, concat!(
			"Hello dear!\n",
			"      ^^^^\n",
		));
	}

	#[test]
	fn drop_flushes_unflushed() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, true);
		writer.write_snippet(&Snippet::new("Hello"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("dear").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("world").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new("!"));
		drop(writer);
		assert_eq!(buffer, concat!(
			"Hello dear world!\n",
			"      ^^^^ ^^^^^\n",
		));
	}

	#[test]
	fn drop_doesnt_flush_flushed_line() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, true);
		writer.write_snippet(&Snippet::new("Hello"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("dear").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("world").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new("!"));
		writer.flush_line();
		drop(writer);
		assert_eq!(buffer, concat!(
			"Hello dear world!\n",
			"      ^^^^ ^^^^^\n",
		));
	}

	#[test]
	fn exceeding_line_length_flushes_line() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, true);
		writer.write_snippet(&Snippet::new("four"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new("!"));
		drop(writer);
		assert_eq!(buffer, concat!(
			"four four four four \n",
			"     ^^^^      ^^^^\n",
			"four!\n",
			"^^^^\n",
		));
	}

	#[test]
	fn newline_flushes_line() {
		let mut buffer = String::new();
		let mut writer = WrappingWriter::new(&mut buffer, 20, true);
		writer.write_snippet(&Snippet::new("four"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four\n"));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new(" "));
		writer.write_snippet(&Snippet::new("four").undercurl(yansi::Style::new()));
		writer.write_snippet(&Snippet::new("!"));
		drop(writer);
		assert_eq!(buffer, concat!(
			"four four four\n",
			"     ^^^^\n",
			" four four!\n",
			" ^^^^ ^^^^\n",
		));
	}
}
