use fpas_parser::{Designator, DesignatorPart, Program};
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::{
    EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS, STD_UNIT_CONSOLE,
    canonical_std_unit_from_segments, is_std_root_segment, std_symbols as s,
};

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

    pub(super) fn add_local(&mut self, name: &str) -> u16 {
        let slot = self.next_slot;
        self.locals.push(Local {
            name: canonical_name(name),
            depth: self.scope_depth,
            slot,
        });
        self.next_slot += 1;
        slot
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

    pub(super) fn program_uses_std_console(program: &Program) -> bool {
        program.uses.iter().any(|u| {
            u.parts.len() == 2
                && is_std_root_segment(&u.parts[0])
                && canonical_std_unit_from_segments(&u.parts[0], &u.parts[1])
                    == Some(STD_UNIT_CONSOLE)
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

    pub(super) fn register_std_console_enums(&mut self) {
        self.register_enum_variants(s::STD_CONSOLE_KEY_KIND, KEY_KIND_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_EVENT_KIND, EVENT_KIND_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_MOUSE_ACTION, MOUSE_ACTION_VARIANTS);
        self.register_enum_variants(s::STD_CONSOLE_MOUSE_BUTTON, MOUSE_BUTTON_VARIANTS);
    }
}
