use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, Value};
use fpas_parser::Expr;

use super::super::Compiler;

impl Compiler {
    /// Lower scalar, array, and dictionary literal expressions.
    pub(super) fn compile_literal_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(n, span) => {
                self.emit_constant(Value::Integer(*n), Self::location_of(span))?;
            }
            Expr::Real(n, span) => {
                self.emit_constant(Value::Real(*n), Self::location_of(span))?;
            }
            Expr::Str(s, span) => {
                self.emit_constant(Value::Str(s.clone()), Self::location_of(span))?;
            }
            Expr::Bool(b, span) => {
                self.emit_constant(Value::Boolean(*b), Self::location_of(span))?;
            }
            Expr::ArrayLiteral(elems, span) => {
                let location = Self::location_of(span);
                for elem in elems {
                    self.compile_expr(elem)?;
                }
                self.emit(
                    Op::MakeArray(Self::checked_u16(elems.len(), "array elements", *span)?),
                    location,
                );
            }
            Expr::DictLiteral(pairs, span) => {
                let location = Self::location_of(span);
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.emit(
                    Op::MakeDict(Self::checked_u16(pairs.len(), "dict pairs", *span)?),
                    location,
                );
            }
            other => {
                let span = other.span();
                return Err(internal_compiler_error(
                    "Compiler routed a non-literal expression to literal lowering.",
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                ));
            }
        }

        Ok(())
    }
}
