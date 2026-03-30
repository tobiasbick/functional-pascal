use super::NameRewriter;
use fpas_parser::Decl;

impl NameRewriter<'_> {
    pub(super) fn resolve_import_name(
        &mut self,
        short_name: &str,
        line: u32,
        column: u32,
    ) -> Option<String> {
        if let Some(candidates) = self.ambiguous.get(short_name) {
            self.record_error(format!(
                "{}:{}:{}: error: Ambiguous imported symbol `{short_name}`.\n  help: Use a fully qualified name. Candidates: {}.",
                self.path,
                line,
                column,
                candidates.join(", ")
            ));
            return None;
        }
        self.resolved.get(short_name).cloned()
    }

    pub(super) fn predeclare_decl_name(&mut self, decl: &Decl) {
        match decl {
            Decl::Const(c) => self.declare_value(&c.name),
            Decl::Var(v) | Decl::MutableVar(v) => self.declare_value(&v.name),
            Decl::TypeDef(td) => self.declare_type(&td.name),
            Decl::Function(f) => self.declare_value(&f.name),
            Decl::Procedure(p) => self.declare_value(&p.name),
        }
    }

    pub(super) fn push_scope(&mut self) {
        self.value_scopes.push(Default::default());
        self.type_scopes.push(Default::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.value_scopes.pop();
        self.type_scopes.pop();
    }

    pub(super) fn declare_value(&mut self, name: &str) {
        if let Some(scope) = self.value_scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub(super) fn declare_type(&mut self, name: &str) {
        if let Some(scope) = self.type_scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub(super) fn is_local_value(&self, name: &str) -> bool {
        self.value_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    pub(super) fn is_local_type(&self, name: &str) -> bool {
        self.type_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    pub(super) fn record_error(&mut self, message: String) {
        if self.first_error.is_none() {
            self.first_error = Some(message);
        }
    }
}
