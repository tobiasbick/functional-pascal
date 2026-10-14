//! Source-visible distinct type names for debugger conversions.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use fpas_ir::{DistinctTypeName, TypeId};
use fpas_parser::Decl;
use fpas_sema::Ty;

use super::TypeTable;
use crate::CompileError;

impl TypeTable {
    /// Retains the names under which an explicitly imported distinct type is visible.
    pub(in crate::lowering) fn register_imported_distinct_names(
        &mut self,
        qualified: &str,
        underlying: TypeId,
    ) {
        self.imported_distinct_names
            .insert(qualified.to_ascii_lowercase(), underlying);
    }

    /// Lists distinct type names declared in or imported into the compiling source.
    ///
    /// Imported spellings already contain the source-local unit alias.
    pub(in crate::lowering) fn distinct_type_names(
        &mut self,
        metadata: &fpas_sema::AnalysisMetadata,
        declarations: &[Decl],
    ) -> Result<Vec<DistinctTypeName>, CompileError> {
        let mut names = Vec::new();
        for declaration in declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            let name = definition.name.to_ascii_lowercase();
            if let Some(Ty::Distinct(distinct)) = metadata.named_types.get(&name) {
                names.push(DistinctTypeName {
                    underlying: self.intern(&distinct.underlying, 1, 1)?,
                    name,
                });
            }
        }
        for (name, underlying) in &self.imported_distinct_names {
            names.push(DistinctTypeName {
                name: name.clone(),
                underlying: *underlying,
            });
        }
        names.sort_by(|left, right| left.name.cmp(&right.name));
        names.dedup_by(|left, right| left.name == right.name);
        Ok(names)
    }
}
