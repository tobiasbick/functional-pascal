//! Recursive syntax-to-type mapping for recovered native receiver chains.
//! Reference: `docs/pascal/tools/editor-integration.md`.

use super::MAX_RECEIVER_DEPTH;
use crate::navigation::{NavigationDocument, find_type};
use crate::{DocumentSymbol, SymbolKind};
use fpas_parser::{CompilationUnit, Decl, FormalParam, TypeBody, TypeExpr};
use fpas_sema::{FunctionTy, ParamTy, ProcedureTy, Ty};
use std::collections::HashMap;

type ResolvingTypes = HashMap<(usize, usize), Ty>;

/// Resolves a declaration's value type, retaining the signature of a routine value.
/// See `docs/pascal/tools/editor-integration.md`.
pub(super) fn from_symbol(
    documents: &[NavigationDocument],
    owner: usize,
    symbol: &DocumentSymbol,
    depth: usize,
) -> Option<Ty> {
    if let Some(semantic) = documents[owner]
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
        && let Some(ty) = semantic.metadata().value_declaration_types.get(&(
            symbol.full_span.source_id(),
            symbol.full_span.offset(),
            symbol.name.to_ascii_lowercase(),
        ))
        // Imported applied records can be references without member tables. Resolve
        // their declarations below before substituting the concrete field types.
        && !matches!(ty, Ty::Record(record) if record.fields.is_empty() && !record.type_args.is_empty())
    {
        return Some(ty.clone());
    }
    if symbol.kind == SymbolKind::EnumMember && symbol.callable.is_some() {
        let Ty::Enum(enumeration) = named_type(
            documents,
            owner,
            symbol.type_name.as_deref()?,
            depth,
            &mut HashMap::new(),
        )?
        else {
            return None;
        };
        return enumeration.constructor_type(&symbol.name).map(Ty::Function);
    }
    if matches!(
        symbol.kind,
        SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method
    ) && let Some(callable) = &symbol.callable
    {
        let signature = &callable.label;
        let kind = signature.split_whitespace().next()?;
        let parameters = signature.get(signature.find('(')?..)?;
        return from_text(documents, owner, &format!("{kind}{parameters}"), depth);
    }
    let name = symbol
        .detail
        .split_once(": ")
        .map(|(_, ty)| ty)
        .or(symbol.type_name.as_deref());
    let Some(name) = name else {
        if !matches!(symbol.kind, SymbolKind::Constant | SymbolKind::Variable) {
            return None;
        }
        // Recover an inferred local's initializer while its following member access is incomplete.
        let start = symbol.full_span.offset();
        let end = start.checked_add(symbol.full_span.length())?;
        let declaration = documents[owner].snapshot.source().get(start..end)?;
        let (_, initializer) = declaration.split_once(":=")?;
        let initializer = initializer.trim().trim_end_matches(';').trim();
        return super::receiver_type(documents, owner, initializer, end, depth + 1);
    };
    if let Some((_, parameter)) =
        crate::navigation::resolve_unqualified(documents, owner, name, symbol.full_span.offset())
        && parameter.kind == SymbolKind::TypeParameter
    {
        let constraint = parameter
            .detail
            .rsplit_once(": ")
            .and_then(|(_, name)| fpas_sema::TypeConstraint::from_name(name));
        return Some(Ty::GenericParam(
            parameter.qualified_name.clone(),
            constraint,
        ));
    }
    if let Some(semantic) = documents[owner]
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
        && let Some((_, ty)) = semantic
            .metadata()
            .named_types
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
    {
        return Some(ty.clone());
    }
    from_text(documents, owner, name, depth)
}

/// Parses a type fragment with the shared parser before resolving its receiver shape.
pub(super) fn from_text(
    documents: &[NavigationDocument],
    owner: usize,
    text: &str,
    depth: usize,
) -> Option<Ty> {
    let (syntax, diagnostics) = fpas_parser::parse_type_expression(text);
    if !diagnostics.is_empty() {
        return None;
    }
    from_syntax(documents, owner, &syntax, depth)
}

/// Maps recursive type syntax, resolving aliases in their declaring unit's import environment.
pub(super) fn from_syntax(
    documents: &[NavigationDocument],
    owner: usize,
    syntax: &TypeExpr,
    depth: usize,
) -> Option<Ty> {
    from_syntax_in_scope(
        documents,
        owner,
        syntax,
        depth,
        &HashMap::new(),
        &mut HashMap::new(),
    )
}

