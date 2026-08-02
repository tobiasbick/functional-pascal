//! Independent source-unit compilation into relocatable objects.

use fpas_parser::{Decl, Program, RecordMethod, TypeBody, Unit, Visibility};
use fpas_unit::interface::{InterfaceType, SymbolKind, UnitInterface};
use fpas_unit::object::{DefinitionKind, ObjectDefinition, ObjectImport, RelocatableObject};

use crate::compiler::Compiler;
use crate::error::{CompileError, internal_compiler_error};

/// Public interface and relocatable implementation emitted for one source unit.
pub struct CompiledUnitObject {
    /// Stable public semantic interface.
    pub interface: UnitInterface,
    /// Relocatable implementation and startup code.
    pub object: RelocatableObject,
}

/// Analyze and compile one unit without loading dependency implementation ASTs.
pub fn compile_unit_object(
    unit: &Unit,
    interfaces: &[UnitInterface],
) -> Result<CompiledUnitObject, Vec<CompileError>> {
    compile_unit_object_with_support(unit, interfaces, interfaces)
}

/// Compile one unit with direct imports and transitive qualified type interfaces.
pub fn compile_unit_object_with_support(
    unit: &Unit,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<CompiledUnitObject, Vec<CompileError>> {
    let analysis =
        fpas_sema::analyze_unit_with_interface_support(unit, interfaces, supporting_interfaces)
            .map_err(|error| {
                vec![internal_compiler_error(
                    error.to_string(),
                    "Rebuild the dependency sidecar; its semantic interface is invalid.",
                    unit.span.line,
                    unit.span.column,
                )]
            })?;
    if !analysis.metadata.errors.is_empty() {
        return Err(analysis.metadata.errors);
    }
    let Some(interface) = analysis.interface else {
        return Err(vec![internal_compiler_error(
            "Semantic analysis succeeded without producing a unit interface.",
            "This is an internal compiler error. Re-run compilation and report the source unit.",
            unit.span.line,
            unit.span.column,
        )]);
    };

    let mut compiler = Compiler::new(analysis.metadata);
    compiler
        .compile_unit(unit, interfaces)
        .map_err(|error| vec![error])?;
    let chunk = compiler.finish();
    chunk.validate_invariants().map_err(|error| {
        vec![internal_compiler_error(
            format!("Compiled unit chunk failed invariant check: {error}"),
            "This is an internal compiler error. Re-run compilation and report the source unit.",
            unit.span.line,
            unit.span.column,
        )]
    })?;

    let owner = unit.name.parts.join(".");
    let definitions = collect_definitions(unit);
    let imports = collect_imports(interfaces);
    let object = RelocatableObject::from_chunk(&owner, &chunk, definitions, imports).map_err(
        |error| {
            vec![internal_compiler_error(
                error.to_string(),
                "This is an internal compiler error. Re-run compilation and report the source unit.",
                unit.span.line,
                unit.span.column,
            )]
        },
    )?;
    Ok(CompiledUnitObject { interface, object })
}

/// Compile a root program against already analyzed unit interfaces.
pub fn compile_program_object(
    program: &Program,
    interfaces: &[UnitInterface],
) -> Result<RelocatableObject, Vec<CompileError>> {
    compile_program_object_with_support(program, interfaces, interfaces)
}

/// Compile a root program with direct imports and transitive qualified type interfaces.
pub fn compile_program_object_with_support(
    program: &Program,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<RelocatableObject, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_program_with_interface_support(
        program,
        interfaces,
        supporting_interfaces,
    )
    .map_err(|error| {
        vec![internal_compiler_error(
            error.to_string(),
            "Rebuild the dependency sidecar; its semantic interface is invalid.",
            program.span.line,
            program.span.column,
        )]
    })?;
    if !metadata.errors.is_empty() {
        return Err(metadata.errors);
    }
    let mut compiler = Compiler::new(metadata);
    compiler
        .compile_program_with_interfaces(program, interfaces)
        .map_err(|error| vec![error])?;
    let chunk = compiler.finish();
    let definitions = collect_program_definitions(program);
    let imports = collect_imports(interfaces);
    RelocatableObject::from_chunk(&program.name, &chunk, definitions, imports).map_err(|error| {
        vec![internal_compiler_error(
            error.to_string(),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            program.span.line,
            program.span.column,
        )]
    })
}

fn collect_definitions(unit: &Unit) -> Vec<ObjectDefinition> {
    let owner = unit.name.parts.join(".");
    let mut definitions = Vec::new();
    for declaration in &unit.declarations {
        let public = declaration.visibility() != Visibility::Private;
        match declaration {
            Decl::Const(value) => definitions.push(definition(
                format!("{owner}.{}", value.name),
                DefinitionKind::Global,
                public,
            )),
            Decl::Var(value) | Decl::MutableVar(value) => definitions.push(definition(
                format!("{owner}.{}", value.name),
                DefinitionKind::Global,
                public,
            )),
            Decl::Function(value) => definitions.push(definition(
                format!("{owner}.{}", value.name),
                DefinitionKind::Callable,
                public,
            )),
            Decl::Procedure(value) => definitions.push(definition(
                format!("{owner}.{}", value.name),
                DefinitionKind::Callable,
                public,
            )),
            Decl::TypeDef(value) => {
                if let TypeBody::Record(record) = &value.body {
                    let public_accessors: Vec<_> = record
                        .properties
                        .iter()
                        .filter(|property| property.visibility != Visibility::Private)
                        .flat_map(|property| [property.read.as_ref(), property.write.as_ref()])
                        .flatten()
                        .chain(
                            record
                                .events
                                .iter()
                                .filter(|event| event.visibility != Visibility::Private)
                                .flat_map(|event| [&event.read, &event.write]),
                        )
                        .collect();
                    for method in &record.methods {
                        let method_is_public = method.visibility() != Visibility::Private
                            || public_accessors
                                .iter()
                                .any(|accessor| accessor.eq_ignore_ascii_case(method.name()));
                        definitions.push(definition(
                            format!("{owner}.{}.{}", value.name, record_method_name(method)),
                            DefinitionKind::Callable,
                            public && method_is_public,
                        ));
                    }
                }
            }
        }
    }
    definitions.sort_by(|left, right| left.name.cmp(&right.name));
    definitions
}

fn collect_program_definitions(program: &Program) -> Vec<ObjectDefinition> {
    let mut definitions = Vec::new();
    for declaration in &program.declarations {
        let (name, kind) = match declaration {
            Decl::Const(value) => (&value.name, DefinitionKind::Global),
            Decl::Var(value) | Decl::MutableVar(value) => (&value.name, DefinitionKind::Global),
            Decl::Function(value) => (&value.name, DefinitionKind::Callable),
            Decl::Procedure(value) => (&value.name, DefinitionKind::Callable),
            Decl::TypeDef(_) => continue,
        };
        definitions.push(definition(name.clone(), kind, false));
    }
    definitions
}

fn collect_imports(interfaces: &[UnitInterface]) -> Vec<ObjectImport> {
    let mut imports = Vec::new();
    for interface in interfaces {
        for symbol in &interface.symbols {
            match symbol.kind {
                SymbolKind::Constant(_) | SymbolKind::Variable | SymbolKind::MutableVariable => {
                    imports.push(ObjectImport {
                        name: symbol.qualified_name.clone(),
                        kind: DefinitionKind::Global,
                    })
                }
                SymbolKind::Function | SymbolKind::Procedure => imports.push(ObjectImport {
                    name: symbol.qualified_name.clone(),
                    kind: DefinitionKind::Callable,
                }),
                SymbolKind::Type => collect_record_imports(&symbol.ty, &mut imports),
                SymbolKind::EnumMember(_) | SymbolKind::EnumVariantConstructor => {}
            }
        }
    }
    imports.sort_by(|left, right| left.name.cmp(&right.name));
    imports.dedup_by(|left, right| {
        left.kind == right.kind && left.name.eq_ignore_ascii_case(&right.name)
    });
    imports
}

fn collect_record_imports(ty: &InterfaceType, imports: &mut Vec<ObjectImport>) {
    let InterfaceType::Record(record) = ty else {
        return;
    };
    let member_is_public = |name: &str| {
        !record
            .private_members
            .iter()
            .any(|private| private.eq_ignore_ascii_case(name))
    };
    for method in record
        .methods
        .iter()
        .chain(&record.static_routines)
        .filter(|method| member_is_public(&method.name))
    {
        imports.push(ObjectImport {
            name: format!("{}.{}", record.name, method.name),
            kind: DefinitionKind::Callable,
        });
    }
    for property in record
        .properties
        .iter()
        .filter(|property| member_is_public(&property.name))
    {
        for name in [property.getter.as_ref(), property.setter.as_ref()]
            .into_iter()
            .flatten()
        {
            imports.push(ObjectImport {
                name: name.clone(),
                kind: DefinitionKind::Callable,
            });
        }
    }
    for event in record
        .events
        .iter()
        .filter(|event| member_is_public(&event.name))
    {
        for name in [&event.getter, &event.setter] {
            imports.push(ObjectImport {
                name: name.clone(),
                kind: DefinitionKind::Callable,
            });
        }
    }
}

fn definition(name: String, kind: DefinitionKind, public: bool) -> ObjectDefinition {
    ObjectDefinition {
        name: name.to_ascii_lowercase(),
        kind,
        public,
    }
}

fn record_method_name(method: &RecordMethod) -> &str {
    match method {
        RecordMethod::Function(value) | RecordMethod::StaticFunction(value) => &value.name,
        RecordMethod::Procedure(value) | RecordMethod::StaticProcedure(value) => &value.name,
    }
}
