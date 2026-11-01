//! Call relations retain compiler-selected declarations, containing owners and uncertainty.
#![allow(
    clippy::expect_used,
    reason = "call fixtures identify each required checked fact"
)]

use crate::support::TempDirectory;
use fpas_language_service::LanguageService;
use fpas_language_service::{CallableKind, CancellationToken, SourceQueryOperation, SourceSession};
use fpas_sema::SourceBindingKind;

#[test]
fn a_callable_reference_reconstructs_the_same_scope_with_unchanged_open_buffers() {
    let temp = TempDirectory::new("source-call-overlay-reference");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; function Leaf(): integer; begin return 1; end function; begin discard Leaf(); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&path, 1, source)
        .expect("open buffer");
    let overlays = service.documents().open_snapshots();
    let cancel = CancellationToken::new();
    let mut first =
        SourceSession::load(&path, None, &overlays, &cancel).expect("inferred prepare scope");
    let owner = first
        .callable_at(&path, source.find("Leaf").expect("name"), &cancel)
        .expect("prepare")
        .expect("owner");
    let reference = first.callable_reference(&owner);
    let snapshot = first.snapshot().clone();
    let context = first.context().clone();
    drop(first);
    let mut second = SourceSession::load(
        std::path::Path::new(&reference.context.input),
        None,
        &overlays,
        &cancel,
    )
    .expect("reconstructed scope");
    assert_eq!(second.context(), &context);
    assert_eq!(second.snapshot().inventory, snapshot.inventory);
    assert_eq!(second.snapshot().id, snapshot.id);
    assert_eq!(
        second
            .query_callable(&reference, true, 100, &cancel)
            .expect("fresh hierarchy")
            .direct_calls
            .known_total,
        Some(1)
    );
}

#[test]
fn retained_static_constructors_are_not_mistaken_for_calls_and_generic_defaults_are_marked() {
    let temp = TempDirectory::new("source-call-retained");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; function Seed(): integer; begin return 7; end function; type Settings = record Lower: integer; Upper: integer := 10; end record; const Limits: Settings := Settings(Lower := 0); type Bounds = Limits.Lower..Limits.Upper; type Generic of T = record Value: T; Initial: integer := Seed(); end record; begin const A := Generic(Value := 1); const B := Generic(Value := 'text'); end.";
    let path = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut service = LanguageService::load(&manifest);
    let analysis = service.analyze_document(&path).expect("static analysis");
    assert!(
        !analysis
            .semantic()
            .expect("checked metadata")
            .metadata()
            .static_bound_declarations
            .is_empty(),
        "fixture exercises retained allocations"
    );
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Seed", SourceQueryOperation::Callers, 100, &cancel)
        .expect("default call");
    assert!(
        result.reasons.is_empty(),
        "{:?}; {:?}",
        result.reasons,
        result.diagnostics
    );
    let calls = result.calls.expect("relations");
    assert_eq!(calls.direct_calls.known_total, Some(1));
    assert!(calls.direct_calls.items[0].containing.generic);
    assert_eq!(
        calls.direct_calls.items[0].containing.identity.kind,
        CallableKind::RecordDefault
    );
    assert!(
        calls.dynamic_calls.items.is_empty(),
        "retained constructors have checked classification"
    );
}

#[test]
fn methods_native_calls_constructors_generic_identity_and_spawns_are_classified_from_checked_facts()
{
    let temp = TempDirectory::new("source-call-forms");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; type Box = record Value: integer; Callback: function(): integer; function Add(Self: Box; Delta: integer): integer; begin return Self.Value + Delta; end function; end record; function Leaf(): integer; begin return 1; end function; function Echo<T>(Value: T): T; begin return Value; end function; function Make(): Box; begin return Box(Value := 1, Callback := Leaf); end function; begin const Item := Make(); discard Item.Add(Delta := 2); discard Make().Add(3); const Bound := Item.Add; discard Bound(4); discard Item.Callback(); discard 'abc'.Length(); discard Echo(5); scope const Job := go Leaf(); end scope; end.";
    let path = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_position(
            &path,
            source.find("Main").expect("entry"),
            SourceQueryOperation::Callees,
            100,
            &cancel,
        )
        .expect("entry callees");
    assert_eq!(
        result.reasons,
        ["dynamic_targets"].into_iter().collect(),
        "{:?}",
        result.diagnostics
    );
    let calls = result.calls.expect("relations");
    let methods = calls
        .direct_calls
        .items
        .iter()
        .filter(|r| {
            r.target
                .as_ref()
                .is_some_and(|t| t.identity.qualified_name == "Main.Box.Add")
        })
        .collect::<Vec<_>>();
    assert_eq!(methods.len(), 2);
    for method in methods {
        assert!(method.implicit_receiver);
        assert!(method.arguments_complete);
        assert_eq!(method.arguments[0].parameter_index, Some(1));
    }
    assert!(calls.direct_calls.items.iter().any(|r| {
        r.target.as_ref().is_some_and(|t| {
            t.identity.kind == CallableKind::Builtin && t.identity.qualified_name == "string.Length"
        })
    }));
    assert!(calls.direct_calls.items.iter().any(|r| {
        r.target
            .as_ref()
            .is_some_and(|t| t.identity.qualified_name == "Main.Echo" && t.generic)
    }));
    let spawn = calls
        .direct_calls
        .items
        .iter()
        .find(|r| r.invocation == fpas_language_service::InvocationKind::Spawn)
        .expect("spawn");
    assert_eq!(
        spawn
            .target
            .as_ref()
            .expect("spawn target")
            .identity
            .qualified_name,
        "Main.Leaf"
    );
    assert_eq!(calls.dynamic_calls.known_total, Some(2));
    let bound = &calls.callable_values.items[0];
    assert_eq!(
        bound
            .target
            .as_ref()
            .expect("bound method")
            .identity
            .qualified_name,
        "Main.Box.Add"
    );
    let make = session
        .query_symbol("Make", SourceQueryOperation::Callees, 100, &cancel)
        .expect("make")
        .calls
        .expect("relations");
    assert!(
        make.direct_calls.items.is_empty(),
        "record construction is not a routine call"
    );
    assert!(make.dynamic_calls.items.is_empty());
    assert_eq!(
        make.callback_arguments.items[0]
            .target
            .as_ref()
            .expect("stored callback")
            .identity
            .qualified_name,
        "Main.Leaf"
    );
}

