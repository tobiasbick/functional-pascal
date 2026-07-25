//! Program- and declaration-level lowering.
//!
//! **Documentation:** `docs/pascal/language/functions/README.md`, `docs/pascal/program-structure/units.md` (from the repository root).

mod callables;

use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Decl, Program, TypeBody, Unit};
use fpas_unit::interface::{InterfaceType, UnitInterface};

use fpas_std::{STD_UNIT_CONSOLE, STD_UNIT_JSON, STD_UNIT_TOML};

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
        if Self::program_uses_std_unit(program, STD_UNIT_JSON) {
            self.register_std_json_enum();
        }
        if Self::program_uses_std_unit(program, STD_UNIT_TOML) {
            self.register_std_toml_enum();
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

    pub fn compile_program_with_interfaces(
        &mut self,
        program: &Program,
        interfaces: &[UnitInterface],
    ) -> Result<(), CompileError> {
        self.build_program_interface_aliases(interfaces);
        self.register_interface_enums(interfaces);
        self.compile_program(program)
    }

    pub fn compile_unit(
        &mut self,
        unit: &Unit,
        interfaces: &[UnitInterface],
    ) -> Result<(), CompileError> {
        if Self::uses_std_unit(&unit.uses, STD_UNIT_CONSOLE) {
            self.register_std_console_enums();
        }
        if Self::uses_std_unit(&unit.uses, STD_UNIT_JSON) {
            self.register_std_json_enum();
        }
        if Self::uses_std_unit(&unit.uses, STD_UNIT_TOML) {
            self.register_std_toml_enum();
        }
        let owner = unit.name.parts.join(".");
        self.set_owner_unit(&owner);
        self.build_unit_short_aliases(unit, interfaces);
        self.register_interface_enums(interfaces);
        self.collect_unit_globals(unit);

        for declaration in &unit.declarations {
            self.compile_decl(declaration)?;
        }
        self.emit(Op::Halt, Self::location_of(&unit.span));
        Ok(())
    }

    fn collect_unit_globals(&mut self, unit: &Unit) {
        for declaration in &unit.declarations {
            let name = match declaration {
                Decl::Const(value) => &value.name,
                Decl::Var(value) | Decl::MutableVar(value) => &value.name,
                _ => continue,
            };
            self.module_globals.insert(canonical_name(name));
        }
    }

    fn register_interface_enums(&mut self, interfaces: &[UnitInterface]) {
        for interface in interfaces {
            for symbol in &interface.symbols {
                let InterfaceType::Enum(enum_ty) = &symbol.ty else {
                    continue;
                };
                let mut next_value = 0_i64;
                let variants: Vec<_> = enum_ty
                    .variants
                    .iter()
                    .map(|variant| {
                        let backing = variant.backing_value.unwrap_or(next_value);
                        next_value = backing.saturating_add(1);
                        super::EnumVariantInfo {
                            name: variant.name.clone(),
                            backing,
                            field_names: variant
                                .fields
                                .iter()
                                .map(|field| field.name.clone())
                                .collect(),
                        }
                    })
                    .collect();
                let info = super::EnumInfo {
                    type_name: enum_ty.name.clone(),
                    has_data: variants
                        .iter()
                        .any(|variant| !variant.field_names.is_empty()),
                    variants,
                };
                self.enums
                    .insert(canonical_name(&enum_ty.name), info.clone());
                self.enums.insert(canonical_name(&symbol.name), info);
            }
        }
    }

    /// Whether the current declaration is at program/unit scope (not inside a callable body).
    fn is_module_level(&self) -> bool {
        self.scope_depth == 0 && self.enclosing_locals.is_empty()
    }

    pub(super) fn compile_decl(&mut self, decl: &Decl) -> Result<(), CompileError> {
        match decl {
            Decl::Const(const_def) => {
                let location = Self::location_of(&const_def.span);
                self.compile_expr(&const_def.value)?;
                if self.is_module_level() {
                    let idx = self.add_constant(
                        Value::Str(
                            canonical_name(&self.qualify_owned_name(&const_def.name)).into(),
                        ),
                        location,
                    )?;
                    self.emit(Op::SetGlobal(idx), location);
                    self.emit(Op::Pop, location);
                } else {
                    let _slot = self.add_local(&const_def.name, location)?;
                }
                Ok(())
            }
            Decl::Var(var_def) | Decl::MutableVar(var_def) => {
                let location = Self::location_of(&var_def.span);
                self.compile_expr(&var_def.value)?;
                if self.is_module_level() {
                    let idx = self.add_constant(
                        Value::Str(canonical_name(&self.qualify_owned_name(&var_def.name)).into()),
                        location,
                    )?;
                    self.emit(Op::SetGlobal(idx), location);
                    self.emit(Op::Pop, location);
                } else {
                    let _slot = self.add_local(&var_def.name, location)?;
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
            let runtime_type_name = self.qualify_owned_name(&type_def.name);
            self.enums.insert(
                canonical_name(&type_def.name),
                super::EnumInfo {
                    type_name: runtime_type_name.clone(),
                    variants,
                    has_data,
                },
            );
            if !runtime_type_name.eq_ignore_ascii_case(&type_def.name)
                && let Some(info) = self.enums.get(&canonical_name(&type_def.name)).cloned()
            {
                self.enums.insert(canonical_name(&runtime_type_name), info);
            }
        }

        if let TypeBody::Alias(fpas_parser::TypeExpr::Named { id, .. }) = &type_def.body {
            let source_name = id.parts.join(".");
            if let Some(info) = self.enums.get(&canonical_name(&source_name)).cloned() {
                self.enums.insert(canonical_name(&type_def.name), info);
            }
        }

        if let TypeBody::Record(record) = &type_def.body {
            let runtime_type_name = self.qualify_owned_name(&type_def.name);
            for method in &record.methods {
                self.compile_record_method(&runtime_type_name, method)?;
            }
        }

        Ok(())
    }
}
