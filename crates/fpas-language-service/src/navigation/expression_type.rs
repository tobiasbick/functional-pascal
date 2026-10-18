//! Source-position queries over the compiler's checked expression metadata.

use std::path::Path;
use std::sync::Arc;

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Span;
use fpas_parser::{
    CaseLabel, CompilationUnit, Decl, Designator, DesignatorPart, Expr, FuncBody, Pattern,
    PostfixOperation, RecordMethod, Stmt, TypeBody,
};
use fpas_sema::{AnalysisMetadata, Ty};

use crate::{LanguageService, LanguageServiceError, NavigationResult};

/// A checked expression type tied to its exact source range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionType {
    /// Functional Pascal spelling of the inferred or declared expression type.
    pub type_name: String,
    /// Smallest checked expression or chain prefix containing the query position.
    pub span: SourceSpan,
}

impl LanguageService {
    /// Returns the smallest checked expression containing a UTF-8 byte offset.
    ///
    /// Uses compiler metadata from the original snapshot AST, including generic substitutions.
    /// Positions outside expressions and expressions with unknown types return `None`.
    pub fn expression_type(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Option<ExpressionType>>, LanguageServiceError> {
        let analysis = self.analyze_document(path)?;
        let value = analysis.semantic().and_then(|semantic| {
            let mut finder = Finder {
                metadata: semantic.metadata(),
                offset,
                found: None,
            };
            match analysis.snapshot().compilation_unit() {
                CompilationUnit::Program(program) => {
                    finder.declarations(&program.declarations);
                    finder.statements(&program.body);
                }
                CompilationUnit::Unit(unit) => finder.declarations(&unit.declarations),
            }
            finder.found
        });
        let value = value.filter(|_| {
            !fpas_lexer::collect_comments(analysis.snapshot().source())
                .iter()
                .any(|comment| {
                    comment.span.offset <= offset
                        && offset < comment.span.offset.saturating_add(comment.span.length)
                })
        });
        Ok(NavigationResult {
            snapshot: Arc::clone(analysis.snapshot()),
            value,
        })
    }
}

struct Finder<'a> {
    metadata: &'a AnalysisMetadata,
    offset: usize,
    found: Option<ExpressionType>,
}

