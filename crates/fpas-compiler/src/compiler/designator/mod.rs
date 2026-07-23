//! Designator lowering (read/write paths, builtins, enum constants).
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` (from the repository root).

mod builtin_consts;
mod enum_consts;
mod read;
mod write;

use fpas_parser::{Designator, DesignatorPart};

use super::{Compiler, LocalRef, canonical_name};

impl Compiler {
    /// Return the longest leading identifier chain that names a module global.
    fn module_global_prefix(&self, designator: &Designator) -> Option<(String, usize)> {
        let mut joined = String::new();
        let mut resolved = None;
        for (index, part) in designator.parts.iter().enumerate() {
            let DesignatorPart::Ident(name, _) = part else {
                break;
            };
            if !joined.is_empty() {
                joined.push('.');
            }
            joined.push_str(name);
            let canonical = canonical_name(&joined);
            if self.module_globals.contains(&canonical) {
                resolved = Some((
                    canonical_name(&self.qualify_owned_name(&canonical)),
                    index + 1,
                ));
            }
        }
        resolved
    }
}
