use rustc_version::{version_meta, Channel};

fn main() {
	println!("cargo:rustc-check-cfg=cfg(nightly)");
	let version = version_meta().unwrap();

	if version.channel <= Channel::Nightly {
		println!("cargo:rustc-cfg=feature=\"nightly\"");
	}
	if (version.semver.major, version.semver.minor) >= (1, 88) {
		println!("cargo:rustc-cfg=feature=\"span-locations\"");
	}
}
