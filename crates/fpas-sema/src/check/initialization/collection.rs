//! Source ownership and retained expressions for initialization analysis.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::{BindingId, identity};
use crate::check::Checker;
use fpas_lexer::Span;
use fpas_parser::{Decl, Expr, FormalParam, FuncBody, RecordMethod, Stmt, TypeBody, Visibility};
use std::collections::BTreeMap;

mod expressions;

/// Stable source ownership, independent of semantic-check visitation order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Owner {
    Root,
    Routine(BindingId),
    Closure(BindingId),
}

/// Immutable source body and its persistent declaration/capture identity.
pub(super) struct Routine<'a> {
    pub body: &'a FuncBody,
    pub params: &'a [FormalParam],
    pub requires: &'a [fpas_parser::RequiresClause],
    pub ensures: &'a [fpas_parser::EnsuresClause],
    pub capture_key: usize,
    pub export_name: Option<String>,
}

/// One value's owner and source initializer, with parameters represented symbolically.
pub(super) struct Binding<'a> {
    pub name: String,
    pub owner: Owner,
    pub initializer: Option<&'a Expr>,
    pub mutable: bool,
    pub parameter: Option<usize>,
    pub startup: bool,
}

/// Source ownership tables; semantic references provide all name resolution.
pub(super) struct Model<'a> {
    pub routines: BTreeMap<Owner, Routine<'a>>,
    pub bindings: BTreeMap<BindingId, Binding<'a>>,
    pub methods: BTreeMap<String, Owner>,
}

impl<'a> Model<'a> {
    /// Collect named and anonymous bodies, bindings, and retained record defaults.
    pub fn collect(
        declarations: &'a [Decl],
        body: Option<&'a [Stmt]>,
        checker: &'a Checker,
    ) -> Self {
        let mut model = Self {
            routines: BTreeMap::new(),
            bindings: BTreeMap::new(),
            methods: BTreeMap::new(),
        };
        model.declarations(declarations, Owner::Root);
        if let Some(body) = body {
            model.statements(body, Owner::Root);
        }
        for fields in checker.record_defaults.values() {
            for (_, expression) in fields {
                if let Some(expression) = expression {
                    model.expression(expression, Owner::Root);
                }
            }
        }
        model
    }

    fn add_routine(
        &mut self,
        name: &str,
        routine: Routine<'a>,
        span: Span,
        parent: Owner,
        public: bool,
    ) {
        let owner = Owner::Routine(identity(span));
        self.routines.insert(
            owner,
            Routine {
                export_name: (parent == Owner::Root && public).then(|| name.to_owned()),
                ..routine
            },
        );
        for (index, param) in routine.params.iter().enumerate() {
            self.bindings.insert(
                identity(param.span),
                Binding {
                    name: param.name.clone(),
                    owner,
                    initializer: None,
                    mutable: param.mode == fpas_parser::ParamMode::Var,
                    parameter: Some(index),
                    startup: false,
                },
            );
        }
        self.body(routine.body, owner);
        for clause in routine.requires {
            self.expression(&clause.condition, owner);
        }
        for clause in routine.ensures {
            self.expression(&clause.condition, owner);
        }
    }

