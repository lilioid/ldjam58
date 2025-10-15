use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

fn main() {
    // Location of the repository root
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let counter_path = manifest_dir.join("build_number.txt");

    // Read current number (default to 21 to match existing SL-021 in code)
    let current: u32 = match fs::read_to_string(&counter_path) {
        Ok(s) => s.trim().parse::<u32>().unwrap_or(21),
        Err(_) => 21,
    };

    let next = current.saturating_add(1);

    // Persist the increment for the next compile
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&counter_path)
        .expect("Failed to open build_number.txt for writing");
    writeln!(file, "{}", next).expect("Failed to write build number");

    // Tell Cargo to re-run this build script when the counter changes
    println!("cargo:rerun-if-changed={}", counter_path.display());

    // Generate a small Rust file with the label constant
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_path = out_dir.join("build_info.rs");
    let label = format!("SL-{:03}", current); // Use the pre-increment value for this build
    let contents = format!("pub const BUILD_LABEL: &str = \"{}\";\n", label);
    fs::write(&gen_path, contents).expect("Failed to write generated build_info.rs");
}
