//! Local bindings, scopes, and name resolution for codegen.
//!
//! **Documentation:** `docs/pascal/language/functions/README.md` (from the repository root).

use fpas_parser::{Designator, DesignatorPart, Program};
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::{
    EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS, TUI_EVENT_KIND_VARIANTS,
    TUI_EXIT_REASON_VARIANTS, TUI_VIEW_KIND_VARIANTS, canonical_std_unit_from_segments,
    is_std_root_segment, std_symbols as s,
};

use crate::error::{CompileError, compile_error};
use fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW;

use super::{Compiler, EnumInfo, EnumVariantInfo, Local, LocalRef, canonical_name};

impl Compiler {
    pub(super) fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub(super) fn end_scope(&mut self, location: impl Copy + super::emit::IntoEmitLocation) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }
            self.locals.pop();
            self.next_slot -= 1;
            self.emit(fpas_bytecode::Op::Pop, location);
        }
    }

    pub(super) fn add_local(
        &mut self,
        name: &str,
        location: impl Copy + super::emit::IntoEmitLocation,
    ) -> Result<u16, CompileError> {
        let location = location.into_emit_location();
        let slot = self.next_slot;
        self.next_slot = slot.checked_add(1).ok_or_else(|| {
            compile_error(
                COMPILE_BYTECODE_OPERAND_OVERFLOW,
                format!(
                    "Too many locals in this routine (maximum is {}).",
                    u16::MAX
                ),
                format!(
                    "Reduce the number of local variables, parameters, and temporaries in this routine to at most {}.",
                    u16::MAX
                ),
                Compiler::call_site_span(location),
            )
        })?;
        self.locals.push(Local {
            name: canonical_name(name),
            depth: self.scope_depth,
            slot,
        });
        Ok(slot)
    }

    pub(super) fn resolve_local(&self, name: &str) -> Option<LocalRef> {
        let canonical = canonical_name(name);
        for local in self.locals.iter().rev() {
            if local.name == canonical {
                return Some(LocalRef::Local(local.slot));
            }
        }

        for (depth_minus_1, parent) in self.enclosing_locals.iter().rev().enumerate() {
            for local in parent.iter().rev() {
                if local.name == canonical {
                    return Some(LocalRef::Enclosing((depth_minus_1 + 1) as u16, local.slot));
                }
            }
        }

        None
    }

    pub(super) fn resolve_designator_name(d: &Designator) -> String {
        let mut result = String::new();
        for part in &d.parts {
            if let DesignatorPart::Ident(name, _) = part {
                if !result.is_empty() {
                    result.push('.');
                }
                result.push_str(name);
            }
        }
        result
    }

    pub(super) fn program_uses_std_unit(program: &Program, unit: &str) -> bool {
        program.uses.iter().any(|u| {
            u.parts.len() == 2
                && is_std_root_segment(&u.parts[0])
                && canonical_std_unit_from_segments(&u.parts[0], &u.parts[1]) == Some(unit)
        })
    }

    fn register_enum_variants(&mut self, type_name: &str, variant_names: &[&str]) {
        let variants: Vec<EnumVariantInfo> = variant_names
            .iter()
            .enumerate()
            .map(|(i, name)| EnumVariantInfo {
                name: (*name).to_string(),
                backing: i as i64,
                field_names: vec![],
            })
            .collect();
        self.enums.insert(
            canonical_name(type_name),
            EnumInfo {
                variants,
                has_data: false,
            },
        );
    }

    fn register_data_enum_variants(&mut self, type_name: &str, variants: &[(&str, &[&str])]) {
        let variants: Vec<EnumVariantInfo> = variants
            .iter()
            .enumerate()
            .map(|(i, (name, fields))| EnumVariantInfo {
                name: (*name).to_string(),
                backing: i as i64,
                field_names: fields.iter().map(|field| (*field).to_string()).collect(),
            })
            .collect();
        self.enums.insert(
            canonical_name(type_name),
            EnumInfo {
                variants,
                has_data: true,
            },
        );
    }

    pub(super) fn register_std_console_enums(&mut self) {
        self.register_enum_variants(s::STD_CONSOLE_KEY_KIND, KEY_KIND_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_EVENT_KIND, EVENT_KIND_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_MOUSE_ACTION, MOUSE_ACTION_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_MOUSE_BUTTON, MOUSE_BUTTON_VARIANTS);
    }

    pub(super) fn register_std_tui_enums(&mut self) {
        // `Std.Tui.TuiEvent.key` is `Std.Console.KeyEvent` (field `kind` is `Std.Console.KeyKind`).
        self.register_enum_variants(s::STD_CONSOLE_KEY_KIND, KEY_KIND_VARIANTS);
        self.register_enum_variants(s::STD_TUI_EVENT_KIND, TUI_EVENT_KIND_VARIANTS);
        self.register_enum_variants(s::STD_TUI_EXIT_REASON, TUI_EXIT_REASON_VARIANTS);
        self.register_enum_variants(s::STD_TUI_VIEW_KIND, TUI_VIEW_KIND_VARIANTS);
    }

    pub(super) fn register_std_json_enum(&mut self) {
        self.register_data_enum_variants(
            s::STD_JSON_VALUE,
            &[
                ("Null", &[]),
                ("Bool", &["Value"]),
                ("Number", &["Value"]),
                ("String", &["Value"]),
                ("Array", &["Items"]),
                ("Object", &["Fields"]),
            ],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_bytecode::SourceLocation;
    use fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW;
    use fpas_sema::{ExprTypeMap, MethodCallMap, RecordDefaultsMap, ScalarCaseBindingMap};

    fn empty_compiler() -> Compiler {
        Compiler::new(
            ExprTypeMap::default(),
            MethodCallMap::default(),
            RecordDefaultsMap::default(),
            ScalarCaseBindingMap::default(),
        )
    }

    #[test]
    fn add_local_overflow_reports_bytecode_operand_error() {
        let mut compiler = empty_compiler();
        let location = SourceLocation::new(1, 1);
        compiler.begin_scope();
        for index in 0..u16::MAX {
            compiler
                .add_local(&format!("v{index}"), location)
                .expect("locals within limit must succeed");
        }
        let err = compiler
            .add_local("overflow", location)
            .expect_err("local slot overflow must fail");
        assert_eq!(err.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }
}
