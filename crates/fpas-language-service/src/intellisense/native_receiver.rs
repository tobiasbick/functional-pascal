//! Static receiver shapes for catalog completion, including recovered expressions.

use crate::SymbolKind;
use crate::navigation::{NavigationDocument, find_type, resolve_qualified, resolve_unqualified};
use fpas_parser::{Expr, PostfixOperation};
use fpas_sema::Ty;
mod inference;
use inference::{operation_result, shape};

pub(super) fn receiver_type(
    documents: &[NavigationDocument],
    target: usize,
    receiver: &str,
    offset: usize,
    depth: usize,
) -> Option<Ty> {
    if depth > 16 {
        return None;
    }
    let receiver = receiver.trim();
    let document = &documents[target];
    // Completed chains retain the exact semantic result, including generic Map outputs.
    let end = document.snapshot.source().get(..offset)?.rfind(receiver)? + receiver.len();
    if let Some(semantic) = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
    {
        for call in semantic.metadata().fluent_calls.values() {
            let start = document.tokens.iter().position(|token| {
                token.span.offset >= call.call_span.offset
                    && matches!(token.token, fpas_lexer::Token::LParen)
            });
            if let Some(start) = start {
                let mut nesting = 0usize;
                for token in &document.tokens[start..] {
                    match token.token {
                        fpas_lexer::Token::LParen => nesting += 1,
                        fpas_lexer::Token::RParen => {
                            nesting -= 1;
                            if nesting == 0 {
                                if token.span.offset + token.span.length == end {
                                    return Some(call.result_ty.clone());
                                }
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if receiver.starts_with('(') && receiver.ends_with(')') {
        return receiver_type(
            documents,
            target,
            &receiver[1..receiver.len() - 1],
            offset,
            depth + 1,
        );
    }
    if receiver.starts_with('\'') && receiver.ends_with('\'') {
        return Some(Ty::String);
    }
    if receiver.parse::<i64>().is_ok() {
        return Some(Ty::Integer);
    }
    if receiver.parse::<f64>().is_ok() {
        return Some(Ty::Real);
    }
    if receiver.eq_ignore_ascii_case("true") || receiver.eq_ignore_ascii_case("false") {
        return Some(Ty::Boolean);
    }
    let (expression, diagnostics) = fpas_parser::parse_expression(receiver);
    if diagnostics.is_empty() {
        match &expression {
            Expr::ArrayLiteral(values, _) => {
                let inner = values
                    .first()
                    .and_then(|value| {
                        receiver_type(
                            documents,
                            target,
                            value.span().text(receiver)?,
                            offset,
                            depth + 1,
                        )
                    })
                    .unwrap_or(Ty::Error);
                return Some(Ty::Array(Box::new(inner)));
            }
            Expr::DictLiteral(values, _) => {
                let pair = values
                    .first()
                    .and_then(|(key, value)| {
                        Some((
                            receiver_type(
                                documents,
                                target,
                                key.span().text(receiver)?,
                                offset,
                                depth + 1,
                            )?,
                            receiver_type(
                                documents,
                                target,
                                value.span().text(receiver)?,
                                offset,
                                depth + 1,
                            )?,
                        ))
                    })
                    .unwrap_or((Ty::Error, Ty::Error));
                return Some(Ty::Dict(Box::new(pair.0), Box::new(pair.1)));
            }
            Expr::OptionSome(value, _) => {
                return Some(Ty::Option(Box::new(receiver_type(
                    documents,
                    target,
                    value.span().text(receiver)?,
                    offset,
                    depth + 1,
                )?)));
            }
            Expr::OptionNone(_) => return Some(Ty::Option(Box::new(Ty::Error))),
            Expr::ResultOk(value, _) => {
                return Some(Ty::Result(
                    Box::new(receiver_type(
                        documents,
                        target,
                        value.span().text(receiver)?,
                        offset,
                        depth + 1,
                    )?),
                    Box::new(Ty::Error),
                ));
            }
            Expr::ResultError(value, _) => {
                return Some(Ty::Result(
                    Box::new(Ty::Error),
                    Box::new(receiver_type(
                        documents,
                        target,
                        value.span().text(receiver)?,
                        offset,
                        depth + 1,
                    )?),
                ));
            }
            Expr::Designator(designator)
                if designator
                    .parts
                    .iter()
                    .any(|part| matches!(part, fpas_parser::DesignatorPart::Index(_, _))) =>
            {
                let first_index = designator
                    .parts
                    .iter()
                    .position(|part| matches!(part, fpas_parser::DesignatorPart::Index(_, _)))?;
                let root = designator.parts[..first_index]
                    .iter()
                    .map(|part| match part {
                        fpas_parser::DesignatorPart::Ident(name, _) => Some(name.as_str()),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(".");
                let mut ty = receiver_type(documents, target, &root, offset, depth + 1)?;
                for part in &designator.parts[first_index..] {
                    ty = match (part, ty) {
                        (fpas_parser::DesignatorPart::Index(_, _), Ty::Array(inner)) => *inner,
                        (fpas_parser::DesignatorPart::Index(_, _), Ty::Dict(_, value)) => *value,
                        (fpas_parser::DesignatorPart::Index(_, _), Ty::String) => Ty::String,
                        (fpas_parser::DesignatorPart::Ident(name, _), Ty::Record(record)) => record
                            .fields
                            .iter()
                            .find(|(field, _)| field.eq_ignore_ascii_case(name))?
                            .1
                            .clone(),
                        _ => return None,
                    };
                }
                return Some(ty);
            }
            Expr::Call {
                designator, args, ..
            } => {
                let parts = &designator.parts;
                if let [
                    fpas_parser::DesignatorPart::Ident(owner, _),
                    fpas_parser::DesignatorPart::Ident(name, _),
                ] = parts.as_slice()
                    && let Some(operation) = fpas_sema::native_factory(owner, name)
                {
                    if operation.name == "Chr" {
                        return Some(Ty::String);
                    }
                    let value = args
                        .iter()
                        .find(|arg| {
                            arg.argument_name()
                                .is_some_and(|name| name.eq_ignore_ascii_case("Value"))
                        })
                        .or_else(|| args.first())?
                        .argument_value();
                    return Some(Ty::Array(Box::new(receiver_type(
                        documents,
                        target,
                        value.span().text(receiver)?,
                        offset,
                        depth + 1,
                    )?)));
                }
                if let Some(fpas_parser::DesignatorPart::Ident(name, span)) = parts.last() {
                    let prefix = receiver.get(..span.offset)?.trim_end_matches('.');
                    if !prefix.is_empty()
                        && let Some(ty) =
                            receiver_type(documents, target, prefix, offset, depth + 1)
                        && let Some(result) = operation_result(
                            documents,
                            target,
                            &ty,
                            name,
                            args,
                            receiver,
                            offset,
                            depth + 1,
                        )
                    {
                        return Some(result);
                    }
                }
            }
            _ => {}
        }
    }
    if diagnostics.is_empty()
        && let Expr::Postfix {
            base, operations, ..
        } = &expression
    {
        let mut ty = receiver_type(
            documents,
            target,
            base.span().text(receiver)?,
            offset,
            depth + 1,
        )?;
        for operation in operations {
            ty = match operation {
                PostfixOperation::MethodCall { name, args, .. } => operation_result(
                    documents,
                    target,
                    &ty,
                    name,
                    args,
                    receiver,
                    offset,
                    depth + 1,
                )?,
                PostfixOperation::Index { .. } => match ty {
                    Ty::Array(inner) | Ty::Option(inner) => *inner,
                    Ty::Dict(_, value) => *value,
                    Ty::String => Ty::String,
                    _ => return None,
                },
                PostfixOperation::Field { .. } => return None,
            };
        }
        return Some(ty);
    }
    let name = receiver.split_once('(').map_or(receiver, |(name, _)| name);
    let parts = name.split('.').map(str::to_owned).collect::<Vec<_>>();
    let (index, symbol) = if parts.len() == 1 {
        resolve_unqualified(documents, target, name, offset)?
    } else {
        resolve_qualified(documents, target, &parts, offset)?
    };
    if matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
        return None;
    }
    let name = symbol
        .type_name
        .or_else(|| symbol.detail.rsplit_once(": ").map(|(_, ty)| ty.to_owned()))?;
    if let Some(semantic) = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
        && let Some((_, ty)) = semantic
            .metadata()
            .named_types
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(&name))
    {
        return Some(ty.clone());
    }
    if let Some(ty) = shape(&name) {
        return Some(ty);
    }
    let (alias_index, alias) = find_type(documents, target, index, &name)?;
    let declaration = documents[alias_index]
        .snapshot
        .source()
        .get(alias.full_span.offset()..alias.full_span.end())?;
    shape(
        declaration
            .split_once('=')
            .map_or(declaration, |(_, ty)| ty.trim().trim_end_matches(';')),
    )
}
