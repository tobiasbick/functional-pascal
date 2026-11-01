//! Snapshot-owned source relationships, independent of editor and command-line protocols.

use crate::{DeclarationOrigin, DocumentSnapshot};
use fpas_diagnostics::SourceSpan;
use std::{collections::BTreeSet, sync::Arc};

/// The source owner of a body or executable declaration expression.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CallableKind {
    /// A named function, procedure or record method.
    Routine,
    /// An anonymous function or procedure.
    Closure,
    /// The executable entry and top-level initializers of a program.
    ProgramEntry,
    /// Ordered unit-level initializers.
    UnitInitialization,
    /// A record field's default expression in its declaration environment.
    RecordDefault,
    /// A constant or subrange evaluated during structural type collection.
    StaticBound,
    /// An intrinsic or fixed native operation, without a source body.
    Builtin,
}

/// A physical declaration identity, or a compiler-owned native identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallableIdentity {
    /// Canonical physical source, absent for native operations.
    pub file: Option<String>,
    /// Full declaration start in UTF-8 bytes, absent for native operations.
    pub start: Option<usize>,
    /// Source owner category.
    pub kind: CallableKind,
    /// Case-preserving qualified source name, or canonical native operation name.
    pub qualified_name: String,
}

/// An immutable callable declaration and its exact source boundaries.
#[derive(Clone)]
pub struct SourceCallable {
    /// Declaration identity, shared by all its call sites.
    pub identity: CallableIdentity,
    /// Declaration source; never an API stub pretending to be a body.
    pub snapshot: Option<Arc<DocumentSnapshot>>,
    /// Full body or declaration-expression extent.
    pub span: Option<SourceSpan>,
    /// Tight identifier, or anonymous owner's opening source range.
    pub selection_span: Option<SourceSpan>,
    /// Source or compiler origin.
    pub origin: DeclarationOrigin,
    /// Whether this owner declares or inherits type parameters.
    pub generic: bool,
}

/// The meaning of a source relationship; a value passage does not imply execution.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CallRelationKind {
    /// A call with an exact compiler-proven target.
    DirectCall,
    /// A callable declaration used as a value outside a call argument.
    CallableValue,
    /// A callable value passed to another call.
    CallbackArgument,
    /// A call through a value without proof of its runtime target.
    DynamicCall,
}

/// How the written call is invoked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationKind {
    /// Ordinary synchronous invocation.
    Ordinary,
    /// The call directly wrapped by `go`.
    Spawn,
}

/// One written argument and its checked position in the callee's signature.
#[derive(Clone)]
pub struct CallArgument {
    /// Complete argument, including any name and `var` marker.
    pub span: SourceSpan,
    /// Value expression, excluding a named argument's label.
    pub value_span: SourceSpan,
    /// Written name, absent for positional arguments.
    pub name: Option<String>,
    /// Declaration parameter index, including an implicit receiver when present.
    pub parameter_index: Option<usize>,
    /// True when the argument is explicitly passed by reference.
    pub by_reference: bool,
}

/// One physical source use in one checked consumer context.
#[derive(Clone)]
pub struct CallRelation {
    /// Direct call, callable value, callback passage or dynamic call.
    pub kind: CallRelationKind,
    /// Containing body; nested routines and closures have independent owners.
    pub containing: Arc<SourceCallable>,
    /// Exact target when proven by the compiler.
    pub target: Option<Arc<SourceCallable>>,
    /// Immutable source of the use.
    pub snapshot: Arc<DocumentSnapshot>,
    /// Complete written call or value-use range.
    pub span: SourceSpan,
    /// Checked consumer manifest, or loose source context.
    pub context: String,
    /// Ordinary or spawned call.
    pub invocation: InvocationKind,
    /// Arguments in written evaluation order.
    pub arguments: Vec<CallArgument>,
    /// False when the checked written-to-parameter mapping is unavailable.
    pub arguments_complete: bool,
    /// Whether the call supplies an implicit record/native receiver.
    pub implicit_receiver: bool,
    /// Callee receiving a callback, when its own target is proven.
    pub receiving: Option<Arc<SourceCallable>>,
    /// Checked parameter receiving the callback.
    pub receiving_parameter: Option<usize>,
    /// Why the exact runtime target or checked mapping is unavailable.
    pub unresolved_reason: Option<&'static str>,
}

/// Independent bounded collection and semantic coverage for a relationship category.
pub struct CallCollection {
    /// Deterministically ordered retained uses.
    pub items: Vec<CallRelation>,
    /// Exact total after complete eligible traversal, otherwise unknown.
    pub known_total: Option<usize>,
    /// Whether all known matching items fit in the response.
    pub items_complete: bool,
}

/// Direct source edges and explicit limits on what they prove about runtime calls.
pub struct CallRelations {
    /// Proven calls, including compiler-owned native operations.
    pub direct_calls: CallCollection,
    /// Callable declaration values outside argument positions.
    pub callable_values: CallCollection,
    /// Callback arguments, without inferred execution edges.
    pub callback_arguments: CallCollection,
    /// Relevant unresolved call sites.
    pub dynamic_calls: CallCollection,
    /// Whether all eligible direct edges were analyzed, independently of response limits.
    pub direct_edges_complete: bool,
    /// Whether every inventoried relevant invocation has an exact target.
    pub runtime_targets_complete: bool,
    /// Semantic exclusions, unavailable mappings and output limits.
    pub reasons: BTreeSet<&'static str>,
}
