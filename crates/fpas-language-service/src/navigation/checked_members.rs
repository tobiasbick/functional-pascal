//! Declaration identity of members selected through checked expression receivers.

use super::NavigationDocument;
use crate::{DocumentSymbol, SymbolVisibility};
use fpas_sema::Ty;

pub(super) fn resolve(
    documents: &[NavigationDocument],
    selected: usize,
    token: usize,
    name: &str,
) -> Option<(usize, DocumentSymbol)> {
    let document = &documents[selected];
    let metadata = document.analysis.as_ref()?.semantic()?.metadata();
    let span = document.tokens.get(token)?.span;
    let receiver = metadata
        .member_receivers
        .get(&(span.source_id, span.offset))
        .or_else(|| {
            document.tokens.get(token.checked_sub(1)?).and_then(|dot| {
                metadata
                    .member_receivers
                    .get(&(dot.span.source_id, dot.span.offset))
            })
        })?;
    let Ty::Record(record) = receiver else {
        return None;
    };
    let qualified = if record.name.contains('.') {
        record.name.clone()
    } else {
        record.owner_unit.as_ref().map_or_else(
            || record.name.clone(),
            |owner| format!("{owner}.{}", record.name),
        )
    };
    let (owner, definition) = super::find_type(documents, selected, selected, &qualified)?;
    let member = definition.type_members().find(|member| {
        member.name.eq_ignore_ascii_case(name)
            && (owner == selected || member.visibility == SymbolVisibility::Public)
    })?;
    let mut member = member.clone();
    crate::intellisense::record_members::instantiate_checked_member(&mut member, record);
    Some((owner, member))
}
