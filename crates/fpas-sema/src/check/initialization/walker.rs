//! Availability of declaration identities while evaluating bodies and initializers.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::collection::{Model, Owner, Routine};
use super::{BindingId, identity};
use crate::check::Checker;
use crate::scope::SymbolKind;
use fpas_lexer::Span;
use fpas_parser::{Decl, Designator, Expr, FuncBody};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Possible callable origin carried by an expression or stored aggregate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Origin {
    Routine(Owner),
    Imported(BindingId),
    BoundMethod(usize, BindingId),
    Parameter(usize),
    Binding(BindingId),
    Unknown,
}
/// Conservative union of the callable origins reachable through a value.
pub(super) type Value = BTreeSet<Origin>;

/// Direct and transitive dependencies of a body at the recursive fixed point.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Summary {
    pub dependencies: BTreeSet<BindingId>,
    pub called_parameters: BTreeSet<usize>,
    pub called_bindings: BTreeSet<BindingId>,
    pub started_parameters: BTreeSet<usize>,
    pub started_bindings: BTreeSet<BindingId>,
    pub mutable_globals: BTreeSet<String>,
    pub unknown_worker: bool,
    pub writes: BTreeSet<BindingId>,
    pub returned: Value,
    pub unknown: bool,
}

/// Unsafe use reported at the actual source call, capture, or expression.
pub(super) struct Failure {
    pub span: Span,
    pub name: String,
}

/// Evaluate one lexical owner's source in initialization order.
pub(super) struct Walker<'c, 'a> {
    pub checker: &'c Checker,
    pub model: &'c Model<'a>,
    pub summaries: &'c HashMap<Owner, Summary>,
    pub owner: Owner,
    pub ready: HashSet<BindingId>,
    pub values: HashMap<BindingId, Value>,
    pub resolving: HashSet<BindingId>,
    pub invoking: BTreeSet<Origin>,
    pub summary: Summary,
    pub errors: Vec<Failure>,
    pub task_errors: Vec<Failure>,
    pub worker: bool,
    pub worker_start: Option<usize>,
    pub validate: bool,
}

impl<'c, 'a> Walker<'c, 'a> {
    /// Start summary inference or final use-site validation for one owner.
    pub fn new(
        checker: &'c Checker,
        model: &'c Model<'a>,
        summaries: &'c HashMap<Owner, Summary>,
        owner: Owner,
        validate: bool,
    ) -> Self {
        Self {
            checker,
            model,
            summaries,
            owner,
            ready: HashSet::new(),
            values: HashMap::new(),
            resolving: HashSet::new(),
            invoking: BTreeSet::new(),
            summary: Summary::default(),
            errors: Vec::new(),
            task_errors: Vec::new(),
            worker: false,
            worker_start: None,
            validate,
        }
    }

    /// Evaluate clauses and the body with initialized symbolic parameters.
    pub fn routine(&mut self, routine: &Routine<'a>) {
        for (index, param) in routine.params.iter().enumerate() {
            self.ready.insert(identity(param.span));
            self.values.insert(
                identity(param.span),
                [Origin::Parameter(index)].into_iter().collect(),
            );
        }
        for clause in routine.requires {
            self.expr(&clause.condition);
        }
        let FuncBody::Block { nested, stmts } = routine.body;
        self.initializers(nested);
        self.statements(stmts);
        for clause in routine.ensures {
            self.expr(&clause.condition);
        }
    }

    /// Complete each declaration only after its written initializer has been checked.
    pub fn initializers(&mut self, declarations: &'a [Decl]) {
        for declaration in declarations {
            let (value, span) = match declaration {
                Decl::Const(value) => (&value.value, value.span),
                Decl::Var(value) => (&value.value, value.span),
                _ => continue,
            };
            let origins = self.expr(value);
            self.values.insert(identity(span), origins);
            self.ready.insert(identity(span));
        }
    }

    /// Validate own bindings or propagate an enclosing dependency to the caller.
    pub fn dependency(&mut self, binding: BindingId, span: Span, route: &str) {
        let Some(info) = self.model.bindings.get(&binding) else {
            return;
        };
        if info.owner == super::collection::Owner::Root && info.mutable {
            self.summary.mutable_globals.insert(info.name.clone());
        }
        if info.owner == self.owner {
            if !self.ready.contains(&binding) && self.validate {
                self.errors.push(Failure {
                    span,
                    name: format!("`{}` is not initialized ({route})", info.name),
                });
            }
        } else {
            self.summary.dependencies.insert(binding);
        }
    }

    /// Reject unprovable startup calls and retain uncertainty in routine summaries.
    pub fn unknown(&mut self, span: Span) {
        self.summary.unknown = true;
        if self.worker {
            self.worker_unknown(span);
        }
        if self.validate
            && self.owner == Owner::Root
            && self
                .model
                .bindings
                .iter()
                .any(|(id, binding)| binding.startup && !self.ready.contains(id))
        {
            self.errors.push(Failure {
                span,
                name: "callable dependencies are unknown during initialization".to_owned(),
            });
        }
    }

