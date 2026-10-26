//! Import organization that preserves aliases, comments, and unit initialization effects.

use std::collections::{HashMap, HashSet};

use fpas_diagnostics::SourceSpan;
use fpas_lexer::{Token, collect_comments};
use fpas_parser::{CompilationUnit, Decl, Import};

use crate::SemanticEdit;
use crate::navigation::NavigationDocument;

pub(super) fn organize(
    documents: &[NavigationDocument],
    target: usize,
) -> Option<Vec<SemanticEdit>> {
    let document = &documents[target];
    let uses = document
        .tokens
        .iter()
        .position(|token| matches!(token.token, Token::Uses))?;
    let start = document.tokens[uses].span.offset;
    let semicolon = document.tokens[uses..]
        .iter()
        .find(|token| matches!(token.token, Token::Semicolon))?;
    let end = semicolon.span.offset + semicolon.span.length;
    if collect_comments(document.snapshot.source())
        .iter()
        .any(|comment| {
            comment.span.offset < end && start < comment.span.offset + comment.span.length
        })
    {
        return None;
    }

    let mut organized = Vec::new();
    let mut sortable = Vec::new();
    let mut inert_cache = HashMap::new();
    for import in &document.uses {
        let inert = inert_unit(
            documents,
            &import.unit.parts.join("."),
            &mut HashSet::new(),
            &mut inert_cache,
        );
        let alias = import.alias.as_ref()?;
        let used = document.tokens.iter().any(|token| {
            !(start..end).contains(&token.span.offset)
                && matches!(&token.token, Token::Ident(name) if name.eq_ignore_ascii_case(&alias.name))
        });
        if inert {
            if used {
                sortable.push(import);
            }
        } else {
            // Runtime initializers and their dependency order are observable even for unused imports.
            flush_sorted(&mut sortable, &mut organized);
            organized.push(import);
        }
    }
    flush_sorted(&mut sortable, &mut organized);
    let mut compilation = document.snapshot.compilation_unit().clone();
    let imports = match &mut compilation {
        CompilationUnit::Program(program) => &mut program.uses,
        CompilationUnit::Unit(unit) => &mut unit.uses,
    };
    *imports = organized.into_iter().cloned().collect();
    let replacement = if imports.is_empty() {
        String::new()
    } else {
        let formatted = fpas_fmt::format_compilation_unit(&compilation);
        let start = formatted.find("uses ")?;
        let end = start + formatted[start..].find(';')? + 1;
        formatted[start..end].to_owned()
    };
    let replacement = if document.snapshot.source().contains("\r\n") {
        replacement.replace('\n', "\r\n")
    } else {
        replacement
    };
    if document.snapshot.source().get(start..end)? == replacement {
        return None;
    }
    Some(vec![SemanticEdit {
        span: SourceSpan::new(start, end - start, 1, 1),
        new_text: replacement,
    }])
}

fn flush_sorted<'a>(sortable: &mut Vec<&'a Import>, output: &mut Vec<&'a Import>) {
    sortable.sort_by_cached_key(|import| import.unit.parts.join(".").to_ascii_lowercase());
    output.append(sortable);
}

fn inert_unit(
    documents: &[NavigationDocument],
    owner: &str,
    visiting: &mut HashSet<String>,
    cache: &mut HashMap<String, bool>,
) -> bool {
    let key = owner.to_ascii_lowercase();
    if let Some(inert) = cache.get(&key) {
        return *inert;
    }
    if !visiting.insert(key.clone()) {
        return false;
    }
    let inert = documents
        .iter()
        .find(|document| document.owner.eq_ignore_ascii_case(owner))
        .is_some_and(|document| {
            if document.is_editor_api {
                return true;
            }
            let CompilationUnit::Unit(unit) = document.snapshot.compilation_unit() else {
                return false;
            };
            !document.snapshot.has_parse_errors()
                && !unit
                    .declarations
                    .iter()
                    .any(|declaration| matches!(declaration, Decl::Const(_) | Decl::Var(_)))
                && unit.uses.iter().all(|import| {
                    inert_unit(documents, &import.unit.parts.join("."), visiting, cache)
                })
        });
    visiting.remove(&key);
    cache.insert(key, inert);
    inert
}
