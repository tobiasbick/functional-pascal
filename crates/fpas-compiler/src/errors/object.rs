//! Preserve essential schemas and the hidden constructor ABI in cached unit objects.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::*;
use fpas_unit::{
    errors::ErrorRoutineConstructors,
    object::{
        ImportShape, ObjectConstant, ObjectDebugLocation, ObjectErrorSchema, ObjectImport,
        RelocatableObject, SymbolReference,
    },
};

impl ErrorMetadata {
    pub(crate) fn apply_object(
        &self,
        object: &mut RelocatableObject,
        plan: &mut crate::lowering::ImportPlan,
    ) -> Result<(), fpas_unit::object::ObjectError> {
        if self.schemas.values().any(|schema| {
            schema
                .parameters
                .contains(&Some(fpas_unit::interface::TypeConstraint::Equatable))
        }) {
            object.equality_declarations = self.equality_declarations().map_err(|_| {
                fpas_unit::object::ObjectError::InvalidTableReference("equality declarations")
            })?;
        }
        for (&id, signature) in &self.requirements {
            if let Some(function) = object.functions.get_mut(id.get() as usize)
                && !signature.requirements.is_empty()
            {
                function.error_constructors = Some(signature.clone());
            }
        }
        for (&id, display) in &self.displays {
            if let Some(function) = object.functions.get_mut(id.get() as usize) {
                function.display_name = Some(display.clone());
            }
        }
        for (id, import) in &mut plan.functions {
            if let ImportShape::Function { arity, .. } = &mut import.shape {
                *arity = object
                    .functions
                    .get(id.get() as usize)
                    .ok_or(fpas_unit::object::ObjectError::InvalidTableReference(
                        "error routine import",
                    ))?
                    .arity;
            }
        }
        for (&id, request) in &self.factories {
            for constant in &mut object.constants {
                if matches!(constant, ObjectConstant::Function { function: SymbolReference::Local(index), .. } if *index == id.get())
                {
                    *constant = ObjectConstant::ErrorConstructor(request.clone());
                }
            }
            let value = ObjectConstant::ErrorConstructor(request.clone());
            let constant = match object
                .constants
                .iter()
                .position(|existing| existing == &value)
            {
                Some(index) => index,
                None => {
                    object.constants.push(value);
                    object.constants.len() - 1
                }
            };
            let constant = u32::try_from(constant).map_err(|_| {
                fpas_unit::object::ObjectError::Overflow("error constructor constant")
            })?;
            for relocation in &mut object.relocations {
                if matches!(relocation.kind, fpas_unit::object::RelocationKind::Function(SymbolReference::Local(index)) if index == id.get())
                {
                    let word = &mut object.functions[relocation.function as usize].code
                        [relocation.instruction as usize];
                    let instruction = fpas_bytecode::Instruction::from_word(*word);
                    if instruction
                        .opcode()
                        .map_err(|e| fpas_unit::object::ObjectError::Instruction(e.to_string()))?
                        != fpas_bytecode::Opcode::MakeClosure
                    {
                        return Err(fpas_unit::object::ObjectError::InvalidTableReference(
                            "error factory must be an ordinary noncapturing value",
                        ));
                    }
                    let operands = instruction
                        .abc_operands()
                        .map_err(|e| fpas_unit::object::ObjectError::Instruction(e.to_string()))?;
                    *word = fpas_bytecode::Instruction::abx(
                        fpas_bytecode::Opcode::LoadConstant,
                        operands.a,
                        constant,
                    )
                    .map_err(|e| fpas_unit::object::ObjectError::Instruction(e.to_string()))?
                    .word();
                    relocation.kind = fpas_unit::object::RelocationKind::Constant(constant);
                }
            }
            let function = &object.functions[id.get() as usize];
            plan.functions.push((
                id,
                ObjectImport {
                    name: function.name.clone(),
                    shape: ImportShape::Function {
                        arity: function.arity,
                        capture_count: 0,
                        returns_value: true,
                    },
                },
            ));
        }
        for (declaration, span) in &self.locations {
            let schema = self.schemas.get(declaration).ok_or(
                fpas_unit::object::ObjectError::InvalidTableReference("local error schema"),
            )?;
            object.error_schemas.push(ObjectErrorSchema {
                schema: schema.clone(),
                location: ObjectDebugLocation {
                    source: 0,
                    line: span.line,
                    column: span.column,
                },
            });
        }
        Ok(())
    }

