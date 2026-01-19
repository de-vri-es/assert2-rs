use std::{io::Write, path::Path};

macro_rules! error {
	($($args:tt)*) => {
		eprintln!("{}: {}", yansi::Paint::red("Error").bright(), format_args!($($args)*))
	};
}

macro_rules! warn {
	($($args:tt)*) => {
		writeln!(std::io::stderr().lock(), "{}: {}", yansi::Paint::yellow("Warning").bold().bright(), format_args!($($args)*)).ok()
	};
}

#[test]
fn ui_tests() {
	if let Err(()) = do_main() {
		std::process::exit(1);
	}
}

fn do_main() -> Result<(), ()> {
	let cases = std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui-tests"))
		.map_err(|e| error!("Failed to open directory \"tests/ui-tests\": {e}"))?;

	let mut failed = 0;
	for entry in cases {
		let entry = entry.map_err(|e| error!("Failed to read dir entry from \"tests/ui-tests\": {e}"))?;
		let file_name = entry.file_name();
		let file_type = entry.file_type()
			.map_err(|e| error!("Failed to stat \"tests/ui-tests/{}\": {e}", entry.file_name().to_string_lossy()))?;
		if file_type.is_dir() {
			let name = file_name
				.to_str()
				.ok_or_else(|| error!("Directory entry contains invalid UTF-8: {:?}", entry.file_name()))?;

			if !run_test(name, &entry.path())? {
				failed += 1;
			}
		}
	}

	if failed == 0 {
		Ok(())
	} else {
		Err(())
	}
}

fn run_test(name: &str, path: &Path) -> Result<bool, ()> {
	let assert2_path = env!("CARGO_MANIFEST_DIR");
	let cargo_target_tmpdir = env!("CARGO_TARGET_TMPDIR");
	let target_dir = Path::new(&cargo_target_tmpdir).join("target");

	let working_dir = tempfile::TempDir::new_in(cargo_target_tmpdir)
		.map_err(|e| error!("Failed to create temporary directory for test {name:?}: {e}"))?;

	std::fs::copy(path.join("main.rs"), working_dir.path().join("main.rs"))
		.map_err(|e| error!("Failed to copy cases/{name}/main.rs to {}: {e}", working_dir.path().display()))?;
	let manifest = std::fs::OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(working_dir.path().join("Cargo.toml"))
		.map_err(|e| error!("Failed to create manifest for test {name:?} at {}/Cargo.toml: {e}", working_dir.path().display()))?;
	write_manifest(manifest, name, assert2_path)
		.map_err(|e| error!("Failed to write manifest for test {name:?} to {}/Cargo.toml: {e}", working_dir.path().display()))?;

	write!(std::io::stderr().lock(), "UI test {} ... ", yansi::Paint::bold(name).bright()).ok();
	let mut failed = false;
	let mut fail = |reason: &str| {
		if !failed {
			writeln!(std::io::stderr().lock(), "{}: {reason}", yansi::Paint::red("fail")).ok();
			failed = true;
		}
	};

	let build_status = std::process::Command::new("cargo")
		.current_dir(working_dir.path())
		.args(["build", "--quiet"])
		.arg("--target-dir")
		.arg(&target_dir)
		.status()
		.map_err(|e| {
			fail("build failed");
			error!("cargo build: failed to spawn process: {e}");
		})?;
	if !build_status.success() {
		fail("build failed");
		error!("cargo build: {build_status}");
		return Err(());
	}

	let mut output = std::process::Command::new("cargo")
		.current_dir(working_dir.path())
		.args(["run", "--quiet"])
		.arg("--target-dir")
		.arg(&target_dir)
		.env("ASSERT2", "color")
		.output()
		.map_err(|e| {
			fail("spawn failed");
			error!("cargo run: failed to spawn process: {e}");
		})?;
	adjust_output(&mut output.stdout);
	adjust_output(&mut output.stderr);

	let stdout_path = path.join("expected.stdout");
	let stderr_path = path.join("expected.stderr");
	let exit_code_path = path.join("expected-exit-code");
	let expected_stdout = read_file(&stdout_path)?;
	let expected_stderr = read_file(&stderr_path)?;
	let expected_exit_code = read_exit_code(&exit_code_path)?;

	if let Some(expected_stdout) = &expected_stdout {
		if !compare_output(expected_stdout, &output.stdout) {
			fail("stdout does not match");
			print_output_diff("stdout", expected_stdout, &output.stdout);
		}
	} else {
		write_file(&stdout_path, &output.stdout)?;
	}

	if let Some(expected_stderr) = &expected_stderr {
		if !compare_output(expected_stderr, &output.stderr) {
			fail("stderr does not match");
			print_output_diff("stderr", expected_stderr, &output.stderr);
		}
	} else {
		write_file(&stderr_path, &output.stderr)?;
	}

	// Validate exit code
	match (output.status.code(), expected_exit_code) {
		// If expected exit code is specified, check against it
		(Some(actual), Some(expected)) => {
			if actual != expected {
				fail(&format!("exit code mismatch: expected {expected}, got {actual}"));
			}
		},
		// If no expected exit code is specified, default to expecting non-zero (panic)
		(Some(0), None) => {
			fail("program did not panic");
		},
		// Handle abnormal termination (signal, etc.)
		(None, _) => {
			fail(&format!("abnormal test exit: {}", output.status));
		},
		// Non-zero exit without expected exit code is OK (default panic behavior)
		(Some(_), None) => {},
	}

	if !failed {
		writeln!(std::io::stderr().lock(), "{}", yansi::Paint::green("ok")).ok();
	}

	match (expected_stdout.is_some(), expected_stderr.is_some()) {
		(true, true) => (),
		(true, false) => {
			warn!("No expected stderr found, stored output of this run as expected output");
		},
		(false, true) => {
			warn!("No expected stdout found, stored output of this run as expected output");
		},
		(false, false) => {
			warn!("No expected stdout or stderr found, stored output of this run as expected output");
		}
	}

	working_dir.close()
		.map_err(|e| error!("Failed to clean up temporary directory: {e}"))?;
	Ok(!failed)
}

