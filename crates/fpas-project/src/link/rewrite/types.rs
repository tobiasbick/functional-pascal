use super::NameRewriter;
use fpas_parser::{Decl, FuncBody, TypeBody, TypeExpr, VarDef};

impl NameRewriter<'_> {
    pub(super) fn rewrite_decl(&mut self, decl: &mut Decl) {
        match decl {
            Decl::Const(const_def) => {
                self.rewrite_type_expr(&mut const_def.type_expr);
                self.rewrite_expr(&mut const_def.value);
            }
            Decl::Var(var_def) | Decl::MutableVar(var_def) => {
                self.rewrite_var_def(var_def);
            }
            Decl::TypeDef(type_def) => {
                self.rewrite_type_body(&mut type_def.body);
            }
            Decl::Function(function_decl) => {
                self.rewrite_callable(
                    &mut function_decl.params,
                    Some(&mut function_decl.return_type),
                    &mut function_decl.body,
                );
            }
            Decl::Procedure(procedure_decl) => {
                self.rewrite_callable(&mut procedure_decl.params, None, &mut procedure_decl.body);
            }
        }
    }

    pub(super) fn rewrite_callable(
        &mut self,
        params: &mut [fpas_parser::FormalParam],
        return_type: Option<&mut TypeExpr>,
        body: &mut FuncBody,
    ) {
        for param in params.iter_mut() {
            self.rewrite_type_expr(&mut param.type_expr);
        }
        if let Some(return_type) = return_type {
            self.rewrite_type_expr(return_type);
        }

        let FuncBody::Block { nested, stmts } = body;

        self.push_scope();
        for param in params.iter() {
            self.declare_value(&param.name);
        }
        for decl in nested.iter() {
            self.predeclare_decl_name(decl);
        }
        for decl in nested.iter_mut() {
            self.rewrite_decl(decl);
        }
        for stmt in stmts.iter_mut() {
            self.rewrite_stmt(stmt);
        }
        self.pop_scope();
    }

    pub(super) fn rewrite_type_body(&mut self, body: &mut TypeBody) {
        match body {
            TypeBody::Record(record) => {
                for field in &mut record.fields {
                    self.rewrite_type_expr(&mut field.type_expr);
                }
                for method in &mut record.methods {
                    match method {
                        fpas_parser::RecordMethod::Function(f) => {
                            self.rewrite_callable(
                                &mut f.params,
                                Some(&mut f.return_type),
                                &mut f.body,
                            );
                        }
                        fpas_parser::RecordMethod::Procedure(p) => {
                            self.rewrite_callable(&mut p.params, None, &mut p.body);
                        }
                    }
                }
            }
            TypeBody::Enum(_) => {}
            TypeBody::Alias(type_expr) => self.rewrite_type_expr(type_expr),
        }
    }

    pub(super) fn rewrite_var_def(&mut self, var_def: &mut VarDef) {
        self.rewrite_type_expr(&mut var_def.type_expr);
        self.rewrite_expr(&mut var_def.value);
    }

    pub(super) fn rewrite_type_expr(&mut self, type_expr: &mut TypeExpr) {
        match type_expr {
            TypeExpr::Named { id: name, .. } => {
                if name.parts.len() != 1 {
                    return;
                }
                let short_name = &name.parts[0];
                if self.is_local_type(short_name) {
                    return;
                }
                let Some(qualified) =
                    self.resolve_import_name(short_name, name.span.line, name.span.column)
                else {
                    return;
                };
                name.parts = qualified.split('.').map(str::to_string).collect();
            }
            TypeExpr::Array(inner, _) => self.rewrite_type_expr(inner),
            TypeExpr::FunctionType {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    self.rewrite_type_expr(&mut param.type_expr);
                }
                self.rewrite_type_expr(return_type);
            }
            TypeExpr::ProcedureType { params, .. } => {
                for param in params {
                    self.rewrite_type_expr(&mut param.type_expr);
                }
            }
            TypeExpr::Result {
                ok_type, err_type, ..
            } => {
                self.rewrite_type_expr(ok_type);
                self.rewrite_type_expr(err_type);
            }
            TypeExpr::Option { inner_type, .. } | TypeExpr::Channel { inner_type, .. } => {
                self.rewrite_type_expr(inner_type);
            }
            TypeExpr::Dict {
                key_type,
                value_type,
                ..
            } => {
                self.rewrite_type_expr(key_type);
                self.rewrite_type_expr(value_type);
            }
        }
    }
}
