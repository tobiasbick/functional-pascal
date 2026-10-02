//! Explicit aliases for type references in generated intrinsic APIs.

use fpas_lexer::Token;
use std::collections::{BTreeMap, HashSet};

/// Qualify external type names through explicit imports and keep own type names local.
pub(super) fn qualify(owner: &str, source: &str) -> String {
    let tokens = fpas_lexer::lex(source).0;
    let header_end = tokens
        .iter()
        .find(|token| token.token == Token::Semicolon)
        .expect("unit header")
        .span
        .end_offset()
        .expect("header span");
    let mut used_names = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.span.offset < header_end || index > 0 && tokens[index - 1].token == Token::Dot
            {
                return None;
            }
            if let Token::Ident(name) = &token.token {
                Some(name.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    let mut aliases = BTreeMap::<String, String>::new();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.span.offset < header_end
            || !matches!(&token.token, Token::Ident(name) if name.eq_ignore_ascii_case("Std"))
        {
            continue;
        }
        let mut parts = Vec::new();
        let mut part = index;
        while let Some(Token::Ident(name)) = tokens.get(part).map(|token| &token.token) {
            parts.push(name.as_str());
            if tokens
                .get(part + 1)
                .is_none_or(|token| token.token != Token::Dot)
            {
                break;
            }
            part += 2;
        }
        let name = parts.join(".");
        let Some(unit) = fpas_sema::intrinsic_std_units()
            .iter()
            .find(|unit| name.starts_with(&format!("{unit}.")))
        else {
            continue;
        };
        let replacement = if unit.eq_ignore_ascii_case(owner) {
            String::new()
        } else {
            aliases
                .entry((*unit).to_owned())
                .or_insert_with(|| {
                    let base = unit.rsplit('.').next().expect("unit suffix");
                    let mut alias = base.to_owned();
                    let mut suffix = 2;
                    while used_names.contains(&alias.to_ascii_lowercase()) {
                        alias = format!("{base}{suffix}");
                        suffix += 1;
                    }
                    used_names.insert(alias.to_ascii_lowercase());
                    alias
                })
                .clone()
                + "."
        };
        edits.push((
            token.span.offset,
            token.span.offset + unit.len() + 1,
            replacement,
        ));
    }
    let mut output = source.to_owned();
    for (start, end, replacement) in edits.into_iter().rev() {
        output.replace_range(start..end, &replacement);
    }
    let imports = aliases
        .into_iter()
        .map(|(unit, alias)| format!("\nuses {unit} as {alias};"))
        .collect::<String>();
    output.insert_str(header_end, &imports);
    output
}
