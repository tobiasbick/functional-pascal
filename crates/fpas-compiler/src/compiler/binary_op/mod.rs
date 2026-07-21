//! Binary operator lowering for arithmetic, comparisons, and string concatenation.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` (from the repository root).

mod arithmetic;
mod comparisons;

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{BinaryOp, Expr};
use fpas_sema::Ty;

use super::Compiler;

pub(super) fn is_generic_param(ty: &Ty) -> bool {
    matches!(ty, Ty::GenericParam(..))
}

impl Compiler {
    /// Return the semantic type of an expression or an internal compile error.
    pub(super) fn ty_of(&self, expr: &Expr) -> Result<Ty, CompileError> {
        let key = fpas_sema::expr_lookup_key(expr);
        if let Some(ty) = self.expr_types.get(&key) {
            return Ok(ty.clone());
        }
        // Fallback for constant sub-expressions that were cloned (e.g. record field
        // defaults copied into RecordDefaultsMap lose their type-map entries because
        // the pointer identity changes on clone).
        self.infer_const_ty(expr).ok_or_else(|| {
            let span = expr.span();
            internal_compiler_error(
                "Expression type is missing after semantic analysis.",
                "This is an internal compiler error. Re-run compilation and report the source program.",
                span.line,
                span.column,
            )
        })
    }

    /// Infer the type of a constant expression without consulting the type map.
    ///
    /// Only handles the subset of expressions that can legitimately appear as record
    /// field default values: literals, paren-wrapped literals, and unary negation of
    /// numeric literals.
    fn infer_const_ty(&self, expr: &Expr) -> Option<Ty> {
        match expr {
            Expr::Integer(_, _) => Some(Ty::Integer),
            Expr::Real(_, _) => Some(Ty::Real),
            Expr::Str(_, _) => Some(Ty::String),
            Expr::Bool(_, _) => Some(Ty::Boolean),
            Expr::UnaryOp { operand, .. } => self.infer_const_ty(operand),
            Expr::Paren(inner, _) => self.infer_const_ty(inner),
            _ => None,
        }
    }

    fn maybe_int_to_real_for_ty(&mut self, ty: &Ty, location: SourceLocation) {
        if *ty == Ty::Integer {
            self.emit(Op::IntToReal, location);
        }
    }

    fn emit_numeric_binary(
        &mut self,
        int_op: Op,
        real_op: Op,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;
        let use_real = *lt == Ty::Real || *rt == Ty::Real;
        self.compile_expr(left)?;
        if use_real {
            self.maybe_int_to_real_for_ty(lt, location);
        }
        self.compile_expr(right)?;
        if use_real {
            self.maybe_int_to_real_for_ty(rt, location);
        }
        self.emit(if use_real { real_op } else { int_op }, location);
        Ok(())
    }

    fn emit_string_binary(
        &mut self,
        left: &Expr,
        right: &Expr,
        _operand_types: (&Ty, &Ty),
        op: Op,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;
        self.emit(op, location);
        Ok(())
    }

    pub(super) fn compile_binary_op(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let lt = self.ty_of(left)?;
        let rt = self.ty_of(right)?;
        let operand_types = (&lt, &rt);

        match op {
            BinaryOp::Add => self.compile_add(left, right, operand_types, location),
            BinaryOp::Sub | BinaryOp::Mul => {
                self.compile_numeric_arithmetic(op, left, right, operand_types, location)
            }
            BinaryOp::RealDiv => self.compile_real_div(left, right, operand_types, location),
            BinaryOp::IntDiv | BinaryOp::Mod => self.compile_direct_binary(
                if op == BinaryOp::IntDiv {
                    Op::DivInt
                } else {
                    Op::ModInt
                },
                left,
                right,
                location,
            ),
            BinaryOp::And | BinaryOp::Or => self.compile_direct_binary(
                if op == BinaryOp::And { Op::And } else { Op::Or },
                left,
                right,
                location,
            ),
            BinaryOp::Xor => self.compile_direct_binary(Op::BitXor, left, right, location),
            BinaryOp::Shl | BinaryOp::Shr => self.compile_direct_binary(
                if op == BinaryOp::Shl {
                    Op::Shl
                } else {
                    Op::Shr
                },
                left,
                right,
                location,
            ),
            BinaryOp::Eq | BinaryOp::NotEq => {
                self.compile_equality(op, left, right, operand_types, location)
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                self.compile_ordering(op, left, right, operand_types, location)
            }
            BinaryOp::In => self.compile_direct_binary(Op::Contains, left, right, location),
        }
    }
}
