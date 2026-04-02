use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, DesignatorPart};

use super::super::canonical_name;
use super::Compiler;

impl Compiler {
    pub(super) fn try_emit_enum_constant(
        &mut self,
        d: &Designator,
        location: SourceLocation,
    ) -> Result<bool, crate::error::CompileError> {
        if d.parts.len() < 2 {
            return Ok(false);
        }
        if !d
            .parts
            .iter()
            .all(|p| matches!(p, DesignatorPart::Ident(_, _)))
        {
            return Ok(false);
        }
        let names: Vec<&str> = d
            .parts
            .iter()
            .filter_map(|p| match p {
                DesignatorPart::Ident(n, _) => Some(n.as_str()),
                _ => None,
            })
            .collect();
        let Some((member, type_segments)) = names.split_last() else {
            return Ok(false);
        };
        let type_name = type_segments.join(".");
        let resolved_type = self.short_aliases.get(&canonical_name(&type_name)).cloned();
        let type_names_to_try: Vec<&str> = {
            let mut v = vec![type_name.as_str()];
            if let Some(ref r) = resolved_type {
                v.push(r.as_str());
            }
            v
        };

        for tn in type_names_to_try {
            if let Some(info) = self.enums.get(&canonical_name(tn))
                && let Some(variant) = info
                    .variants
                    .iter()
                    .find(|v| v.name.eq_ignore_ascii_case(member))
            {
                if info.has_data {
                    // Data enum: emit MakeEnum with zero fields for fieldless variants.
                    // Variants with fields are constructed via compile_call.
                    if !variant.field_names.is_empty() {
                        return Ok(false);
                    }
                    let type_idx = self.add_constant(Value::Str(tn.into()), location)?;
                    let variant_idx = self.add_constant(Value::Str((*member).into()), location)?;
                    self.emit(Op::MakeEnum(type_idx, variant_idx, 0), location);
                } else {
                    // Simple enum: emit integer backing value.
                    self.emit_constant(Value::Integer(variant.backing), location)?;
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Look up an enum variant with associated data by a (possibly qualified) name.
    /// Returns `(type_name, variant_info)` if found.
    pub(in super::super) fn find_enum_variant_with_data(
        &self,
        name: &str,
    ) -> Option<(String, super::super::EnumVariantInfo)> {
        // Try "TypeName.Variant" split.
        if let Some(dot) = name.rfind('.') {
            let type_part = &name[..dot];
            let variant_part = &name[dot + 1..];
            let type_names_to_try = [
                type_part.to_string(),
                self.short_aliases
                    .get(&canonical_name(type_part))
                    .cloned()
                    .unwrap_or_default(),
            ];
            for tn in &type_names_to_try {
                if tn.is_empty() {
                    continue;
                }
                if let Some(info) = self.enums.get(&canonical_name(tn))
                    && info.has_data
                    && let Some(v) = info
                        .variants
                        .iter()
                        .find(|v| v.name.eq_ignore_ascii_case(variant_part))
                {
                    return Some((tn.clone(), v.clone()));
                }
            }
        }
        None
    }
}
