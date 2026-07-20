use std::{env, path::PathBuf};

const REQUIRED_FRONTEND_FILES: [&str; 2] = ["index.html", "404.html"];

fn main() {
    if env::var_os("CARGO_FEATURE_EMBEDDED_FRONTEND").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join("../frontend/out");
    println!("cargo:rerun-if-changed={}", output_dir.display());
    for required_file in REQUIRED_FRONTEND_FILES {
        let path = output_dir.join(required_file);
        println!("cargo:rerun-if-changed={}", path.display());
        assert!(
            path.is_file(),
            "embedded frontend requires {}; run `pnpm --filter frontend build:embedded` before building with --features embedded-frontend",
            path.display()
        );
    }
}
