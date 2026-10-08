use super::{ScopeStack, Symbol, SymbolKind};
use crate::types::Ty;

#[test]
fn pop_scope_never_removes_root_scope() {
    let mut stack = ScopeStack::new();
    stack.push_scope();
    stack.pop_scope();
    stack.pop_scope();

    assert!(
        stack.define(
            "x",
            Symbol {
                constant: None,
                ty: Ty::Integer,
                mutable: false,
                kind: SymbolKind::Var,
                task_bound: false,
            }
        ),
        "root scope must remain usable after extra pop_scope"
    );
}

#[test]
fn remove_from_current_drops_only_the_innermost_symbol() {
    let mut stack = ScopeStack::new();
    stack.push_scope();
    assert!(stack.define(
        "offset",
        Symbol {
            constant: None,
            ty: Ty::Integer,
            mutable: false,
            kind: SymbolKind::Var,
            task_bound: false,
        }
    ));
    assert!(stack.remove_from_current("Offset"));
    assert!(stack.lookup_current("offset").is_none());
    assert!(!stack.remove_from_current("offset"));
}
