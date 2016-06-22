use std::env;
use std::fs;
use std::os::windows;
use std::path::Path;

fn main() {
	let target = env::var("TARGET").unwrap();
	let target: Vec<&str> = target.split("-").collect();

	match target[target.len() - 2] {
		"windows" | "win" | "win32" | "win64" | "mingw" | "mingw32" | "mingw64" => {
			for entry in fs::read_dir("./").unwrap() {
				let entry = entry.unwrap();
				let meta = fs::metadata(entry.path()).unwrap();

				if meta.is_file() {
					let path = entry.path();

					match path.extension() {
						Some(value) => {
							if value.to_str().unwrap().to_lowercase().eq("dll") {
								let filename = path.file_name().unwrap();

								fs::copy(filename, Path::new("./target").join(env::var("PROFILE").unwrap()).join(filename)).unwrap();
							}
						}
						_ => {}
					}
				}
			}
			windows::fs::symlink_dir("./assets", Path::new("./target").join(env::var("PROFILE").unwrap()).join("assets"));
		},
		_ => {}
	}
}