impl Finder<'_> {
    fn consider(&mut self, span: Span, ty: Option<&Ty>) {
        if span.offset <= self.offset
            && self.offset < span.offset.saturating_add(span.length)
            && self
                .found
                .as_ref()
                .is_none_or(|found| span.length < found.span.length())
            && let Some(ty) = ty.filter(|ty| !ty.is_error())
        {
            self.found = Some(ExpressionType {
                type_name: ty.to_string(),
                span: span.diagnostic_span_or_synthetic(),
            });
        }
    }

    fn declarations(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            match declaration {
                Decl::Const(value) => self.expression(&value.value),
                Decl::Var(value) => self.expression(&value.value),
                Decl::Function(function) => {
                    self.contracts(&function.requires, &function.ensures);
                    self.body(&function.body);
                }
                Decl::Procedure(procedure) => {
                    self.contracts(&procedure.requires, &procedure.ensures);
                    self.body(&procedure.body);
                }
                Decl::TypeDef(definition) => match &definition.body {
                    TypeBody::Subrange { lower, upper } => {
                        self.expression(lower);
                        self.expression(upper);
                    }
                    TypeBody::Record(record) => {
                        for field in &record.fields {
                            self.optional(field.default_value.as_ref());
                        }
                        for method in &record.methods {
                            match method {
                                RecordMethod::Function(function)
                                | RecordMethod::StaticFunction(function) => {
                                    self.contracts(&function.requires, &function.ensures);
                                    self.body(&function.body);
                                }
                                RecordMethod::Procedure(procedure)
                                | RecordMethod::StaticProcedure(procedure) => {
                                    self.contracts(&procedure.requires, &procedure.ensures);
                                    self.body(&procedure.body);
                                }
                            }
                        }
                    }
                    TypeBody::Enum(_) | TypeBody::Alias(_) | TypeBody::Distinct(_) => {}
                },
            }
        }
    }

    fn contracts(
        &mut self,
        requires: &[fpas_parser::RequiresClause],
        ensures: &[fpas_parser::EnsuresClause],
    ) {
        for clause in requires {
            self.expression(&clause.condition);
        }
        for clause in ensures {
            self.expression(&clause.condition);
        }
    }

    fn body(&mut self, body: &FuncBody) {
        let FuncBody::Block { nested, stmts } = body;
        self.declarations(nested);
        self.statements(stmts);
    }

    fn statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Stmt) {
        let span = statement.span();
        if self.offset < span.offset || self.offset >= span.offset.saturating_add(span.length) {
            return;
        }
        match statement {
            Stmt::Block(body, _) | Stmt::Scope { body, .. } => self.statements(body),
            Stmt::Const(binding) | Stmt::Var(binding) => self.expression(&binding.value),
            Stmt::Assign { target, value, .. } => {
                self.designator(target);
                self.expression(value);
            }
            Stmt::Return(value, _) => self.optional(value.as_ref()),
            Stmt::Panic(value, _)
            | Stmt::Discard { expr: value, .. }
            | Stmt::Expression { expr: value, .. } => self.expression(value),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expression(condition);
                self.statement(then_branch);
                if let Some(branch) = else_branch {
                    self.statement(branch);
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.expression(expr);
                for arm in arms {
                    self.labels(&arm.labels);
                    self.optional(arm.guard.as_ref());
                    self.statement(&arm.body);
                }
                if let Some(body) = else_body {
                    self.statements(body);
                }
            }
            Stmt::For {
                start, end, body, ..
            } => {
                self.expression(start);
                self.expression(end);
                self.statement(body);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.expression(iterable);
                self.statement(body);
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expression(condition);
                self.statement(body);
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.statements(body);
                self.expression(condition);
            }
            Stmt::Call {
                designator, args, ..
            } => {
                self.designator(designator);
                self.expressions(args);
            }
            Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn optional(&mut self, expression: Option<&Expr>) {
        if let Some(expression) = expression {
            self.expression(expression);
        }
    }

    fn expressions(&mut self, expressions: &[Expr]) {
        for expression in expressions {
            self.expression(expression);
        }
    }

    fn expression(&mut self, expression: &Expr) {
        let span = expression.span();
        if self.offset < span.offset || self.offset >= span.offset.saturating_add(span.length) {
            return;
        }
        self.consider(
            span,
            self.metadata
                .expr_types
                .get(&fpas_sema::expr_lookup_key(expression)),
        );
        match expression {
            Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
                self.designator(designator)
            }
            Expr::Call {
                designator, args, ..
            } => {
                self.designator(designator);
                self.expressions(args);
            }
            Expr::UnaryOp { operand, .. }
            | Expr::Paren(operand, _)
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _)
            | Expr::Try(operand, _)
            | Expr::Go(operand, _)
            | Expr::NamedArgument { value: operand, .. } => self.expression(operand),
            Expr::BinaryOp { left, right, .. } => {
                self.expression(left);
                self.expression(right);
            }
            Expr::ArrayLiteral(values, _) => self.expressions(values),
            Expr::DictLiteral(values, _) => {
                for (key, value) in values {
                    self.expression(key);
                    self.expression(value);
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.expression(base);
                for field in fields {
                    self.expression(&field.value);
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.expression(base);
                for operation in operations {
                    let suffix = match operation {
                        PostfixOperation::Field { span, .. } => *span,
                        PostfixOperation::Index { index, span } => {
                            self.expression(index);
                            *span
                        }
                        PostfixOperation::MethodCall { args, span, .. } => {
                            self.expressions(args);
                            *span
                        }
                    };
                    let ty = self
                        .metadata
                        .postfix_types
                        .get(&(suffix.source_id, suffix.offset))
                        .or_else(|| {
                            self.metadata
                                .path_types
                                .get(&(suffix.source_id, suffix.offset))
                        });
                    self.consider(
                        Span {
                            length: suffix.offset + suffix.length - span.offset,
                            ..span
                        },
                        ty,
                    );
                }
            }
            Expr::Closure(closure) => self.body(&closure.body),
            Expr::Is { value, pattern, .. } => {
                self.expression(value);
                self.pattern(pattern);
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                for branch in branches {
                    self.expression(&branch.condition);
                    self.expression(&branch.value);
                }
                self.expression(else_value);
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                self.expression(selector);
                for arm in arms {
                    self.labels(&arm.labels);
                    self.optional(arm.guard.as_ref());
                    self.expression(&arm.value);
                }
                if let Some(arm) = else_arm {
                    self.expression(&arm.value);
                }
            }
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Error(_) => {}
        }
    }

    fn designator(&mut self, designator: &Designator) {
        for part in &designator.parts {
            let suffix = match part {
                DesignatorPart::Ident(_, span) => *span,
                DesignatorPart::Index(index, span) => {
                    self.expression(index);
                    *span
                }
            };
            self.consider(
                Span {
                    length: suffix.offset + suffix.length - designator.span.offset,
                    ..designator.span
                },
                self.metadata
                    .path_types
                    .get(&(suffix.source_id, suffix.offset)),
            );
        }
    }

    fn labels(&mut self, labels: &[CaseLabel]) {
        for label in labels {
            match label {
                CaseLabel::Value { start, end, .. } => {
                    self.expression(start);
                    self.optional(end.as_ref());
                }
                CaseLabel::Pattern(pattern) => self.pattern(pattern),
                CaseLabel::Binding { .. } => {}
            }
        }
    }

    fn pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Value(value) => self.expression(value),
            Pattern::Variant { fields, .. } | Pattern::Destructure { fields, .. } => {
                for field in fields {
                    self.pattern(&field.pattern);
                }
            }
            Pattern::Binding { .. } | Pattern::Wildcard(_) => {}
        }
    }
}
