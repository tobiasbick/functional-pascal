use super::super::super::{Compiler, canonical_name};
use crate::error::CompileError;
use fpas_bytecode::SourceLocation;
use fpas_parser::{CaseArm, Expr, Stmt};
use fpas_sema::Ty;

impl Compiler {
    pub(in super::super) fn compile_case_stmt(
        &mut self,
        expr: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let case_ty = self.ty_of(expr);
        let enum_type_name = match &case_ty {
            Ty::Enum(enum_ty) if enum_ty.has_data() => Some(enum_ty.name.as_str()),
            Ty::Named(name)
                if self
                    .enums
                    .get(&canonical_name(name))
                    .is_some_and(|enum_ty| enum_ty.has_data) =>
            {
                Some(name.as_str())
            }
            _ => None,
        };

        self.compile_expr(expr)?;
        let case_slot = self.next_slot;
        self.begin_scope();
        self.add_local("__case_val", location)?;

        if let Some(enum_type_name) = enum_type_name {
            self.compile_case_data_enum(arms, else_body, case_slot, enum_type_name, location)?;
        } else {
            self.compile_case_scalar(arms, else_body, case_slot, &case_ty, location)?;
        }

        self.end_scope(location);
        Ok(())
    }
}