    /// Require capture storage to exist before a callable environment is created.
    pub fn captures(&mut self, owner: Owner, span: Span) {
        let Some(routine) = self.model.routines.get(&owner) else {
            return;
        };
        let captures = match owner {
            Owner::Closure(_) => self
                .checker
                .closure_infos
                .get(&routine.capture_key)
                .map(|info| &info.captures),
            _ => self
                .checker
                .nested_routine_captures
                .get(&routine.capture_key)
                .map(|info| &info.captures),
        };
        if let Some(captures) = captures {
            for capture in captures {
                self.dependency(identity(capture.declaration), span, "capture creation");
            }
        }
    }

    /// Read the semantically selected binding and evaluate its written index expressions.
    pub fn designator(&mut self, designator: &Designator, capture: bool) -> Value {
        for part in &designator.parts {
            if let fpas_parser::DesignatorPart::Index(value, _) = part {
                self.expr(value);
            }
        }
        let Some(reference) = self
            .checker
            .initialization
            .references
            .get(&identity(designator.span))
        else {
            return Value::new();
        };
        if reference.declaration.is_none() && reference.kind == SymbolKind::Var {
            self.summary.mutable_globals.insert(reference.name.clone());
        }
        if let Some(declaration) = reference.declaration {
            if matches!(reference.kind, SymbolKind::Function | SymbolKind::Procedure) {
                let owner = Owner::Routine(declaration);
                if capture {
                    self.captures(owner, designator.span);
                }
                return [Origin::Routine(owner)].into_iter().collect();
            }
            self.dependency(declaration, designator.span, "value access");
            return self.binding_value(declaration);
        }
        if matches!(
            reference.kind,
            SymbolKind::Function | SymbolKind::Procedure | SymbolKind::BuiltinStd
        ) {
            return [Origin::Imported(identity(designator.span))]
                .into_iter()
                .collect();
        }
        if holds_callable(&reference.ty, self.checker, &mut HashSet::new()) {
            [Origin::Unknown].into_iter().collect()
        } else {
            Value::new()
        }
    }

    /// Recover immutable initializer origins while treating mutable storage conservatively.
    pub fn binding_value(&mut self, declaration: BindingId) -> Value {
        if let Some(value) = self.values.get(&declaration) {
            return value.clone();
        }
        let Some(binding) = self.model.bindings.get(&declaration) else {
            return [Origin::Unknown].into_iter().collect();
        };
        if binding.mutable {
            return [Origin::Binding(declaration)].into_iter().collect();
        }
        if let Some(index) = binding.parameter {
            return if binding.owner == self.owner {
                [Origin::Parameter(index)].into_iter().collect()
            } else {
                [Origin::Binding(declaration)].into_iter().collect()
            };
        }
        if !self.resolving.insert(declaration) {
            return [Origin::Unknown].into_iter().collect();
        }
        let mut probe = Self::new(
            self.checker,
            self.model,
            self.summaries,
            binding.owner,
            false,
        );
        probe.resolving = self.resolving.clone();
        probe.invoking = self.invoking.clone();
        let value = binding
            .initializer
            .map(|value| probe.expr(value))
            .unwrap_or_default();
        self.resolving.remove(&declaration);
        value
    }

    /// Drop origins when the checked static type cannot contain a callable.
    pub fn filtered(&self, expr: &Expr, value: Value) -> Value {
        if self
            .checker
            .expr_types
            .get(&crate::expr_lookup_key(expr))
            .is_some_and(|ty| !holds_callable(ty, self.checker, &mut HashSet::new()))
        {
            Value::new()
        } else {
            value
        }
    }

    /// Invalidate callable provenance when a call or assignment may change its storage.
    pub fn write(&mut self, binding: BindingId) {
        self.values
            .insert(binding, [Origin::Unknown].into_iter().collect());
        if self
            .model
            .bindings
            .get(&binding)
            .is_some_and(|binding| binding.owner != self.owner)
        {
            self.summary.writes.insert(binding);
        }
    }
}

fn holds_callable(ty: &crate::types::Ty, checker: &Checker, seen: &mut HashSet<String>) -> bool {
    use crate::types::Ty;
    match ty {
        Ty::Function(_) | Ty::Procedure(_) | Ty::GenericParam(_, _) => true,
        Ty::Array(inner) | Ty::Option(inner) | Ty::Channel(inner) | Ty::Task(inner) => {
            holds_callable(inner, checker, seen)
        }
        Ty::Dict(key, value) | Ty::Result(key, value) => {
            holds_callable(key, checker, seen) || holds_callable(value, checker, seen)
        }
        Ty::Record(record) => {
            seen.insert(record.name.to_ascii_lowercase())
                && record
                    .fields
                    .iter()
                    .any(|(_, ty)| holds_callable(ty, checker, seen))
        }
        Ty::Enum(enumeration) => {
            seen.insert(enumeration.name.to_ascii_lowercase())
                && enumeration.variants.iter().any(|variant| {
                    variant
                        .fields
                        .iter()
                        .any(|(_, ty)| holds_callable(ty, checker, seen))
                })
        }
        Ty::Named(name) => {
            seen.insert(name.to_ascii_lowercase())
                && checker
                    .scopes
                    .lookup_type(name)
                    .is_some_and(|symbol| holds_callable(&symbol.ty, checker, seen))
        }
        _ => false,
    }
}
