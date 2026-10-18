mod captures;
mod imports;

use crate::types::Ty;
use fpas_lexer::Span;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub(crate) fn canonical_symbol_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Classification and known scalar or record-field values of a constant binding.
///
/// **Documentation:** `docs/pascal/language/basics/constants.md`
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantInfo {
    /// Known static record fields used by constant projections and pattern coverage.
    pub record: Option<std::sync::Arc<fpas_unit::interface::RecordConstant>>,
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
    original_name: String,
    symbol: Symbol,
    declaration: Option<Span>,
    routine_capture_key: Option<usize>,
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
    declaration_types: HashMap<(u32, usize), Ty>,
    pub(crate) imports: imports::ImportNames,
    scopes: Vec<Scope>,
    /// Recorded parameter escapes of imported routines and record routines by canonical name.
    param_escapes: HashMap<String, std::sync::Arc<[bool]>>,
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
            declaration_types: HashMap::new(),
            scopes: vec![Scope::new()],
            imports: Default::default(),
            param_escapes: HashMap::new(),
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
        if let Some(scope) = self.scopes.pop() {
            for symbol in scope.symbols.into_values() {
                if let Some(span) = symbol.declaration {
                    self.declaration_types
                        .insert((span.source_id, span.offset), symbol.symbol.ty);
                }
            }
        }
    }

    /// Retains checked source types after lexical scopes have left the checker.
    pub(crate) fn declaration_types(&self) -> HashMap<(u32, usize), Ty> {
        let mut types = self.declaration_types.clone();
        for scope in &self.scopes {
            for symbol in scope.symbols.values() {
                if let Some(span) = symbol.declaration {
                    types.insert((span.source_id, span.offset), symbol.symbol.ty.clone());
                }
            }
        }
        types
    }

    /// Define a symbol in the current (innermost) scope.
    /// Returns false if already defined in the same scope.
    pub fn define(&mut self, name: &str, symbol: Symbol) -> bool {
        if self.imports.is_alias(name) {
            return false;
        }
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
        if self.imports.is_alias(name) {
            return false;
        }
        let scope_index = self.scopes.len() - 1;
        Self::define_in_scope(
            &mut self.scopes[scope_index],
            name,
            symbol,
            Some(declaration),
        )
    }

    /// Define a local qualified source symbol in the outermost scope, including enum variants.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md` (from the repository root).
    pub fn define_in_root(&mut self, name: &str, symbol: Symbol) -> bool {
        if self.imports.is_alias(name) {
            return false;
        }
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
                original_name: name.to_string(),
                symbol,
                declaration,
                routine_capture_key: None,
            },
        );
        true
    }

    /// Retains source identity when a declaration replaces an imported builtin.
    pub(crate) fn set_declaration(&mut self, name: &str, declaration: Span) {
        let name = canonical_symbol_name(name);
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.symbols.get_mut(&name) {
                symbol.declaration = Some(declaration);
                return;
            }
        }
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
        self.imports
            .lookup_source(name)
            .map(|symbol| (0, symbol, None))
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
        if let Some(symbol) = self.imports.lookup_metadata(name)
            && symbol.kind == SymbolKind::Type
        {
            return Some(symbol);
        }
        let canonical = canonical_symbol_name(name);
        self.scopes
            .first()
            .and_then(|scope| scope.symbols.get(&canonical))
            .filter(|entry| entry.symbol.kind == SymbolKind::Type)
            .map(|entry| &entry.symbol)
    }

    /// Records which parameters of an imported routine may outlive a call.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`
    pub(crate) fn set_param_escapes(&mut self, name: &str, escapes: std::sync::Arc<[bool]>) {
        self.param_escapes
            .insert(self.imports.canonical_name(name), escapes);
    }

    /// Returns recorded parameter escapes of an imported routine, if `name` denotes one.
    pub(crate) fn param_escapes(&self, name: &str) -> Option<std::sync::Arc<[bool]>> {
        self.param_escapes
            .get(&self.imports.canonical_name(name))
            .cloned()
    }

    /// Look up the original stored spelling for a symbol name.
    pub fn lookup_original_name(&self, name: &str) -> Option<&str> {
        let canonical_name = canonical_symbol_name(name);
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(&canonical_name) {
                return Some(&sym.original_name);
            }
        }
        self.imports.original_source_name(name)
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
        let mut types = self.scopes[0]
            .symbols
            .iter()
            .filter(|(_, entry)| entry.symbol.kind == SymbolKind::Type)
            .map(|(name, entry)| (name.clone(), entry.symbol.ty.clone()))
            .collect::<BTreeMap<_, _>>();
        types.extend(self.imports.type_symbols());
        types
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
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
