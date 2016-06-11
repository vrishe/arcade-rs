use std::env;
use std::fs;
use std::path::Path;

fn main() {
	let target = env::var("TARGET").unwrap();
	let target: Vec<&str> = target.split("-").collect();

	match target[target.len() - 2] {
		"windows" | "win" | "win32" | "win64" | "mingw" | "mingw32" | "mingw64" => {
			let profile = env::var("PROFILE").unwrap();

			fs::copy("SDL2.dll", Path::new("./target").join(profile).join("SDL2.dll")).unwrap();
		},
		_ => {}
	}
}

