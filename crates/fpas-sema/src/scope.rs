use crate::types::Ty;
use fpas_lexer::Span;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub(crate) fn canonical_symbol_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Classification and optional scalar value of a constant binding.
///
/// **Documentation:** `docs/pascal/language/basics/constants.md`
#[derive(Debug, Clone)]
pub struct ConstantInfo {
    /// True only for the language's compile-time constant-expression forms.
    pub compile_time: bool,
    /// Scalar value that can be embedded in a compiled-unit interface.
    pub value: Option<fpas_unit::interface::ConstantValue>,
}

/// A resolved lexical binding, including constant classification.
///
/// **Documentation:** `docs/pascal/language/basics/variables.md`
#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Ty,
    pub mutable: bool,
    pub kind: SymbolKind,
    /// `true` when this binding holds a task-bound callable (mutable captures).
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub task_bound: bool,
    /// Constant-expression classification, independent of binding mutability.
    pub constant: Option<ConstantInfo>,
}

impl Symbol {
    /// Returns whether this binding is a `var` parameter.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    pub(crate) fn is_var_parameter(&self) -> bool {
        self.kind == SymbolKind::Param && self.mutable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Const,
    Var,
    Param,
    Function,
    Procedure,
    /// Polymorphic standard-library call (`Std.Math.Abs`, `Std.Arrays.Push`, …).
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
    discard: fpas_unit::interface::DiscardInfo,
    original_name: String,
    symbol: Symbol,
    declaration: Option<Span>,
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
    imported_discard: HashMap<String, fpas_unit::interface::DiscardInfo>,
    /// Current loop depth (for break/continue validation).
    pub loop_depth: u32,
    /// Current function context (for return validation).
    pub function_ctx: Option<FunctionCtx>,
}

