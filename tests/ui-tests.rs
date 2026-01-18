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
		.map_err(|e| error!("Failed to open directory \"cases\": {e}"))?;

	let mut failed = 0;
	for entry in cases {
		let entry = entry.map_err(|e| error!("Failed to read dir entry from \"cases\": {e}"))?;
		let file_name = entry.file_name();
		let file_type = entry.file_type()
			.map_err(|e| error!("Failed to stat \"cases/{}\": {e}", entry.file_name().to_string_lossy()))?;
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
	let expected_stdout = read_file(&stdout_path)?;
	let expected_stderr = read_file(&stderr_path)?;

	if let Some(expected_stdout) = &expected_stdout {
		if !compare_output(expected_stdout, &output.stdout) {
			fail("stdout does not match");
		}
	} else {
		write_file(&stdout_path, &output.stdout)?;
	}

	if let Some(expected_stderr) = &expected_stderr {
		if !compare_output(expected_stderr, &output.stderr) {
			fail("stderr does not match");
		}
	} else {
		write_file(&stderr_path, &output.stderr)?;
	}

	if output.status.code() == Some(0) {
		fail("program did not panic");
	} else if output.status.code().is_none() {
		fail(&format!("abnormal test exit: {}", output.status));
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
