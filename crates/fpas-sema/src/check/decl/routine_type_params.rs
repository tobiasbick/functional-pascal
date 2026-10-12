//! Routine parameters retain identities distinct from enclosing generic scopes.
//! Documentation: docs/pascal/language/functions/generic-routines.md.

use crate::{
    check::Checker,
    types::{GenericParamDef, Ty},
};
use fpas_parser::TypeParam;

impl Checker {
    /// Qualifies routine parameters and installs them in the temporary type scope.
    pub(super) fn routine_type_params(
        &mut self,
        name: &str,
        parameters: &[TypeParam],
    ) -> Vec<GenericParamDef> {
        let mut path = self.routine_type_path.clone();
        path.push(name.to_owned());
        let owner = path.join(".");
        let resolved = Self::resolve_type_params(parameters)
            .into_iter()
            .map(|parameter| GenericParamDef {
                name: format!("{owner}.{}", parameter.name),
                ..parameter
            })
            .collect::<Vec<_>>();
        for (source, parameter) in parameters.iter().zip(&resolved) {
            if let Some(symbol) = self.scopes.lookup_mut(&source.name) {
                symbol.ty = Ty::GenericParam(parameter.name.clone(), parameter.constraint);
            }
        }
        resolved
    }
}