fn read_file(path: &Path) -> Result<Option<Vec<u8>>, ()> {
	let mut file = match std::fs::File::open(path) {
		Ok(x) => x,
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(e) => {
			error!("Failed to open {} for reading: {e}", path.display());
			return Err(());
		},
	};
	let mut buffer = Vec::new();
	std::io::Read::read_to_end(&mut file, &mut buffer)
		.map_err(|e| error!("Failed to read from {}: {e}", path.display()))?;
	Ok(Some(buffer))
}

fn read_exit_code(path: &Path) -> Result<Option<i32>, ()> {
	match read_file(path)? {
		None => Ok(None),
		Some(data) => {
			let content = std::str::from_utf8(&data)
				.map_err(|e| error!("Failed to parse {} as UTF-8: {e}", path.display()))?;
			let code = content.trim().parse::<i32>()
				.map_err(|e| error!("Failed to parse exit code from {}: {e}", path.display()))?;
			Ok(Some(code))
		}
	}
}

fn write_file(path: &Path, data: &[u8]) -> Result<(), ()> {
	let mut file = std::fs::File::create(path)
		.map_err(|e| error!("Failed to open {} for writing: {e}", path.display()))?;
	std::io::Write::write_all(&mut file, data)
		.map_err(|e| error!("Failed to write to {}: {e}", path.display()))?;
	Ok(())
}

fn compare_output(expected: &[u8], actual: &[u8]) -> bool {
	expected == actual
}

fn escape_non_printable(s: &str) -> String {
	let mut result = String::with_capacity(s.len());
	for ch in s.chars() {
		match ch {
			'\\' => result.push_str("\\\\"),
			'\n' => result.push_str("\\n"),
			'\r' => result.push_str("\\r"),
			'\t' => result.push_str("\\t"),
			'\0' => result.push_str("\\0"),
			c if c.is_control() => {
				// Escape control characters as hex bytes
				let mut buf = [0u8; 4];
				let bytes = c.encode_utf8(&mut buf).as_bytes();
				for &byte in bytes {
					use std::fmt::Write;
					write!(&mut result, "\\x{:02x}", byte).unwrap();
				}
			}
			c => result.push(c),
		}
	}
	result
}

