use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_dir.join("../../lib");
    println!("cargo:rerun-if-changed={}", source_root.display());

    let out_dir = std::env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR");
    let profile_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Cargo OUT_DIR must be below the target profile directory");
    copy_tree(&source_root, &profile_dir.join("lib"));
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in fs::read_dir(source).expect("standard library source directory must be readable") {
        let entry = entry.expect("standard library directory entry must be readable");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&destination_path)
                .expect("standard library output directory must be creatable");
            copy_tree(&source_path, &destination_path);
        } else {
            fs::create_dir_all(destination)
                .expect("standard library output directory must be creatable");
            fs::copy(&source_path, &destination_path)
                .expect("standard library source must be copied beside fpas");
        }
    }
}
