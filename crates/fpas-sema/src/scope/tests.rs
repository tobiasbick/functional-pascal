use super::{ScopeStack, Symbol, SymbolKind};
use crate::types::Ty;

#[test]
fn same_span_value_declarations_retain_independent_source_names_and_types() {
    let mut stack = ScopeStack::new();
    stack.push_scope();
    let span = fpas_lexer::Span {
        offset: 20,
        length: 40,
        line: 3,
        column: 1,
        source_id: 7,
    };
    for (name, ty) in [("Amount", Ty::Integer), ("Enabled", Ty::Boolean)] {
        assert!(stack.define_with_declaration(
            name,
            Symbol {
                constant: None,
                ty,
                mutable: false,
                kind: SymbolKind::Const,
                capability: crate::TransferCapability::Transferable
            },
            span
        ));
    }
    stack.pop_scope();
    let types = stack.value_declaration_types();
    assert_eq!(types.get(&(7, 20, "amount".into())), Some(&Ty::Integer));
    assert_eq!(types.get(&(7, 20, "enabled".into())), Some(&Ty::Boolean));
    assert!(!types.contains_key(&(7, 20, "$temporary".into())));
}

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
                capability: crate::TransferCapability::Transferable,
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
            capability: crate::TransferCapability::Transferable,
        }
    ));
    assert!(stack.remove_from_current("Offset"));
    assert!(
        !stack
            .scopes
            .last()
            .expect("nested scope")
            .symbols
            .contains_key("offset")
    );
    assert!(!stack.remove_from_current("offset"));
}