#[test]
fn bounded_call_queries_preserve_semantic_coverage_empty_bodies_and_snapshot_bound_anonymous_owners()
 {
    let temp = TempDirectory::new("source-call-bounds");
    let source = "program Main; function Leaf(): integer; begin return 1; end function; function Parent(): integer; begin const F := function(): integer begin return Leaf(); end function; discard Leaf(); discard Leaf(); return Leaf(); end function; begin discard Parent(); end.";
    let path = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&path, None, &[], &cancel).expect("loose scope");
    let result = session
        .query_position(
            &path,
            source.find("Parent()").expect("parent"),
            SourceQueryOperation::Callees,
            1,
            &cancel,
        )
        .expect("bounded calls");
    assert_eq!(
        result.reasons,
        ["result_limit"].into_iter().collect(),
        "{:?}",
        result.diagnostics
    );
    let calls = result.calls.expect("relations");
    assert_eq!(calls.direct_calls.known_total, Some(3));
    assert_eq!(calls.direct_calls.items.len(), 1);
    assert!(calls.direct_edges_complete && calls.runtime_targets_complete);
    let empty = session
        .query_position(
            &path,
            source.find("Leaf()").expect("leaf"),
            SourceQueryOperation::Callees,
            100,
            &cancel,
        )
        .expect("empty body")
        .calls
        .expect("relations");
    assert_eq!(empty.direct_calls.known_total, Some(0));
    assert!(empty.direct_edges_complete && empty.runtime_targets_complete);
    let closure = session
        .callable_at(
            &path,
            source.find("begin return Leaf").expect("closure body"),
            &cancel,
        )
        .expect("prepare closure")
        .expect("closure owner");
    assert_eq!(closure.identity.kind, CallableKind::Closure);
    let reference = session.callable_reference(&closure);
    let outgoing = session
        .query_callable(&reference, false, 100, &cancel)
        .expect("closure outgoing");
    assert_eq!(outgoing.direct_calls.known_total, Some(1));
    drop(session);
    temp.write("main.fpas", &source.replace("return 1", "return 2"));
    let mut changed = SourceSession::load(&path, None, &[], &cancel).expect("changed scope");
    assert_eq!(
        changed
            .query_callable(&reference, false, 100, &cancel)
            .err()
            .expect("stale owner")
            .code,
        "stale_target"
    );
}

