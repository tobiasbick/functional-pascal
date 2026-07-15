use std::fs;
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_dir.join("../../lib");
    println!("cargo:rerun-if-changed={}", source_root.display());

    let out_dir = std::env::var_os("OUT_DIR")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo must set OUT_DIR"))?;
    let profile_dir = Path::new(&out_dir).ancestors().nth(3).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cargo OUT_DIR must be below the target profile directory",
        )
    })?;
    copy_tree(&source_root, &profile_dir.join("lib"))?;
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&destination_path)?;
            copy_tree(&source_path, &destination_path)?;
        } else {
            fs::create_dir_all(destination)?;
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}