fn from_syntax_in_scope(
    documents: &[NavigationDocument],
    owner: usize,
    syntax: &TypeExpr,
    depth: usize,
    parameters: &HashMap<String, Ty>,
    resolving: &mut ResolvingTypes,
) -> Option<Ty> {
    if depth > MAX_RECEIVER_DEPTH {
        return None;
    }
    let resolve = |inner, resolving: &mut ResolvingTypes| {
        from_syntax_in_scope(documents, owner, inner, depth + 1, parameters, resolving)
            .map(Box::new)
    };
    Some(match syntax {
        TypeExpr::Application { id, arguments, .. } => {
            let name = id.parts.join(".");
            let arguments: Vec<_> = arguments
                .iter()
                .map(|argument| {
                    from_syntax_in_scope(
                        documents,
                        owner,
                        argument,
                        depth + 1,
                        parameters,
                        resolving,
                    )
                })
                .collect::<Option<_>>()?;
            let base = named_type(documents, owner, &name, depth, resolving)?;
            match base {
                Ty::Record(record) => {
                    Ty::Record(std::sync::Arc::new(record.instantiate(arguments)))
                }
                Ty::Enum(enumeration) => {
                    Ty::Enum(std::sync::Arc::new(enumeration.instantiate(arguments)))
                }
                other => other,
            }
        }
        TypeExpr::Named { id, .. } => {
            let name = id.parts.join(".");
            if let Some(ty) = parameters.get(&name.to_ascii_lowercase()) {
                return Some(ty.clone());
            }
            match name.to_ascii_lowercase().as_str() {
                "string" => Ty::String,
                "integer" => Ty::Integer,
                "boolean" => Ty::Boolean,
                "real" => Ty::Real,
                "task" => Ty::Task(Box::new(Ty::Unit)),
                _ => return named_type(documents, owner, &name, depth, resolving),
            }
        }
        TypeExpr::Array(inner, _) => Ty::Array(resolve(inner, resolving)?),
        TypeExpr::Channel(inner, _) => Ty::Channel(resolve(inner, resolving)?),
        TypeExpr::Task(inner, _) => Ty::Task(resolve(inner, resolving)?),
        TypeExpr::Option { inner_type, .. } => Ty::Option(resolve(inner_type, resolving)?),
        TypeExpr::Result {
            ok_type, err_type, ..
        } => Ty::Result(resolve(ok_type, resolving)?, resolve(err_type, resolving)?),
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => Ty::Dict(
            resolve(key_type, resolving)?,
            resolve(value_type, resolving)?,
        ),
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: formal_parameters(documents, owner, params, depth, parameters, resolving)?,
            return_type: resolve(return_type, resolving)?,
            variadic: false,
        }),
        TypeExpr::ProcedureType { params, .. } => Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params: formal_parameters(documents, owner, params, depth, parameters, resolving)?,
            variadic: false,
        }),
    })
}

fn formal_parameters(
    documents: &[NavigationDocument],
    owner: usize,
    params: &[FormalParam],
    depth: usize,
    parameters: &HashMap<String, Ty>,
    resolving: &mut ResolvingTypes,
) -> Option<Vec<ParamTy>> {
    params
        .iter()
        .map(|param| {
            Some(ParamTy {
                name: param.name.clone(),
                mode: param.mode,
                ty: from_syntax_in_scope(
                    documents,
                    owner,
                    &param.type_expr,
                    depth + 1,
                    parameters,
                    resolving,
                )?,
            })
        })
        .collect()
}

