//! Resolves value roots before walking record fields and collection indexes.
//!
//! Import access follows `docs/pascal/program-structure/units.md`.

use fpas_ir::{IrType, TypeId};
use fpas_parser::{Designator, DesignatorPart};

use super::LoweringContext;

impl LoweringContext {
    /// Finds the lexical binding or qualified global prefix of a value path.
    pub(in crate::lowering) fn designator_root(
        &self,
        designator: &Designator,
    ) -> Option<(String, usize)> {
        let DesignatorPart::Ident(first, _) = designator.parts.first()? else {
            return None;
        };
        if self.has_binding(first) || self.has_global(first) {
            return Some((first.clone(), 1));
        }
        let mut name = first.clone();
        for (index, part) in designator.parts.iter().enumerate().skip(1) {
            let DesignatorPart::Ident(member, _) = part else {
                break;
            };
            name.push('.');
            name.push_str(member);
            if self.has_global(&name) {
                return Some((name, index + 1));
            }
        }
        None
    }

    /// Returns the type after traversing a local or imported value path.
    pub(in crate::lowering) fn designator_type(&self, designator: &Designator) -> Option<TypeId> {
        let (name, consumed) = self.designator_root(designator)?;
        let mut ty = self.root_type(&name)?;
        for part in &designator.parts[consumed..] {
            ty = match (part, self.type_kind(ty)?) {
                (DesignatorPart::Ident(name, _), IrType::Record(layout)) => {
                    self.record_field(layout, name)?.1
                }
                (DesignatorPart::Index(_, _), IrType::Array(element)) => element,
                (DesignatorPart::Index(_, _), IrType::Dictionary { value, .. }) => value,
                (DesignatorPart::Index(_, _), IrType::String) => super::types::STRING,
                _ => return None,
            };
        }
        Some(ty)
    }
}
