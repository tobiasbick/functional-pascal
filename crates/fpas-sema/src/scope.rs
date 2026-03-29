use crate::types::Ty;
use fpas_lexer::Span;
use std::collections::HashMap;

/// A symbol in the scope.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Ty,
    pub mutable: bool,
    pub kind: SymbolKind,
}

/// A routine declared with `forward` that still requires a matching body.
///
/// **Documentation:** `docs/pascal/04-functions.md`
#[derive(Debug, Clone)]
pub struct PendingRoutine {
    pub symbol: Symbol,
    pub span: Span,
}

impl Symbol {
    pub fn ty_mut(&mut self) -> &mut Ty {
        &mut self.ty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Const,
    Var,
    Param,
    Function,
    Procedure,
    /// Polymorphic standard-library call (`Std.Math.Abs`, `Std.Array.Push`, …).
    BuiltinStd,
    Type,
    EnumMember,
    /// Enum variant that carries associated data and must be constructed with arguments.
    EnumVariantConstructor,
    ForVar,
}

/// A single scope level.
#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
    pending_routines: HashMap<String, PendingRoutine>,
}

impl Scope {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            pending_routines: HashMap::new(),
        }
    }
}

/// Stack of scopes for lexical scoping.
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    /// Current loop depth (for break/continue validation).
    pub loop_depth: u32,
    /// Current function context (for return validation).
    pub function_ctx: Option<FunctionCtx>,
}

#[derive(Debug, Clone)]
pub struct FunctionCtx {
    pub name: String,
    pub return_type: Option<Ty>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            loop_depth: 0,
            function_ctx: None,
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current (innermost) scope.
    /// Returns false if already defined in the same scope.
    pub fn define(&mut self, name: &str, symbol: Symbol) -> bool {
        let scope_index = self.scopes.len() - 1;
        let scope = &mut self.scopes[scope_index];
        if scope.symbols.contains_key(name) {
            return false;
        }
        scope.symbols.insert(name.to_string(), symbol);
        true
    }

    /// Look up a symbol by name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }

    /// Look up a symbol only in the current (innermost) scope.
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| scope.symbols.get(name))
    }

    /// Mutable lookup for updating a symbol after initial definition.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.symbols.get_mut(name) {
                return Some(sym);
            }
        }
        None
    }

    /// Register a pending forward routine in the current scope.
    pub fn define_pending_routine(&mut self, name: &str, pending: PendingRoutine) {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope stack must always contain at least one scope");
        scope.pending_routines.insert(name.to_string(), pending);
    }

    /// Remove and return a pending forward routine from the current scope.
    pub fn take_pending_routine(&mut self, name: &str) -> Option<PendingRoutine> {
        self.scopes
            .last_mut()
            .and_then(|scope| scope.pending_routines.remove(name))
    }

    /// Drain pending forward routines from the current scope.
    pub fn drain_pending_routines(&mut self) -> Vec<(String, PendingRoutine)> {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope stack must always contain at least one scope");
        scope.pending_routines.drain().collect()
    }

    /// Return all symbol names that start with a given prefix.
    pub fn names_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.scopes {
            for name in scope.symbols.keys() {
                if name.starts_with(prefix) {
                    names.push(name.clone());
                }
            }
        }
        names
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}
