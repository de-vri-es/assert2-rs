use std::{io::Write, path::{Path, PathBuf}};

use tempfile::TempDir;

macro_rules! error {
	($($args:tt)*) => {
		{ writeln!(std::io::stderr().lock(), "{}: {}", yansi::Paint::red("error").bright(), format_args!($($args)*)).ok(); }
	};
}

macro_rules! warn {
	($($args:tt)*) => {
		{ writeln!(std::io::stderr().lock(), "{}: {}", yansi::Paint::yellow("warning").bold().bright(), format_args!($($args)*)).ok(); }
	};
}

#[test]
fn ui_tests() -> std::process::ExitCode {
	if let Err(()) = do_main() {
		std::process::ExitCode::FAILURE
	} else {
		std::process::ExitCode::SUCCESS
	}
}

const CARGO_TARGET_TMPDIR: &str = env!("CARGO_TARGET_TMPDIR");
const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

const EXPECTED_STDOUT: &str = "stdout-expected.txt";
const ACTUAL_STDOUT: &str = "stdout-actual.txt";
const EXPECTED_STDERR: &str = "stderr-expected.txt";
const ACTUAL_STDERR: &str = "stderr-actual.txt";

fn do_main() -> Result<(), ()> {
	let cases = std::fs::read_dir(Path::new(CARGO_MANIFEST_DIR).join("tests/ui-tests"))
		.map_err(|e| error!("Failed to open directory \"tests/ui-tests\": {e}"))?;

	let bless = std::env::var_os("UI_TESTS").is_some_and(|x| x == "bless");

	let mut failed = 0;
	for entry in cases {
		let entry = entry.map_err(|e| error!("Failed to read dir entry from \"tests/ui-tests\": {e}"))?;
		let file_name = entry.file_name();
		let file_type = entry.file_type()
			.map_err(|e| error!("Failed to stat \"tests/ui-tests/{}\": {e}", entry.file_name().to_string_lossy()))?;
		if file_type.is_dir() {
			let name = file_name
				.into_string()
				.map_err(|file_name| error!("Directory entry contains invalid UTF-8: {file_name:?}"))?;

			let test = TestDefinition::new(name, entry.path())?;
			write!(std::io::stderr().lock(), "UI test {} ... ", yansi::Paint::bold(&test.name).bright()).ok();
			let result = test.run(bless)?;
			if result.diagnostics.has_error {
				writeln!(std::io::stderr().lock(), "{}", yansi::Paint::red("fail")).ok();
				failed += 1;
			} else {
				writeln!(std::io::stderr().lock(), "{}", yansi::Paint::green("ok")).ok();
			}
			result.diagnostics.print_all();
			test.cleanup().ok();
		}
	}

	if failed == 0 {
		Ok(())
	} else {
		Err(())
	}
}

struct TestDefinition {
	name: String,
	path: PathBuf,
	expected_stdout: Option<Vec<u8>>,
	expected_stderr: Option<Vec<u8>>,
	working_dir: tempfile::TempDir,
	target_dir: PathBuf,
}

struct TestResults {
	diagnostics: Diagnostics,
	output: Option<std::process::Output>,
}

pub struct Diagnostics {
	data: Vec<(Level, String)>,
	has_error: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum Level {
	Warning,
	Error,
}

impl Diagnostics {
	pub fn new() -> Self {
		Self {
			data: Vec::new(),
			has_error: false,
		}
	}

	pub fn add_warning(&mut self, warning: impl std::fmt::Display) {
		self.data.push((Level::Warning, warning.to_string()));
	}

	pub fn add_error(&mut self, error: impl std::fmt::Display) {
		self.data.push((Level::Error, error.to_string()));
		self.has_error = true;
	}

	pub fn print_all(&self) {
		let mut stderr = std::io::stderr().lock();
		for (level, message) in &self.data {
			let prefix = match level {
				Level::Warning => yansi::Paint::new("warning").yellow().bright().bold(),
				Level::Error => yansi::Paint::new("error").red().bright().bold(),
			};
			writeln!(&mut stderr, "{prefix}: {message}").ok();
		}
	}
}

impl TestDefinition {
	pub fn new(name: String, path: PathBuf) -> Result<Self, ()> {
		let expected_stdout = read_file(&path.join(EXPECTED_STDOUT))?;
		let expected_stderr = read_file(&path.join(EXPECTED_STDERR))?;

		let working_dir = tempfile::TempDir::new_in(CARGO_TARGET_TMPDIR)
			.map_err(|e| error!("Failed to create temporary directory for test {name:?}: {e}"))?;

		setup_test_crate(&name, &path, working_dir.path())?;

		Ok(Self {
			name,
			path,
			expected_stdout,
			expected_stderr,
			working_dir,
			target_dir: Path::new(CARGO_TARGET_TMPDIR).join("target"),
		})
	}

