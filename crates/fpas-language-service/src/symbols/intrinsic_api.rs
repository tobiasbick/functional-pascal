//! Registry declarations for keyword enum members and documented enum aliases.

use fpas_diagnostics::SourceSpan;
use fpas_sema::{IntrinsicStdSymbolKind, Ty, intrinsic_std_symbols};

use super::{CallableSignature, DocumentSymbol, DocumentSymbols, SymbolKind, SymbolVisibility};
use crate::DocumentSnapshot;

pub(super) fn add_registry_symbols(snapshot: &DocumentSnapshot, symbols: &mut DocumentSymbols) {
    let Some(root) = symbols.entries().first() else {
        return;
    };
    let scope_span = root.full_span;
    let unit = symbols.owner().to_owned();
    for symbol in intrinsic_std_symbols(&unit) {
        if symbol.kind != IntrinsicStdSymbolKind::Type {
            continue;
        }
        let Ty::Enum(enum_ty) = symbol.ty else {
            continue;
        };
        let Some(parent) = find_symbol_mut(symbols.entries_mut(), &symbol.qualified_name) else {
            continue;
        };
        for variant in &enum_ty.variants {
            if parent
                .children
                .iter()
                .any(|child| child.name.eq_ignore_ascii_case(&variant.name))
            {
                continue;
            }
            let Some((full_span, selection_span)) = documented_variant_spans(
                snapshot,
                parent.full_span,
                &variant.name,
                scope_span.source_id(),
            ) else {
                continue;
            };
            parent.children.push(DocumentSymbol {
                name: variant.name.clone(),
                qualified_name: format!("{}.{}", symbol.qualified_name, variant.name),
                kind: SymbolKind::EnumMember,
                full_span,
                selection_span,
                scope_span,
                visible_from: 0,
                visibility: SymbolVisibility::Public,
                type_name: Some(symbol.qualified_name.clone()),
                detail: format!("enum member {}", variant.name),
                callable: (!variant.fields.is_empty()).then(|| {
                    let parameters = variant
                        .fields
                        .iter()
                        .map(|(field, ty)| format!("{field}: {ty}"))
                        .collect::<Vec<_>>();
                    CallableSignature {
                        label: format!(
                            "{}({}): {}",
                            variant.name,
                            parameters.join("; "),
                            symbol.qualified_name
                        ),
                        parameters,
                    }
                }),
                children: Vec::new(),
            });
        }
        parent
            .children
            .sort_by_key(|child| child.selection_span.offset());
    }
}

fn documented_variant_spans(
    snapshot: &DocumentSnapshot,
    parent_span: SourceSpan,
    name: &str,
    source_id: u32,
) -> Option<(SourceSpan, SourceSpan)> {
    let source = snapshot.source();
    // Alias members have documentation blocks immediately before their type alias.
    let mut start = source[..parent_span.offset()]
        .rfind('\n')
        .map_or(0, |p| p + 1);
    while start > 0 {
        let previous = source[..start - 1].rfind('\n').map_or(0, |p| p + 1);
        let line = source[previous..start].trim();
        if !line.is_empty() && !line.starts_with("//") {
            break;
        }
        start = previous;
    }
    let end = parent_span.end().min(source.len());
    let marker = format!("// `{name}` enum member.");
    let comment_offset = start + source.get(start..end)?.find(&marker)?;
    let selection_offset = comment_offset + marker.find(name)?;
    let after_summary = comment_offset + marker.len();
    let remaining = source.get(after_summary..end)?;
    let mut line_offset = after_summary;
    let mut declaration_offset = end;
    for (index, line) in remaining.split_inclusive('\n').enumerate() {
        let trimmed = line.trim();
        if index != 0
            && (trimmed.starts_with("// `") && trimmed.ends_with("enum member.")
                || !trimmed.starts_with("//"))
        {
            declaration_offset = line_offset;
            break;
        }
        line_offset += line.len();
    }
    Some((
        source_span(snapshot, declaration_offset, 0, source_id)?,
        source_span(snapshot, selection_offset, name.len(), source_id)?,
    ))
}

