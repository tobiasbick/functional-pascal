use crate::types::Ty;
use std::collections::HashMap;

pub(crate) fn canonical_symbol_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// A symbol in the scope.
///
/// **Documentation:** `docs/pascal/02-basics.md` (from the repository root).
#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Ty,
    pub mutable: bool,
    pub kind: SymbolKind,
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
    symbols: HashMap<String, ScopedSymbol>,
}

#[derive(Debug)]
struct ScopedSymbol {
    original_name: String,
    symbol: Symbol,
}

impl Scope {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
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
        Self::define_in_scope(&mut self.scopes[scope_index], name, symbol)
    }

    /// Define in the outermost (program) scope. Used for `Std.*` short aliases so nested checking
    /// (for example inside a routine body) does not attach imports to a transient inner scope.
    ///
    /// **Documentation:** `docs/pascal/09-units.md` (from the repository root).
    pub fn define_in_root(&mut self, name: &str, symbol: Symbol) -> bool {
        Self::define_in_scope(&mut self.scopes[0], name, symbol)
    }

    fn define_in_scope(scope: &mut Scope, name: &str, symbol: Symbol) -> bool {
        let canonical_name = canonical_symbol_name(name);
        if scope.symbols.contains_key(&canonical_name) {
            return false;
        }
        scope.symbols.insert(
            canonical_name,
            ScopedSymbol {
                original_name: name.to_string(),
                symbol,
            },
        );
        true
    }

    /// Remove a symbol from the program root scope. Used when rebuilding `Std` short aliases.
    pub fn remove_from_root(&mut self, name: &str) -> bool {
        let canonical_name = canonical_symbol_name(name);
        self.scopes[0].symbols.remove(&canonical_name).is_some()
    }

    /// Look up a symbol by name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let canonical_name = canonical_symbol_name(name);
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(&canonical_name) {
                return Some(&sym.symbol);
            }
        }
        None
    }

    /// Look up the original stored spelling for a symbol name.
    pub fn lookup_original_name(&self, name: &str) -> Option<&str> {
        let canonical_name = canonical_symbol_name(name);
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(&canonical_name) {
                return Some(&sym.original_name);
            }
        }
        None
    }

    /// Look up a symbol only in the current (innermost) scope.
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        let canonical_name = canonical_symbol_name(name);
        self.scopes
            .last()
            .and_then(|scope| scope.symbols.get(&canonical_name))
            .map(|entry| &entry.symbol)
    }

    /// Mutable lookup for updating a symbol after initial definition.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        let canonical_name = canonical_symbol_name(name);
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.symbols.get_mut(&canonical_name) {
                return Some(&mut sym.symbol);
            }
        }
        None
    }

    /// Return all symbol names that start with a given prefix.
    pub fn names_with_prefix(&self, prefix: &str) -> Vec<String> {
        let canonical_prefix = canonical_symbol_name(prefix);
        let mut names = Vec::new();
        for scope in &self.scopes {
            for (canonical_name, symbol) in &scope.symbols {
                if canonical_name.starts_with(&canonical_prefix) {
                    names.push(symbol.original_name.clone());
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
