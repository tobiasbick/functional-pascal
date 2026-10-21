use super::*;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;

fn symbol(capability: TransferCapability) -> Symbol {
    Symbol {
        ty: Ty::Error,
        mutable: true,
        kind: SymbolKind::Var,
        capability,
        constant: None,
    }
}

#[test]
fn snapshots_distinguish_shadowed_and_replaced_bindings() {
    let mut scopes = ScopeStack::new();
    scopes.define("Work", symbol(TransferCapability::Transferable));
    let outer = scopes.snapshot_capabilities();
    scopes.push_scope();
    scopes.define("Work", symbol(TransferCapability::TaskBound));
    scopes.restore_capabilities(&outer);
    assert_eq!(
        scopes.lookup("work").unwrap().capability,
        TransferCapability::TaskBound
    );
    let nested = scopes.snapshot_capabilities();
    scopes.pop_scope();
    scopes.push_scope();
    scopes.define("Work", symbol(TransferCapability::Unknown));
    scopes.restore_capabilities(&nested);
    assert_eq!(
        scopes.lookup("work").unwrap().capability,
        TransferCapability::Unknown
    );
    scopes.remove_from_current("Work");
    scopes.define("Work", symbol(TransferCapability::Transferable));
    scopes.restore_capabilities(&nested);
    assert_eq!(
        scopes.lookup("work").unwrap().capability,
        TransferCapability::Transferable
    );
    scopes.pop_scope();
    assert_eq!(scopes.snapshot_capabilities(), outer);
}

#[test]
fn joins_use_binding_identity_and_are_independent_of_arm_order() {
    let mut scopes = ScopeStack::new();
    scopes.define("Work", symbol(TransferCapability::Transferable));
    let free = scopes.snapshot_capabilities();
    scopes.lookup_mut("work").unwrap().capability = TransferCapability::Unknown;
    let unknown = scopes.snapshot_capabilities();
    scopes.lookup_mut("work").unwrap().capability = TransferCapability::TaskBound;
    let local = scopes.snapshot_capabilities();
    for paths in [
        vec![free.clone(), unknown.clone(), local.clone()],
        vec![local, unknown.clone(), free.clone()],
    ] {
        scopes.join_capability_snapshots(&paths);
        assert_eq!(
            scopes.lookup("work").unwrap().capability,
            TransferCapability::TaskBound
        );
    }
    scopes.join_capability_snapshots(&[unknown, free.clone()]);
    assert_eq!(
        scopes.lookup("work").unwrap().capability,
        TransferCapability::Unknown
    );
    scopes.restore_capabilities(&free);
    assert_eq!(
        scopes.lookup("work").unwrap().capability,
        TransferCapability::Transferable
    );
}