    pub(crate) fn export_requirements(
        &self,
        interface: &mut fpas_unit::interface::UnitInterface,
        functions: &[fpas_ir::Function],
    ) -> Result<(), String> {
        let mut routines = std::collections::BTreeSet::new();
        for symbol in &interface.symbols {
            match &symbol.ty {
                fpas_unit::interface::InterfaceType::Record(record)
                    if symbol.kind == fpas_unit::interface::SymbolKind::Type =>
                {
                    for method in record.methods.iter().chain(&record.static_routines) {
                        if !record
                            .private_members
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(&method.name))
                        {
                            routines.insert(
                                format!("{}.{}", record.name, method.name).to_ascii_lowercase(),
                            );
                        }
                    }
                }
                _ if matches!(
                    symbol.kind,
                    fpas_unit::interface::SymbolKind::Function
                        | fpas_unit::interface::SymbolKind::Procedure
                ) =>
                {
                    routines.insert(symbol.qualified_name.to_ascii_lowercase());
                }
                _ => {}
            }
        }
        for qualified in routines {
            let Some(function) = functions.iter().find(|function| {
                function.name.eq_ignore_ascii_case(&qualified)
                    || format!("{}.{}", self.owner, function.name).eq_ignore_ascii_case(&qualified)
            }) else {
                continue;
            };
            if let Some(signature) = self.requirements.get(&function.id)
                && !signature.requirements.is_empty()
            {
                interface.errors.routines.push(ErrorRoutineConstructors {
                    routine: qualified,
                    signature: signature.clone(),
                });
            }
        }
        interface
            .errors
            .routines
            .sort_by(|left, right| left.routine.cmp(&right.routine));
        let mut pending = interface
            .errors
            .routines
            .iter()
            .flat_map(|routine| &routine.signature.requirements)
            .map(|request| request.instance.declaration.clone())
            .collect::<Vec<_>>();
        let mut seen = std::collections::BTreeSet::new();
        while let Some(declaration) = pending.pop() {
            if !seen.insert(declaration.clone()) {
                continue;
            }
            let Some(schema) = self.schemas.get(&declaration) else {
                continue;
            };
            if !interface
                .errors
                .schemas
                .iter()
                .any(|existing| existing.declaration == declaration)
            {
                interface.errors.schemas.push(schema.clone());
            }
            let mut fields = schema
                .variants
                .iter()
                .flat_map(|variant| &variant.fields)
                .map(|field| &field.ty)
                .collect::<Vec<_>>();
            while let Some(ty) = fields.pop() {
                use fpas_bytecode::TypeIdentity as T;
                match ty {
                    T::Nominal {
                        kind,
                        declaration,
                        arguments,
                    } => {
                        if *kind == fpas_bytecode::NominalKind::Error {
                            pending.push(declaration.clone());
                        }
                        fields.extend(arguments);
                    }
                    T::Array(ty) | T::Channel(ty) | T::Task(ty) | T::Option(ty) => fields.push(ty),
                    T::Dictionary(left, right) | T::Result(left, right) => {
                        fields.push(left);
                        fields.push(right);
                    }
                    T::Callable {
                        parameters, result, ..
                    } => {
                        fields.extend(parameters.iter().map(|parameter| &parameter.ty));
                        fields.extend(result.iter().map(Box::as_ref));
                    }
                    _ => {}
                }
            }
        }
        interface
            .errors
            .schemas
            .sort_by(|left, right| left.declaration.cmp(&right.declaration));
        if interface.errors.schemas.iter().any(|schema| {
            schema
                .parameters
                .contains(&Some(fpas_unit::interface::TypeConstraint::Equatable))
        }) {
            interface.errors.equality_declarations = self.equality_declarations()?;
            interface
                .errors
                .equality_declarations
                .sort_by(|left, right| left.identity.cmp(&right.identity));
        }
        Ok(())
    }
}