fn named_type(
    documents: &[NavigationDocument],
    owner: usize,
    name: &str,
    depth: usize,
    resolving: &mut ResolvingTypes,
) -> Option<Ty> {
    if name.eq_ignore_ascii_case("ArrayPopResult") {
        return Some(fpas_sema::array_pop_result_declaration());
    }
    let Some((index, symbol)) = find_type(documents, owner, owner, name) else {
        return Some(Ty::Named(name.to_owned()));
    };
    let key = (index, symbol.full_span.offset());
    if let Some(header) = resolving.get(&key) {
        return Some(header.clone());
    }
    if let Some(semantic) = documents[index]
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
        && let Some((_, ty)) = semantic.metadata().named_types.iter().find(|(name, _)| {
            name.eq_ignore_ascii_case(&symbol.name)
                || name.eq_ignore_ascii_case(&symbol.qualified_name)
        })
    {
        return Some(ty.clone());
    }
    let declarations = match documents[index].snapshot.compilation_unit() {
        CompilationUnit::Program(program) => &program.declarations,
        CompilationUnit::Unit(unit) => &unit.declarations,
    };
    let definition = declarations
        .iter()
        .find_map(|declaration| match declaration {
            Decl::TypeDef(definition) if definition.span.offset == symbol.full_span.offset() => {
                Some(definition)
            }
            _ => None,
        })?;
    match &definition.body {
        TypeBody::Enum(enumeration) => {
            let type_params = definition
                .type_params
                .iter()
                .map(|parameter| fpas_sema::GenericParamDef {
                    name: parameter.name.clone(),
                    constraint: parameter
                        .constraint
                        .as_deref()
                        .and_then(fpas_sema::TypeConstraint::from_name),
                })
                .collect::<Vec<_>>();
            let parameters = type_params
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.to_ascii_lowercase(),
                        Ty::GenericParam(parameter.name.clone(), parameter.constraint),
                    )
                })
                .collect::<HashMap<_, _>>();
            let mut resolved = fpas_sema::EnumTy {
                is_error: enumeration.is_error,
                ..fpas_sema::EnumTy::header(symbol.qualified_name.clone(), type_params)
            };
            // Recursive payloads retain nominal headers while their declarations resolve.
            resolving.insert(key, Ty::Enum(std::sync::Arc::new(resolved.clone())));
            let variants = enumeration
                .members
                .iter()
                .map(|member| {
                    Some(fpas_sema::EnumVariantTy {
                        name: member.name.clone(),
                        message: member.message.clone(),
                        backing_value: member.value,
                        fields: member
                            .fields
                            .iter()
                            .map(|field| {
                                Some((
                                    field.name.clone(),
                                    from_syntax_in_scope(
                                        documents,
                                        index,
                                        &field.type_expr,
                                        depth + 1,
                                        &parameters,
                                        resolving,
                                    )?,
                                ))
                            })
                            .collect::<Option<_>>()?,
                    })
                })
                .collect::<Option<_>>()?;
            resolved.variants = variants;
            resolving.remove(&key);
            Some(Ty::Enum(std::sync::Arc::new(resolved)))
        }
        TypeBody::Alias(syntax) => from_syntax_in_scope(
            documents,
            index,
            syntax,
            depth + 1,
            &HashMap::new(),
            resolving,
        ),
        TypeBody::Record(record) => {
            let type_params: Vec<_> = definition
                .type_params
                .iter()
                .map(|parameter| fpas_sema::GenericParamDef {
                    name: parameter.name.clone(),
                    constraint: parameter
                        .constraint
                        .as_deref()
                        .and_then(fpas_sema::TypeConstraint::from_name),
                })
                .collect();
            let parameters: HashMap<_, _> = type_params
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.to_ascii_lowercase(),
                        Ty::GenericParam(parameter.name.clone(), parameter.constraint),
                    )
                })
                .collect();
            let mut resolved = fpas_sema::RecordTy {
                name: symbol.qualified_name.clone(),
                owner_unit: Some(documents[index].owner.clone()),
                type_params,
                type_args: Vec::new(),
                fields: Vec::new(),
                private_members: record
                    .fields
                    .iter()
                    .filter(|field| field.visibility == fpas_parser::Visibility::Private)
                    .map(|field| field.name.clone())
                    .collect(),
                methods: Vec::new(),
                static_functions: Vec::new(),
                static_procedures: Vec::new(),
            };
            resolving.insert(key, Ty::Record(std::sync::Arc::new(resolved.clone())));
            let fields = record
                .fields
                .iter()
                .map(|field| {
                    Some((
                        field.name.clone(),
                        from_syntax_in_scope(
                            documents,
                            index,
                            &field.type_expr,
                            depth + 1,
                            &parameters,
                            resolving,
                        )?,
                    ))
                })
                .collect::<Option<_>>()?;
            resolved.fields = fields;
            resolving.remove(&key);
            Some(Ty::Record(std::sync::Arc::new(resolved)))
        }
        _ => Some(Ty::Named(symbol.qualified_name.clone())),
    }
}