	pub fn expected_stdout_path(&self) -> PathBuf {
		self.path.join(EXPECTED_STDOUT)
	}

	pub fn actual_stdout_path(&self) -> PathBuf {
		self.path.join(ACTUAL_STDOUT)
	}

	pub fn expected_stderr_path(&self) -> PathBuf {
		self.path.join(EXPECTED_STDERR)
	}

	pub fn actual_stderr_path(&self) -> PathBuf {
		self.path.join(ACTUAL_STDERR)
	}

	pub fn run(&self, bless: bool) -> Result<TestResults, ()> {
		let mut results = TestResults {
			diagnostics: Diagnostics::new(),
			output: None,
		};

		let build_status = match self.build_crate() {
			Ok(status) => status,
			Err(e) => {
				results.diagnostics.add_error(format!("cargo build: failed to spawn process: {e}"));
				return Ok(results);
			},
		};

		if !build_status.success() {
			results.diagnostics.add_error(format!("cargo build: {build_status}"));
			return Ok(results);
		}

		let output = match self.run_test_binary() {
			Ok(output) => results.output.insert(output),
			Err(e) => {
				results.diagnostics.add_error(format!("cargo run: failed to spawn process: {e}"));
				return Ok(results);
			},
		};

		match output.status.code() {
			None => results.diagnostics.add_error(format!("test exit status: {}", output.status)),
			Some(0) => results.diagnostics.add_error("test did not panic"),
			Some(_) => (),
		}

		check_output(
			&mut results.diagnostics,
			"stdout",
			self.expected_stdout.as_deref(),
			&output.stdout,
			bless,
			&self.expected_stdout_path(),
			&self.actual_stdout_path(),
		)?;
		check_output(
			&mut results.diagnostics,
			"stderr",
			self.expected_stderr.as_deref(),
			&output.stderr,
			bless,
			&self.expected_stderr_path(),
			&self.actual_stderr_path(),
		)?;

		Ok(results)
	}

	fn build_crate(&self) -> Result<std::process::ExitStatus, std::io::Error> {
		std::process::Command::new("cargo")
			.current_dir(self.working_dir.path())
			.args(["build", "--quiet"])
			.arg("--target-dir")
			.arg(&self.target_dir)
			.status()
	}

	fn run_test_binary(&self) -> Result<std::process::Output, std::io::Error> {
		std::process::Command::new("cargo")
			.current_dir(self.working_dir.path())
			.args(["run", "--quiet"])
			.arg("--target-dir")
			.arg(&self.target_dir)
			.env("ASSERT2", "color")
			.output()
	}

	pub fn cleanup(self) -> Result<(), ()> {
		self.working_dir.close()
			.map_err(|e| error!("Failed to clean up temporary directory: {e}"))
	}
}

fn check_output(
	diagnostics: &mut Diagnostics,
	stream_name: &str,
	expected: Option<&[u8]>,
	actual: &[u8],
	bless: bool,
	expected_path: &Path,
	actual_path: &Path,
) -> Result<(), ()> {
	match (bless, expected) {
		(true, _) => {
			write_file(expected_path, actual)?;
			Ok(())
		}
		(false, None) => {
			diagnostics.add_warning(format!("no expected {stream_name} available, saving output of this run as expected output."));
			write_file(expected_path, actual)?;
			Ok(())
		},
		(false, Some(expected)) => {
			if actual != expected {
				diagnostics.add_error(format!("{stream_name} does not match, output saved to: {}", actual_path.display()));
				write_file(actual_path, actual)?;
			}
			Ok(())
		}
	}
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

fn write_file(path: &Path, data: &[u8]) -> Result<(), ()> {
	let mut file = std::fs::File::create(path)
		.map_err(|e| error!("Failed to open {} for writing: {e}", path.display()))?;
	std::io::Write::write_all(&mut file, data)
		.map_err(|e| error!("Failed to write to {}: {e}", path.display()))?;
	Ok(())
}

fn setup_test_crate(name: &str, path: &Path, working_dir: &Path) -> Result<(), ()> {
	let assert2_path = env!("CARGO_MANIFEST_DIR");

	std::fs::copy(path.join("main.rs"), working_dir.join("main.rs"))
		.map_err(|e| error!("Failed to copy {}/main.rs to {}: {e}", path.display(), working_dir.display()))?;
	let manifest = std::fs::OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(working_dir.join("Cargo.toml"))
		.map_err(|e| error!("Failed to create manifest at {}/Cargo.toml: {e}", working_dir.display()))?;
	write_manifest(manifest, name, assert2_path)
		.map_err(|e| error!("Failed to write manifest to {}/Cargo.toml: {e}", working_dir.display()))?;
	Ok(())
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
