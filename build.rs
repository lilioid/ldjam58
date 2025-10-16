use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let counter_path = manifest_dir.join("build_number.txt");

    // Only bump on release builds
    let is_release = env::var("PROFILE").as_deref() == Ok("release");

    let current: u32 = match fs::read_to_string(&counter_path) {
        Ok(s) => s.trim().parse::<u32>().unwrap_or(21),
        Err(_) => 21,
    };

    let mut effective = current;

    if is_release {
        let next = current.saturating_add(1);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&counter_path)
            .expect("Failed to open build_number.txt for writing");
        writeln!(file, "{}", next).expect("Failed to write build number");
        effective = current; // use pre-increment for this build’s label, next is persisted

        // Only watch the counter file if we actually use it to control re-runs
        println!("cargo:rerun-if-changed={}", counter_path.display());
    }

    // Always generate the label file in OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_path = out_dir.join("build_info.rs");
    let label = format!("SL-{:03}", effective);
    let contents = format!("pub const BUILD_LABEL: &str = \"{}\";\n", label);
    fs::write(&gen_path, contents).expect("Failed to write generated build_info.rs");
}