fn print_output_diff(stream_name: &str, expected: &[u8], actual: &[u8]) {
	use std::io::Write;
	let mut stderr = std::io::stderr().lock();
	
	writeln!(&mut stderr, "\n{}: Expected {} differs from actual {}", 
		yansi::Paint::yellow("Details").bold().bright(),
		stream_name,
		stream_name
	).ok();
	
	// Try to convert to strings for better readability
	match (std::str::from_utf8(expected), std::str::from_utf8(actual)) {
		(Ok(expected_str), Ok(actual_str)) => {
			writeln!(&mut stderr, "\n{}:", yansi::Paint::cyan("Expected").bold()).ok();
			for line in expected_str.lines() {
				writeln!(&mut stderr, "  {}", escape_non_printable(line)).ok();
			}
			if expected_str.is_empty() {
				writeln!(&mut stderr, "  {}", yansi::Paint::dim("(empty)")).ok();
			}
			
			writeln!(&mut stderr, "\n{}:", yansi::Paint::cyan("Actual").bold()).ok();
			for line in actual_str.lines() {
				writeln!(&mut stderr, "  {}", escape_non_printable(line)).ok();
			}
			if actual_str.is_empty() {
				writeln!(&mut stderr, "  {}", yansi::Paint::dim("(empty)")).ok();
			}
			
			// Show line-by-line diff
			let expected_lines: Vec<_> = expected_str.lines().collect();
			let actual_lines: Vec<_> = actual_str.lines().collect();
			
			if expected_lines.len() != actual_lines.len() {
				writeln!(&mut stderr, "\n{}: Expected {} lines, got {} lines",
					yansi::Paint::yellow("Line count").bold(),
					expected_lines.len(),
					actual_lines.len()
				).ok();
			}
			
			// Show first few differences
			let mut diff_count = 0;
			let max_diffs = 5;
			let min_len = expected_lines.len().min(actual_lines.len());
			let mut total_different_lines = 0;
			
			// Compare lines that exist in both
			for i in 0..min_len {
				if expected_lines[i] != actual_lines[i] {
					total_different_lines += 1;
					if diff_count < max_diffs {
						writeln!(&mut stderr, "\n{} {}:",
							yansi::Paint::yellow("Difference at line").bold(),
							i + 1
						).ok();
						let escaped_expected = escape_non_printable(expected_lines[i]);
						let escaped_actual = escape_non_printable(actual_lines[i]);
						writeln!(&mut stderr, "  - {}", yansi::Paint::red(&escaped_expected)).ok();
						writeln!(&mut stderr, "  + {}", yansi::Paint::green(&escaped_actual)).ok();
						diff_count += 1;
					}
				}
			}
			
			// Count and show extra lines from expected output
			let extra_expected = expected_lines.len().saturating_sub(min_len);
			total_different_lines += extra_expected;
			if extra_expected > 0 && diff_count < max_diffs {
				let end_index = expected_lines.len().min(min_len + max_diffs - diff_count);
				for i in min_len..end_index {
					writeln!(&mut stderr, "\n{} {} (only in expected):",
						yansi::Paint::yellow("Line").bold(),
						i + 1
					).ok();
					let escaped = escape_non_printable(expected_lines[i]);
					writeln!(&mut stderr, "  - {}", yansi::Paint::red(&escaped)).ok();
					diff_count += 1;
				}
			}
			
			// Count and show extra lines from actual output
			let extra_actual = actual_lines.len().saturating_sub(min_len);
			total_different_lines += extra_actual;
			if extra_actual > 0 && diff_count < max_diffs {
				let end_index = actual_lines.len().min(min_len + max_diffs - diff_count);
				for i in min_len..end_index {
					writeln!(&mut stderr, "\n{} {} (only in actual):",
						yansi::Paint::yellow("Line").bold(),
						i + 1
					).ok();
					let escaped = escape_non_printable(actual_lines[i]);
					writeln!(&mut stderr, "  + {}", yansi::Paint::green(&escaped)).ok();
					diff_count += 1;
				}
			}
			
			// Show message if there are more differences we didn't show
			if total_different_lines > diff_count {
				writeln!(&mut stderr, "\n{}", yansi::Paint::dim("(additional differences omitted)")).ok();
			}
		}
		_ => {
			// Binary data or invalid UTF-8
			writeln!(&mut stderr, "\n{}:", yansi::Paint::cyan("Expected (bytes)").bold()).ok();
			writeln!(&mut stderr, "  {} bytes: {:?}", expected.len(), 
				if expected.len() <= 50 { 
					format!("{:?}", expected) 
				} else { 
					format!("{:?}...", &expected[..50]) 
				}
			).ok();
			
			writeln!(&mut stderr, "\n{}:", yansi::Paint::cyan("Actual (bytes)").bold()).ok();
			writeln!(&mut stderr, "  {} bytes: {:?}", actual.len(),
				if actual.len() <= 50 { 
					format!("{:?}", actual) 
				} else { 
					format!("{:?}...", &actual[..50]) 
				}
			).ok();
		}
	}
	writeln!(&mut stderr).ok();
}

fn write_manifest<W: std::io::Write>(mut write: W, name: &str, assert2_path: &str) -> std::io::Result<()> {
	writeln!(&mut write, "[package]")?;
	writeln!(&mut write, "name = {name:?}")?;
	writeln!(&mut write, "version = \"0.0.0\"")?;
	writeln!(&mut write, "edition = \"2021\"")?;
	writeln!(&mut write, "publish = false")?;
	writeln!(&mut write, "[[bin]]")?;
	writeln!(&mut write, "name = {name:?}")?;
	writeln!(&mut write, "path = \"main.rs\"")?;
	writeln!(&mut write, "[dependencies]")?;
	writeln!(&mut write, "assert2 = {{ path = {assert2_path:?} }}")?;
	writeln!(&mut write, "reproducible-panic = \"0.1.2\"")?;
	writeln!(&mut write, "[workspace]")?;

	Ok(())
}

fn adjust_output(output: &mut Vec<u8>) {
	let _ = &output;
	#[cfg(windows)]
	output.retain(|&b| b != b'\r');
}
