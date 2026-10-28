//! Source spellings for generated intrinsic declarations under the mandatory-alias contract.
//! See `docs/pascal/program-structure/units.md`.

use std::collections::{BTreeMap, HashSet};

use fpas_lexer::{Token, lex};

/// Converts metadata type paths to local names or explicitly imported alias names.
pub(super) fn source_names(owner: &str, source: &str) -> Result<String, String> {
    let (tokens, errors) = lex(source);
    if !errors.is_empty() {
        return Err(format!("invalid generated declaration tokens: {errors:?}"));
    }
    let mut references = Vec::new();
    let mut path_tokens = HashSet::new();
    let mut index = 0;
    let mut units = fpas_sema::intrinsic_std_units().to_vec();
    units.sort_by_key(|unit| std::cmp::Reverse(unit.len()));
    while index < tokens.len() {
        if index > 0 && matches!(tokens[index - 1].token, Token::Unit | Token::Dot) {
            index += 1;
            continue;
        }
        let mut matched = None;
        for unit in &units {
            let mut end = index;
            let parts = unit.split('.').collect::<Vec<_>>();
            let valid = parts.iter().enumerate().all(|(part_index, part)| {
                if part_index > 0 {
                    if !tokens
                        .get(end)
                        .is_some_and(|token| token.token == Token::Dot)
                    {
                        return false;
                    }
                    end += 1;
                }
                let same = tokens.get(end).is_some_and(|token| {
                    matches!(&token.token, Token::Ident(name) if name.eq_ignore_ascii_case(part))
                });
                end += 1;
                same
            });
            if valid
                && tokens
                    .get(end)
                    .is_some_and(|token| token.token == Token::Dot)
            {
                matched = Some((*unit, end));
                break;
            }
        }
        if let Some((unit, end)) = matched {
            path_tokens.extend(index..=end);
            references.push((unit, tokens[index].span.offset, tokens[end].span.offset + 1));
            index = end + 1;
        } else {
            index += 1;
        }
    }
    let mut reserved = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if path_tokens.contains(&index) {
                return None;
            }
            match &token.token {
                Token::Ident(name) => Some(name.to_ascii_lowercase()),
                _ => None,
            }
        })
        .collect::<HashSet<_>>();
    let mut aliases = BTreeMap::new();
    for (unit, _, _) in &references {
        if unit.eq_ignore_ascii_case(owner) || aliases.contains_key(unit) {
            continue;
        }
        let last = unit.rsplit('.').next().unwrap_or("Imported");
        let mut alias = last.to_owned();
        if reserved.contains(&alias.to_ascii_lowercase()) {
            alias = format!("Std{last}");
        }
        let base = alias.clone();
        let mut suffix = 2;
        while reserved.contains(&alias.to_ascii_lowercase()) {
            alias = format!("{base}{suffix}");
            suffix += 1;
        }
        reserved.insert(alias.to_ascii_lowercase());
        aliases.insert(*unit, alias);
    }
    let mut output = source.to_owned();
    for (unit, start, end) in references.into_iter().rev() {
        let replacement = if unit.eq_ignore_ascii_case(owner) {
            String::new()
        } else {
            format!("{}.", aliases[unit])
        };
        output.replace_range(start..end, &replacement);
    }
    if !aliases.is_empty() {
        let imports = aliases
            .iter()
            .map(|(unit, alias)| format!("{unit} as {alias}"))
            .collect::<Vec<_>>()
            .join(", ");
        let position = output.find(';').ok_or("generated unit has no header")? + 1;
        output.insert_str(position, &format!("\n\nuses {imports};"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::source_names;

    #[test]
    fn generated_signatures_use_local_types_and_nonconflicting_import_aliases() {
        let source = "unit Std.Net.Sockets;\npublic function Read(Tasks: integer; Link: Std.Net.Sockets.Connection; Token: Std.Tasks.CancellationToken): Std.Net.Sockets.Connection;\nbegin return Link; end function;\nend unit;";
        let actual = source_names("Std.Net.Sockets", source).expect("qualified declarations");
        assert!(actual.contains("uses Std.Tasks as StdTasks;"));
        assert!(
            actual.contains("Link: Connection; Token: StdTasks.CancellationToken): Connection;")
        );
        let (_, diagnostics) = fpas_parser::parse_compilation_unit(&actual);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
}