/// Semantic context of the routine whose body is currently checked.
#[derive(Debug, Clone)]
pub struct FunctionCtx {
    /// Diagnostic name of the routine.
    pub name: String,
    /// Declared result type, or `None` for procedures and the program body.
    pub return_type: Option<Ty>,
    /// Exact linked unit that owns the routine; nested routines inherit it.
    pub owner_unit: Option<String>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            imported_discard: HashMap::new(),
            loop_depth: 0,
            function_ctx: None,
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the innermost scope. The program root scope (index 0) is never removed.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }
        self.scopes.pop();
    }

    /// Define a symbol in the current (innermost) scope.
    /// Returns false if already defined in the same scope.
    pub fn define(&mut self, name: &str, symbol: Symbol) -> bool {
        let scope_index = self.scopes.len() - 1;
        Self::define_in_scope(&mut self.scopes[scope_index], name, symbol, None)
    }

    /// Define a source binding in the current scope with its exact declaration span.
    pub fn define_with_declaration(
        &mut self,
        name: &str,
        symbol: Symbol,
        declaration: Span,
    ) -> bool {
        let scope_index = self.scopes.len() - 1;
        Self::define_in_scope(
            &mut self.scopes[scope_index],
            name,
            symbol,
            Some(declaration),
        )
    }

    /// Define in the outermost (program) scope. Used for `Std.*` short aliases so nested checking
    /// (for example inside a routine body) does not attach imports to a transient inner scope.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md` (from the repository root).
    pub fn define_in_root(&mut self, name: &str, symbol: Symbol) -> bool {
        Self::define_in_scope(&mut self.scopes[0], name, symbol, None)
    }

    fn define_in_scope(
        scope: &mut Scope,
        name: &str,
        symbol: Symbol,
        declaration: Option<Span>,
    ) -> bool {
        let canonical_name = canonical_symbol_name(name);
        if scope.symbols.contains_key(&canonical_name) {
            return false;
        }
        scope.symbols.insert(
            canonical_name,
            ScopedSymbol {
                discard: Default::default(),
                original_name: name.to_string(),
                symbol,
                declaration,
            },
        );
        true
    }

    /// Remove a symbol from the program root scope. Used when rebuilding `Std` short aliases.
    pub fn remove_from_root(&mut self, name: &str) -> bool {
        let canonical_name = canonical_symbol_name(name);
        self.scopes[0].symbols.remove(&canonical_name).is_some()
    }

    /// Remove a symbol from the innermost scope.
    pub fn remove_from_current(&mut self, name: &str) -> bool {
        let canonical_name = canonical_symbol_name(name);
        self.scopes
            .last_mut()
            .is_some_and(|scope| scope.symbols.remove(&canonical_name).is_some())
    }

    /// Look up a symbol and the scope index where it was found (0 = program root).
    pub fn lookup_with_scope(&self, name: &str) -> Option<(usize, &Symbol)> {
        self.lookup_with_scope_and_declaration(name)
            .map(|(scope, symbol, _)| (scope, symbol))
    }

    /// Look up a symbol together with its scope and exact source declaration, when present.
    pub fn lookup_with_scope_and_declaration(
        &self,
        name: &str,
    ) -> Option<(usize, &Symbol, Option<Span>)> {
        let canonical_name = canonical_symbol_name(name);
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(sym) = scope.symbols.get(&canonical_name) {
                return Some((index, &sym.symbol, sym.declaration));
            }
        }
        None
    }

    /// Number of scopes currently on the stack (including the program root).
    #[must_use]
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// Look up a symbol by name, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.lookup_with_scope(name).map(|(_, symbol)| symbol)
    }

    /// Resolves a stored unit-level type identity without accepting a shadowing value or parameter.
    pub(crate) fn lookup_type(&self, name: &str) -> Option<&Symbol> {
        let canonical = canonical_symbol_name(name);
        self.scopes
            .first()
            .and_then(|scope| scope.symbols.get(&canonical))
            .filter(|entry| entry.symbol.kind == SymbolKind::Type)
            .map(|entry| &entry.symbol)
    }

    /// Looks up static value and result guarantees at the same lexical binding as its type.
    pub(crate) fn discard_info(&self, name: &str) -> fpas_unit::interface::DiscardInfo {
        let canonical = canonical_symbol_name(name);
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(&canonical))
            .map(|entry| entry.discard)
            .or_else(|| self.imported_discard.get(&canonical).copied())
            .unwrap_or_default()
    }

    /// Installs guarantees for qualified record members from a unit interface.
    pub(crate) fn set_imported_discard_info(
        &mut self,
        name: &str,
        info: fpas_unit::interface::DiscardInfo,
    ) {
        self.imported_discard
            .insert(canonical_symbol_name(name), info);
    }

    /// Attaches static capture guarantees to an existing binding.
    pub(crate) fn set_discard_info(&mut self, name: &str, info: fpas_unit::interface::DiscardInfo) {
        let canonical = canonical_symbol_name(name);
        for scope in self.scopes.iter_mut().rev() {
            if let Some(entry) = scope.symbols.get_mut(&canonical) {
                entry.discard = info;
                break;
            }
        }
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

    /// Look up a symbol only in the program root scope.
    pub fn lookup_root(&self, name: &str) -> Option<&Symbol> {
        let canonical_name = canonical_symbol_name(name);
        self.scopes[0]
            .symbols
            .get(&canonical_name)
            .map(|entry| &entry.symbol)
    }

    /// Return resolved root type declarations in deterministic canonical-name order.
    pub(crate) fn root_types(&self) -> BTreeMap<String, Ty> {
        self.scopes[0]
            .symbols
            .iter()
            .filter(|(_, entry)| entry.symbol.kind == SymbolKind::Type)
            .map(|(name, entry)| (name.clone(), entry.symbol.ty.clone()))
            .collect()
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

    pub(crate) fn root_symbols_with_prefix(&self, prefix: &str) -> Vec<(String, Symbol)> {
        let canonical_prefix = canonical_symbol_name(prefix);
        let mut symbols = self.scopes[0]
            .symbols
            .iter()
            .filter(|(name, _)| name.starts_with(&canonical_prefix))
            .map(|(_, entry)| (entry.original_name.clone(), entry.symbol.clone()))
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| left.0.cmp(&right.0));
        symbols
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