#[test]
fn direct_relations_preserve_alias_identity_named_arguments_and_dynamic_uncertainty() {
    let temp = TempDirectory::new("source-call-relations");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write("math.fpas", "unit Demo.Math; public function Compute(Left: integer; Right: integer): integer; begin return Left + Right; end function; end unit;");
    let source = "program Main; uses Demo.Math as M; function Compute(): integer; begin return 3; end function; function Apply(F: function(Value: integer): integer): integer; begin return F(9); end function; function Echo(Value: integer): integer; begin return Value; end function; begin const F := M.Compute; discard M.Compute(Right := 2, Left := 1); discard F(3, 4); discard Compute(); discard Apply(Echo); end.";
    let main = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol(
            "Demo.Math.Compute",
            SourceQueryOperation::Callers,
            100,
            &cancel,
        )
        .expect("callers");
    assert_eq!(
        result.reasons,
        ["dynamic_targets"].into_iter().collect(),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.target.as_ref().expect("target").target_ref.is_some(),
        "dynamic runtime targets do not invalidate declaration identity"
    );
    let calls = result.calls.expect("relations");
    assert!(calls.direct_edges_complete);
    assert!(!calls.runtime_targets_complete);
    assert_eq!(calls.direct_calls.known_total, Some(1));
    let call = &calls.direct_calls.items[0];
    assert_eq!(
        &source[call.span.offset()..call.span.end()],
        "M.Compute(Right := 2, Left := 1)"
    );
    assert_eq!(call.containing.identity.kind, CallableKind::ProgramEntry);
    assert!(call.arguments_complete);
    assert_eq!(
        call.arguments
            .iter()
            .map(|a| a.parameter_index)
            .collect::<Vec<_>>(),
        [Some(1), Some(0)]
    );
    assert_eq!(calls.callable_values.known_total, Some(1));
    assert_eq!(
        calls.dynamic_calls.known_total,
        Some(2),
        "unknown calls in the checked scope remain relevant to callers"
    );
    let outgoing = session
        .query_position(
            &main,
            source.find("Apply(F").expect("Apply"),
            SourceQueryOperation::Callees,
            100,
            &cancel,
        )
        .expect("callees");
    let calls = outgoing.calls.expect("relations");
    assert!(calls.direct_calls.items.is_empty());
    assert_eq!(calls.dynamic_calls.known_total, Some(1));
    assert_eq!(
        calls.dynamic_calls.items[0]
            .containing
            .identity
            .qualified_name,
        "Main.Apply"
    );
    let callbacks = session
        .query_symbol("Echo", SourceQueryOperation::Callers, 100, &cancel)
        .expect("callback uses")
        .calls
        .expect("relations");
    assert!(callbacks.direct_calls.items.is_empty());
    let callback = &callbacks.callback_arguments.items[0];
    assert_eq!(
        callback
            .receiving
            .as_ref()
            .expect("receiving call")
            .identity
            .qualified_name,
        "Main.Apply"
    );
    assert_eq!(callback.receiving_parameter, Some(0));
}

#[test]
fn retained_defaults_nested_routines_and_closures_keep_their_own_source_owners() {
    let temp = TempDirectory::new("source-call-owners");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; function Seed(): integer; begin return 7; end function; type Box = record Value: integer := Seed(); end record; function Outer(): integer; function Nested(): integer; begin return Seed(); end function; begin const F := function(): integer begin return Seed(); end function; discard F(); return Nested(); end function; begin const First := Box(); const Second := Box(); discard Outer(); end.";
    let path = temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Seed", SourceQueryOperation::Callers, 100, &cancel)
        .expect("callers");
    assert_eq!(
        result.reasons,
        ["dynamic_targets"].into_iter().collect(),
        "{:?}",
        result.diagnostics
    );
    let calls = result.calls.expect("relations");
    assert_eq!(calls.direct_calls.known_total, Some(3));
    assert_eq!(
        calls
            .direct_calls
            .items
            .iter()
            .map(|c| c.containing.identity.kind)
            .collect::<Vec<_>>(),
        [
            CallableKind::RecordDefault,
            CallableKind::Routine,
            CallableKind::Closure
        ]
    );
    let outer = session
        .query_position(
            &path,
            source.find("Outer()").expect("outer"),
            SourceQueryOperation::Callees,
            100,
            &cancel,
        )
        .expect("outer callees")
        .calls
        .expect("relations");
    assert_eq!(outer.direct_calls.known_total, Some(1));
    assert_eq!(
        outer.direct_calls.items[0]
            .target
            .as_ref()
            .expect("nested target")
            .identity
            .qualified_name,
        "Main.Outer.Nested"
    );
    assert_eq!(outer.dynamic_calls.known_total, Some(1));
    assert_eq!(outer.callable_values.known_total, Some(1));
}

#[test]
fn checked_call_bindings_distinguish_imported_routines_local_routines_and_callable_values() {
    let temp = TempDirectory::new("source-call-bindings");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write("math.fpas", "unit Demo.Math; public function Compute(Value: integer): integer; begin return Value; end function; end unit;");
    let source = "program Main; uses Demo.Math as M; function Compute(): integer; begin return 3; end function; begin const F := M.Compute; discard M.Compute(1); discard F(2); discard Compute(); end.";
    let main = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let analysis = service.analyze_document(&main).expect("checked source");
    assert!(
        !analysis.diagnostics().iter().any(|d| d.is_error()),
        "{:?}",
        analysis.diagnostics()
    );
    let bindings = &analysis
        .semantic()
        .expect("checked bindings")
        .metadata()
        .source_bindings;
    let binding = |needle| {
        bindings
            .iter()
            .find(|((_, start), _)| *start == source.find(needle).expect("written site"))
            .map(|(_, b)| b)
            .expect("resolved binding")
    };
    let imported = binding("M.Compute(1)");
    assert_eq!(imported.kind, SourceBindingKind::Routine);
    assert_eq!(imported.name, "demo.math.compute");
    assert_eq!(imported.parts, 2);
    assert!(imported.declaration.is_none());
    let value = binding("F(2)");
    assert_eq!(value.kind, SourceBindingKind::Value);
    assert!(value.declaration.is_some());
    let local = bindings
        .iter()
        .find(|((_, start), _)| *start == source.rfind("Compute()").expect("local call"))
        .map(|(_, b)| b)
        .expect("local binding");
    assert_eq!(local.kind, SourceBindingKind::Routine);
    assert_eq!(
        local.declaration.expect("local declaration").1,
        source.find("function Compute").expect("declaration")
    );
}
