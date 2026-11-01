//! AST-owned routine boundaries and directly written annotations, without callee expansion.

use fpas_lexer::{Span, Token};
use fpas_parser::{
    CompilationUnit, Decl, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, RecordMethod, Stmt,
    TypeBody, TypeExpr,
};

#[derive(Clone, Copy)]
pub(super) enum Routine<'a> {
    Function(&'a FunctionDecl),
    Procedure(&'a ProcedureDecl),
}
impl<'a> Routine<'a> {
    pub(super) fn body(self) -> &'a FuncBody {
        match self {
            Self::Function(f) => &f.body,
            Self::Procedure(p) => &p.body,
        }
    }
    pub(super) fn span(self) -> Span {
        match self {
            Self::Function(f) => f.span,
            Self::Procedure(p) => p.span,
        }
    }
    fn params(self) -> &'a [FormalParam] {
        match self {
            Self::Function(f) => &f.params,
            Self::Procedure(p) => &p.params,
        }
    }
    pub(super) fn annotations(self, visit: &mut impl FnMut(&TypeExpr, &'static str) -> bool) {
        for param in self
            .params()
            .iter()
            .filter(|p| !p.name.eq_ignore_ascii_case("Self"))
        {
            if !visit(&param.type_expr, "signature") {
                return;
            }
        }
        if let Self::Function(f) = self
            && !visit(&f.return_type, "signature")
        {
            return;
        }
        for param in self
            .params()
            .iter()
            .filter(|p| p.name.eq_ignore_ascii_case("Self"))
        {
            if !visit(&param.type_expr, "receiver") {
                return;
            }
        }
        let FuncBody::Block { nested, stmts } = self.body();
        // Nested routines have their own inspection scope.
        for declaration in nested {
            let ty = match declaration {
                Decl::Const(c) => Some(&c.type_expr),
                Decl::Var(v) => Some(&v.type_expr),
                _ => None,
            };
            if let Some(ty) = ty
                && !visit(ty, "local_annotation")
            {
                return;
            }
        }
        statements(stmts, visit);
    }
    pub(super) fn body_start(self, source: &str) -> Option<usize> {
        let (requires, ensures, result_type) = match self {
            Self::Function(f) => (&f.requires, &f.ensures, Some(type_span(&f.return_type))),
            Self::Procedure(p) => (&p.requires, &p.ensures, None),
        };
        let header_end = requires
            .iter()
            .map(|c| c.span)
            .chain(ensures.iter().map(|c| c.span))
            .chain(result_type)
            .chain(self.params().iter().map(|p| p.span))
            .map(|span| span.offset + span.length)
            .max()
            .unwrap_or(self.span().offset);
        fpas_lexer::lex(source)
            .0
            .into_iter()
            .filter(|t| {
                t.span.offset > header_end
                    && t.span.offset < self.span().offset + self.span().length
                    && matches!(
                        t.token,
                        Token::Begin
                            | Token::Const
                            | Token::Var
                            | Token::Function
                            | Token::Procedure
                    )
            })
            .map(|t| t.span.offset)
            .next()
    }
}

pub(super) fn declarations(unit: &CompilationUnit) -> &[Decl] {
    match unit {
        CompilationUnit::Unit(unit) => &unit.declarations,
        CompilationUnit::Program(program) => &program.declarations,
    }
}

pub(super) fn routine(declarations: &[Decl], start: usize) -> Option<Routine<'_>> {
    for declaration in declarations {
        let candidate = match declaration {
            Decl::Function(f) => Some(Routine::Function(f)),
            Decl::Procedure(p) => Some(Routine::Procedure(p)),
            _ => None,
        };
        if let Some(candidate) = candidate {
            if candidate.span().offset == start {
                return Some(candidate);
            }
            let FuncBody::Block { nested, .. } = candidate.body();
            if let Some(found) = routine(nested, start) {
                return Some(found);
            }
        }
        if let Decl::TypeDef(definition) = declaration
            && let TypeBody::Record(record) = &definition.body
        {
            for method in &record.methods {
                let candidate = match method {
                    RecordMethod::Function(f) | RecordMethod::StaticFunction(f) => {
                        Routine::Function(f)
                    }
                    RecordMethod::Procedure(p) | RecordMethod::StaticProcedure(p) => {
                        Routine::Procedure(p)
                    }
                };
                if candidate.span().offset == start {
                    return Some(candidate);
                }
                let FuncBody::Block { nested, .. } = candidate.body();
                if let Some(found) = routine(nested, start) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub(super) fn named_types(
    ty: &TypeExpr,
    visit: &mut impl FnMut(&fpas_parser::QualifiedId) -> bool,
) -> bool {
    match ty {
        TypeExpr::Named { id, .. } => visit(id),
        TypeExpr::Application { id, arguments, .. } => {
            visit(id) && arguments.iter().all(|ty| named_types(ty, visit))
        }
        TypeExpr::Array(inner, _)
        | TypeExpr::Channel(inner, _)
        | TypeExpr::Task(inner, _)
        | TypeExpr::Option {
            inner_type: inner, ..
        } => named_types(inner, visit),
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => {
            params.iter().all(|p| named_types(&p.type_expr, visit))
                && named_types(return_type, visit)
        }
        TypeExpr::ProcedureType { params, .. } => {
            params.iter().all(|p| named_types(&p.type_expr, visit))
        }
        TypeExpr::Result {
            ok_type, err_type, ..
        } => named_types(ok_type, visit) && named_types(err_type, visit),
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => named_types(key_type, visit) && named_types(value_type, visit),
    }
}

fn statements(body: &[Stmt], visit: &mut impl FnMut(&TypeExpr, &'static str) -> bool) -> bool {
    body.iter().all(|statement| match statement {
        Stmt::Const(binding) | Stmt::Var(binding) => binding
            .type_expr
            .as_ref()
            .is_none_or(|ty| visit(ty, "local_annotation")),
        Stmt::Block(body, _) | Stmt::Scope { body, .. } | Stmt::Repeat { body, .. } => {
            statements(body, visit)
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            statements(std::slice::from_ref(then_branch), visit)
                && else_branch
                    .as_ref()
                    .is_none_or(|body| statements(std::slice::from_ref(body), visit))
        }
        Stmt::Case {
            arms, else_body, ..
        } => {
            arms.iter()
                .all(|arm| statements(std::slice::from_ref(&arm.body), visit))
                && else_body
                    .as_ref()
                    .is_none_or(|body| statements(body, visit))
        }
        Stmt::For { var_type, body, .. } | Stmt::ForIn { var_type, body, .. } => {
            visit(var_type, "local_annotation") && statements(std::slice::from_ref(body), visit)
        }
        Stmt::While { body, .. } => statements(std::slice::from_ref(body), visit),
        _ => true,
    })
}

fn type_span(ty: &TypeExpr) -> Span {
    match ty {
        TypeExpr::Named { span, .. }
        | TypeExpr::Application { span, .. }
        | TypeExpr::FunctionType { span, .. }
        | TypeExpr::ProcedureType { span, .. }
        | TypeExpr::Result { span, .. }
        | TypeExpr::Option { span, .. }
        | TypeExpr::Dict { span, .. }
        | TypeExpr::Array(_, span)
        | TypeExpr::Channel(_, span)
        | TypeExpr::Task(_, span) => *span,
    }
}
