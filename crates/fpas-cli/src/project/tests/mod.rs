use super::{ProjectKind, build_program, load_project};
use crate::test_support::{create_temp_dir, write_text};
use fpas_parser::{Decl, DesignatorPart};
use std::fs;

mod bindings;
mod imports;
mod loading;
mod support;