fn source_span(
    snapshot: &DocumentSnapshot,
    offset: usize,
    length: usize,
    source_id: u32,
) -> Option<SourceSpan> {
    let position = snapshot.line_index().position(offset)?;
    Some(SourceSpan::new_with_source(
        offset,
        length,
        u32::try_from(position.line + 1).unwrap_or(u32::MAX),
        u32::try_from(position.byte_column + 1).unwrap_or(u32::MAX),
        source_id,
    ))
}

fn find_symbol_mut<'a>(
    symbols: &'a mut [DocumentSymbol],
    qualified_name: &str,
) -> Option<&'a mut DocumentSymbol> {
    for symbol in symbols {
        if symbol.qualified_name.eq_ignore_ascii_case(qualified_name) {
            return Some(symbol);
        }
        if let Some(found) = find_symbol_mut(&mut symbol.children, qualified_name) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use crate::{DocumentSnapshot, SourceVersion};

    use super::DocumentSymbols;

    #[test]
    fn editor_snapshot_adds_missing_enum_member_from_registry_and_markdown() {
        let source = Arc::<str>::from(
            "unit Std.Json;\n\npublic type\n  JsonValue = enum\n    // `ArrayValue` enum member.\n    // `Object` enum member.\n    Object(Fields: dict of string to JsonValue);\n  end enum;\nend unit;\n",
        );
        let snapshot = DocumentSnapshot::parse(
            std::path::Path::new("Json.fpas"),
            SourceVersion::Disk(0),
            0,
            source,
        );

        let symbols = DocumentSymbols::from_editor_snapshot(&snapshot);
        let json_value = &symbols.entries()[0].children[0];
        let array = json_value
            .children
            .iter()
            .find(|child| child.qualified_name == "Std.Json.JsonValue.ArrayValue")
            .expect("Array registry declaration");

        assert_eq!(
            &snapshot.source()[array.selection_span.offset()..array.selection_span.end()],
            "ArrayValue"
        );
    }

    #[test]
    fn enum_alias_members_keep_their_own_documentation_and_shared_identity() {
        let source = Arc::<str>::from(
            "unit Std.Mcp;\n// `InvalidArgument` enum member.\n//\n// Parameters:\n// - `Argument`: Rejected input.\n\n// `Busy` enum member.\n\n// Shared MCP error alias.\npublic type McpError = Std.Mcp.Types.McpError;\nend unit;\n",
        );
        let snapshot = DocumentSnapshot::parse(
            std::path::Path::new("Mcp.fpas"),
            SourceVersion::Disk(0),
            0,
            source,
        );
        let symbols = DocumentSymbols::from_editor_snapshot(&snapshot);
        let alias = &symbols.entries()[0].children[0];
        let invalid = alias
            .children
            .iter()
            .find(|v| v.name == "InvalidArgument")
            .expect("aliased constructor");
        let busy = alias
            .children
            .iter()
            .find(|v| v.name == "Busy")
            .expect("aliased member");
        assert_eq!(invalid.qualified_name, "Std.Mcp.McpError.InvalidArgument");
        assert_eq!(
            invalid.callable.as_ref().expect("constructor").parameters,
            ["Argument: string"]
        );
        let documentation = crate::documentation::preceding_documentation(
            snapshot.source(),
            invalid.full_span.offset(),
        )
        .expect("constructor documentation");
        assert!(documentation.contains("- `Argument`: Rejected input."));
        assert!(!documentation.contains("Busy"));
        assert_eq!(
            crate::documentation::preceding_documentation(
                snapshot.source(),
                busy.full_span.offset()
            )
            .as_deref(),
            Some("`Busy` enum member.")
        );
        assert_eq!(
            crate::documentation::preceding_documentation(
                snapshot.source(),
                alias.full_span.offset()
            )
            .as_deref(),
            Some("Shared MCP error alias.")
        );
    }
}
