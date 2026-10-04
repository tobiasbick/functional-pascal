use crate::types::Ty;
use fpas_lexer::Span;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub(crate) fn canonical_symbol_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// A symbol in the scope.
///
/// **Documentation:** `docs/pascal/language/basics/variables.md` (from the repository root).
#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Ty,
    pub mutable: bool,
    pub kind: SymbolKind,
    /// `true` when this binding holds a task-bound callable (mutable captures).
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub task_bound: bool,
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
    import_aliases: HashMap<String, Span>,
    pub(crate) import_alias_conflicts: Vec<(String, Span)>,
    /// Current loop depth (for break/continue validation).
    pub loop_depth: u32,
    /// Current function context (for return validation).
    pub function_ctx: Option<FunctionCtx>,
}

/// Nested lexical scopes temporarily detached while resolving a root declaration.
pub(crate) struct NestedScopes(Vec<Scope>);

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
    /// Resolve a unit-level header without inheriting another header's type parameters.
    pub(crate) fn suspend_nested_scopes(&mut self) -> NestedScopes {
        NestedScopes(self.scopes.split_off(1))
    }

    /// Restore the lexical scopes suspended for one unit-level header.
    pub(crate) fn restore_nested_scopes(&mut self, nested: NestedScopes) {
        self.scopes.extend(nested.0);
    }

    /// Update a collected unit-level declaration without selecting a local shadow.
    pub(crate) fn lookup_root_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.scopes[0]
            .symbols
            .get_mut(&canonical_symbol_name(name))
            .map(|symbol| &mut symbol.symbol)
    }
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            import_aliases: HashMap::new(),
            import_alias_conflicts: Vec::new(),
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
        self.record_import_shadow(name, None);
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
        self.record_import_shadow(name, Some(declaration));
        let scope_index = self.scopes.len() - 1;
        Self::define_in_scope(
            &mut self.scopes[scope_index],
            name,
            symbol,
            Some(declaration),
        )
    }

    /// Define a linked unit symbol in the outermost (program) scope.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md` (from the repository root).
    pub fn define_in_root(&mut self, name: &str, symbol: Symbol) -> bool {
        self.record_import_shadow(name, None);
        Self::define_in_scope(&mut self.scopes[0], name, symbol, None)
    }

    pub(crate) fn reserve_import_alias(&mut self, name: &str, span: Span) {
        self.import_aliases
            .insert(canonical_symbol_name(name), span);
    }

    // The declaration is still defined so the alias conflict is the only diagnostic.
    fn record_import_shadow(&mut self, name: &str, declaration: Option<Span>) {
        if let Some(alias_span) = self.import_aliases.get(&canonical_symbol_name(name)) {
            self.import_alias_conflicts
                .push((name.to_owned(), declaration.unwrap_or(*alias_span)));
        }
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
                original_name: name.to_string(),
                symbol,
                declaration,
            },
        );
        true
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
                ty: Ty::Integer,
                mutable: false,
                kind: SymbolKind::Var,
                task_bound: false,
            }
        ));
        assert!(stack.remove_from_current("Offset"));
        assert!(stack.lookup("offset").is_none());
        assert!(!stack.remove_from_current("offset"));
    }
}