    fn declarations(&mut self, declarations: &'a [Decl], owner: Owner) {
        for declaration in declarations {
            match declaration {
                Decl::Const(value) => self.binding(
                    &value.name,
                    &value.value,
                    value.span,
                    false,
                    owner,
                    owner == Owner::Root,
                ),
                Decl::Var(value) => self.binding(
                    &value.name,
                    &value.value,
                    value.span,
                    true,
                    owner,
                    owner == Owner::Root,
                ),
                Decl::Function(f) => self.add_routine(
                    &f.name,
                    Routine {
                        body: &f.body,
                        params: &f.params,
                        requires: &f.requires,
                        ensures: &f.ensures,
                        capture_key: crate::function_decl_lookup_key(f),
                        export_name: None,
                    },
                    f.span,
                    owner,
                    f.visibility == Visibility::Public,
                ),
                Decl::Procedure(p) => self.add_routine(
                    &p.name,
                    Routine {
                        body: &p.body,
                        params: &p.params,
                        requires: &p.requires,
                        ensures: &p.ensures,
                        capture_key: crate::procedure_decl_lookup_key(p),
                        export_name: None,
                    },
                    p.span,
                    owner,
                    p.visibility == Visibility::Public,
                ),
                Decl::TypeDef(definition) => {
                    if let TypeBody::Record(record) = &definition.body {
                        for method in &record.methods {
                            let name = format!("{}.{}", definition.name, method.name());
                            let span = match method {
                                RecordMethod::Function(f) | RecordMethod::StaticFunction(f) => {
                                    self.add_routine(
                                        &name,
                                        Routine {
                                            body: &f.body,
                                            params: &f.params,
                                            requires: &f.requires,
                                            ensures: &f.ensures,
                                            capture_key: crate::function_decl_lookup_key(f),
                                            export_name: None,
                                        },
                                        f.span,
                                        owner,
                                        definition.visibility == Visibility::Public
                                            && f.visibility == Visibility::Public,
                                    );
                                    f.span
                                }
                                RecordMethod::Procedure(p) | RecordMethod::StaticProcedure(p) => {
                                    self.add_routine(
                                        &name,
                                        Routine {
                                            body: &p.body,
                                            params: &p.params,
                                            requires: &p.requires,
                                            ensures: &p.ensures,
                                            capture_key: crate::procedure_decl_lookup_key(p),
                                            export_name: None,
                                        },
                                        p.span,
                                        owner,
                                        definition.visibility == Visibility::Public
                                            && p.visibility == Visibility::Public,
                                    );
                                    p.span
                                }
                            };
                            self.methods
                                .insert(name.to_ascii_lowercase(), Owner::Routine(identity(span)));
                        }
                    }
                }
            }
        }
    }

    fn body(&mut self, body: &'a FuncBody, owner: Owner) {
        let FuncBody::Block { nested, stmts } = body;
        self.declarations(nested, owner);
        self.statements(stmts, owner);
    }

    fn binding(
        &mut self,
        name: &str,
        expression: &'a Expr,
        span: Span,
        mutable: bool,
        owner: Owner,
        startup: bool,
    ) {
        self.bindings.insert(
            identity(span),
            Binding {
                name: name.to_owned(),
                owner,
                initializer: Some(expression),
                mutable,
                parameter: None,
                startup,
            },
        );
        self.expression(expression, owner);
    }

    fn statements(&mut self, statements: &'a [Stmt], owner: Owner) {
        for statement in statements {
            self.statement(statement, owner);
        }
    }

    fn statement(&mut self, statement: &'a Stmt, owner: Owner) {
        match statement {
            Stmt::Const(value) | Stmt::Var(value) => self.binding(
                &value.name,
                &value.value,
                value.span,
                matches!(statement, Stmt::Var(_)),
                owner,
                false,
            ),
            Stmt::Block(body, _) | Stmt::Scope { body, .. } => self.statements(body, owner),
            Stmt::Assign { target, value, .. } => {
                self.designator(target, owner);
                self.expression(value, owner);
            }
            Stmt::Return(Some(value), _)
            | Stmt::Panic(value, _)
            | Stmt::Discard { expr: value, .. }
            | Stmt::Expression { expr: value, .. } => self.expression(value, owner),
            Stmt::Call {
                designator, args, ..
            } => {
                self.designator(designator, owner);
                for arg in args {
                    self.expression(arg, owner);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expression(condition, owner);
                self.statement(then_branch, owner);
                if let Some(branch) = else_branch {
                    self.statement(branch, owner);
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                self.expression(expr, owner);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.expression(guard, owner);
                    }
                    self.statement(&arm.body, owner);
                }
                if let Some(body) = else_body {
                    self.statements(body, owner);
                }
            }
            Stmt::For {
                start, end, body, ..
            } => {
                self.expression(start, owner);
                self.expression(end, owner);
                self.statement(body, owner);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.expression(iterable, owner);
                self.statement(body, owner);
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expression(condition, owner);
                self.statement(body, owner);
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.statements(body, owner);
                self.expression(condition, owner);
            }
            _ => {}
        }
    }

    fn designator(&mut self, designator: &'a fpas_parser::Designator, owner: Owner) {
        for part in &designator.parts {
            if let fpas_parser::DesignatorPart::Index(value, _) = part {
                self.expression(value, owner);
            }
        }
    }
}
