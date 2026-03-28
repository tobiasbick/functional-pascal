use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_parser::{CaseArm, Expr, Stmt};
use fpas_sema::Ty;

impl Compiler {
    pub(in super::super) fn compile_case_stmt(
        &mut self,
        expr: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        let case_ty = self.ty_of(expr);
        let is_data_enum = matches!(&case_ty, Ty::Enum(enum_ty) if enum_ty.has_data());

        self.compile_expr(expr)?;
        let case_slot = self.next_slot;
        self.begin_scope();
        self.add_local("__case_val");

        if is_data_enum {
            let enum_type_name = match &case_ty {
                Ty::Enum(e) => &e.name,
                _ => unreachable!(),
            };
            self.compile_case_data_enum(arms, else_body, case_slot, enum_type_name, line, column)?;
        } else {
            self.compile_case_scalar(arms, else_body, case_slot, &case_ty, line, column)?;
        }

        self.end_scope((line, column));
        Ok(())
    }
}
