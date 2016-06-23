use std::env;
use std::fs;
use std::option::Option;
use std::path::Path;

fn main() {
	let target = env::var("TARGET").unwrap();
	let target: Vec<&str> = target.split("-").collect();

	match target[target.len() - 2] {
		"windows" | "win" | "win32" | "win64" | "mingw" | "mingw32" | "mingw64" => {
			let target_profile_dir = Path::new("./target").join(env::var("PROFILE").unwrap());

			copy(Path::new("./bin"), &target_profile_dir, Some("dll"));
			copy(Path::new("./assets"), &target_profile_dir.join("assets"), None);
		},
		_ => {}
	}
}

fn copy(from: &Path, to: &Path, extension: Option<&str>) {
	let ext = extension.unwrap_or("*").to_lowercase();

	for entry in fs::read_dir(from).unwrap() {
		let entry = entry.unwrap();

		if entry.path().is_file() {
			match entry.path().extension() {
				Some(value) => {
					if ext.eq("*") || ext.eq(&value.to_str().unwrap_or("").to_lowercase()) {
						let target = to.join(entry.path().file_name().unwrap());

						if !target.is_file() {
							fs::create_dir_all(to).unwrap();
							fs::copy(entry.path(), target).unwrap();							
						}
					}
				}
				_ => {}
			}
		}
	}
}

