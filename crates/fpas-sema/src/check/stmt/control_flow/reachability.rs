//! Normal and abrupt control-flow edges used by callable capability analysis.

use super::super::super::Checker;
use fpas_parser::Stmt;

#[derive(Clone, Copy, Default)]
struct Exits {
    normal: bool,
    breaks: bool,
    continues: bool,
}

impl Exits {
    fn join(self, other: Self) -> Self {
        Self {
            normal: self.normal || other.normal,
            breaks: self.breaks || other.breaks,
            continues: self.continues || other.continues,
        }
    }

    fn sequence(stmts: &[Stmt]) -> Self {
        let mut exits = Self {
            normal: true,
            ..Self::default()
        };
        for stmt in stmts {
            if !exits.normal {
                break;
            }
            let next = Self::statement(stmt);
            exits.normal = next.normal;
            exits.breaks |= next.breaks;
            exits.continues |= next.continues;
        }
        exits
    }

    fn statement(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::Return(..) | Stmt::Panic(..) => Self::default(),
            Stmt::Break(_) => Self {
                breaks: true,
                ..Self::default()
            },
            Stmt::Continue(_) => Self {
                continues: true,
                ..Self::default()
            },
            Stmt::Block(body, _) | Stmt::Scope { body, .. } => Self::sequence(body),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => Self::statement(then_branch).join(else_branch.as_deref().map_or(
                Self {
                    normal: true,
                    ..Self::default()
                },
                Self::statement,
            )),
            Stmt::Case {
                arms, else_body, ..
            } => {
                // Valid cases without else must already cover their finite domain.
                let arms = arms.iter().map(|arm| Self::statement(&arm.body));
                arms.chain(else_body.iter().map(|body| Self::sequence(body)))
                    .reduce(Self::join)
                    .unwrap_or(Self {
                        normal: true,
                        ..Self::default()
                    })
            }
            Stmt::Repeat { body, .. } => {
                let body = Self::sequence(body);
                Self {
                    normal: body.normal || body.breaks || body.continues,
                    ..Self::default()
                }
            }
            // These loops may execute zero times, and consume their own abrupt edges.
            Stmt::While { .. } | Stmt::For { .. } | Stmt::ForIn { .. } => Self {
                normal: true,
                ..Self::default()
            },
            _ => Self {
                normal: true,
                ..Self::default()
            },
        }
    }
}

impl Checker {
    pub(crate) fn stmt_falls_through(stmt: &Stmt) -> bool {
        Exits::statement(stmt).normal
    }

    pub(crate) fn stmts_fall_through(stmts: &[Stmt]) -> bool {
        Exits::sequence(stmts).normal
    }

    /// Preserve ordinary type diagnostics in unreachable tails without adding flow edges.
    pub(crate) fn check_stmt_sequence(&mut self, stmts: &[Stmt]) {
        let mut reachable = true;
        for stmt in stmts {
            if reachable {
                self.check_stmt(stmt);
                reachable = Self::stmt_falls_through(stmt);
            } else {
                self.check_unreachable(|checker| checker.check_stmt(stmt));
            }
        }
    }

    /// Type-check unreachable source without adding writes, exits or implicit returns.
    pub(crate) fn check_unreachable<R>(&mut self, check: impl FnOnce(&mut Self) -> R) -> R {
        let capabilities = self.scopes.snapshot_capabilities();
        let loops = self.scopes.take_capability_loops();
        let returns = self.snapshot_return_capability();
        let result = check(self);
        self.scopes.restore_capabilities(&capabilities);
        self.scopes.restore_capability_loops(loops);
        self.restore_return_capability(returns);
        result
    }
}
