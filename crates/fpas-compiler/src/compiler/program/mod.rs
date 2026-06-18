//! Program- and declaration-level lowering.
//!
//! **Documentation:** `docs/pascal/04-functions.md`, `docs/pascal/program-structure/units.md` (from the repository root).

mod callables;

use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Decl, Program, TypeBody};

use fpas_std::{STD_UNIT_CONSOLE, STD_UNIT_JSON, STD_UNIT_TUI};

use super::{Compiler, canonical_name};

impl Compiler {
    fn collect_module_globals(&mut self, program: &Program) {
        for decl in &program.declarations {
            let name = match decl {
                Decl::Const(c) => &c.name,
                Decl::Var(v) | Decl::MutableVar(v) => &v.name,
                Decl::TypeDef(_) | Decl::Function(_) | Decl::Procedure(_) => continue,
            };
            self.module_globals.insert(canonical_name(name));
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), CompileError> {
        if Self::program_uses_std_unit(program, STD_UNIT_CONSOLE) {
            self.register_std_console_enums();
        }
        if Self::program_uses_std_unit(program, STD_UNIT_TUI) {
            self.register_std_tui_enums();
        }
        if Self::program_uses_std_unit(program, STD_UNIT_JSON) {
            self.register_std_json_enum();
        }
        self.build_short_aliases(program);
        self.collect_module_globals(program);

        for decl in &program.declarations {
            self.compile_decl(decl)?;
        }

        for stmt in &program.body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Op::Halt, Self::location_of(&program.span));
        Ok(())
    }

    /// Whether the current declaration is at program/unit scope (not inside a callable body).
    fn is_module_level(&self) -> bool {
        self.scope_depth == 0 && self.enclosing_locals.is_empty()
    }

    pub(super) fn compile_decl(&mut self, decl: &Decl) -> Result<(), CompileError> {
        match decl {
            Decl::Const(const_def) => {
                self.compile_expr(&const_def.value)?;
                if self.is_module_level() {
                    let location = Self::location_of(&const_def.span);
                    let idx =
                        self.add_constant(Value::Str(canonical_name(&const_def.name)), location)?;
                    self.emit(Op::SetGlobal(idx), location);
                    self.emit(Op::Pop, location);
                } else {
                    let _slot = self.add_local(&const_def.name);
                }
                Ok(())
            }
            Decl::Var(var_def) | Decl::MutableVar(var_def) => {
                self.compile_expr(&var_def.value)?;
                if self.is_module_level() {
                    let location = Self::location_of(&var_def.span);
                    let idx =
                        self.add_constant(Value::Str(canonical_name(&var_def.name)), location)?;
                    self.emit(Op::SetGlobal(idx), location);
                    self.emit(Op::Pop, location);
                } else {
                    let _slot = self.add_local(&var_def.name);
                }
                Ok(())
            }
            Decl::TypeDef(type_def) => self.compile_type_decl(type_def),
            Decl::Function(function) => self.compile_function(function),
            Decl::Procedure(procedure) => self.compile_procedure(procedure),
        }
    }

    fn compile_type_decl(&mut self, type_def: &fpas_parser::TypeDef) -> Result<(), CompileError> {
        if let TypeBody::Enum(enum_ty) = &type_def.body {
            let mut variants = Vec::new();
            let mut next_value: i64 = 0;
            let mut has_data = false;
            for member in &enum_ty.members {
                let backing = member.value.unwrap_or(next_value);
                let field_names: Vec<String> = member
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect();
                if !field_names.is_empty() {
                    has_data = true;
                }
                variants.push(super::EnumVariantInfo {
                    name: member.name.clone(),
                    backing,
                    field_names,
                });
                next_value = backing + 1;
            }
            self.enums.insert(
                canonical_name(&type_def.name),
                super::EnumInfo { variants, has_data },
            );
        }

        if let TypeBody::Record(record) = &type_def.body {
            for method in &record.methods {
                self.compile_record_method(&type_def.name, method)?;
            }
        }

        Ok(())
    }
}
