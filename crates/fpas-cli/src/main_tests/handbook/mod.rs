//! Executable handbook and authoring examples for the delivered syntax.
//!
//! Documentation: `docs/pascal/tools/fmt-style.md` and `docs/specs/grammar.ebnf`.

mod examples;
mod fences;
mod formatting;
mod keywords;
mod rejections;

use super::support::run_cli_args_and_capture_output;
use super::{create_temp_dir, write_text};
use std::{fs, path::Path};

fn document(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).expect("repository document must exist")
}

fn example(path: &str, index: usize) -> String {
    fences::pascal_sources(&document(path))
        .nth(index)
        .expect("documented Pascal example must exist")
        .replace("\r\n", "\n")
}

fn cli(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    run_cli_args_and_capture_output(
        &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
        cwd,
    )
}
