use std::env;
use std::fs;
use std::option::Option;
use std::path::{ Path, PathBuf };

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut target_profile_dir = manifest_dir.clone();
    {
        target_profile_dir.push("target");
        target_profile_dir.push(env::var("PROFILE").unwrap());
    }
    // 1. Copy assets to output folder.
    copy(Path::new("./assets"), &target_profile_dir.join("assets"), None);
    // 2. Copy binaries, per each target paltform.
    let target = env::var("TARGET").unwrap();

    if target.contains("pc-windows") {
        let mut lib_dir = manifest_dir.clone();

        lib_dir.push("lib");
        if target.contains("msvc") {
            lib_dir.push("msvc");
        } 
        if target.contains("mingw") {
            lib_dir.push("gnu-mingw");
        }
        if target.contains("x86_64") {
            lib_dir.push("x64");
        } else {
            lib_dir.push("x32");
        }
        println!("cargo:rustc-link-search={}", lib_dir.display());
        copy(lib_dir.as_path(), &target_profile_dir, Some("dll"));
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
        else if entry.path().is_dir() {
            copy(entry.path().as_path(), to.join(entry.path().file_name().unwrap()).as_path(), extension);
        }
    }
}
