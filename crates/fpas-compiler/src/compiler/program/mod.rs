//! Program- and declaration-level lowering.
//!
//! **Documentation:** `docs/pascal/04-functions.md`, `docs/pascal/09-units.md` (from the repository root).

mod callables;

use crate::error::CompileError;
use fpas_bytecode::Op;
use fpas_parser::{Decl, Program, RecordMethod, TypeBody};

use super::{Compiler, canonical_name};

impl Compiler {
    pub fn compile_program(&mut self, program: &Program) -> Result<(), CompileError> {
        if Self::program_uses_std_console(program) {
            self.register_std_console_enums();
        }
        if Self::program_uses_std_tui(program) {
            self.register_std_tui_enums();
        }
        self.build_short_aliases(program);

        for decl in &program.declarations {
            self.compile_decl(decl)?;
        }

        for stmt in &program.body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Op::Halt, Self::location_of(&program.span));
        Ok(())
    }

    pub(super) fn compile_decl(&mut self, decl: &Decl) -> Result<(), CompileError> {
        match decl {
            Decl::Const(const_def) => {
                self.compile_expr(&const_def.value)?;
                let _slot = self.add_local(&const_def.name);
                Ok(())
            }
            Decl::Var(var_def) | Decl::MutableVar(var_def) => {
                self.compile_expr(&var_def.value)?;
                let _slot = self.add_local(&var_def.name);
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
            let method_names: Vec<String> = record
                .methods
                .iter()
                .map(|method| match method {
                    RecordMethod::Function(function) => function.name.clone(),
                    RecordMethod::Procedure(procedure) => procedure.name.clone(),
                })
                .collect();
            if !method_names.is_empty() {
                self.record_methods
                    .insert(type_def.name.clone(), method_names);
            }
            for method in &record.methods {
                self.compile_record_method(&type_def.name, method)?;
            }
        }

        Ok(())
    }
}
