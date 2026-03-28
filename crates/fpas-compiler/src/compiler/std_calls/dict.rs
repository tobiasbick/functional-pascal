use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_dict_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_DICT_LENGTH => {
                self.expect_exact_args(s::STD_DICT_LENGTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::DictLength, location);
                Ok(true)
            }
            s::STD_DICT_CONTAINS_KEY => {
                self.expect_exact_args(s::STD_DICT_CONTAINS_KEY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::DictContainsKey, location);
                Ok(true)
            }
            s::STD_DICT_KEYS => {
                self.expect_exact_args(s::STD_DICT_KEYS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::DictKeys, location);
                Ok(true)
            }
            s::STD_DICT_VALUES => {
                self.expect_exact_args(s::STD_DICT_VALUES, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::DictValues, location);
                Ok(true)
            }
            s::STD_DICT_REMOVE => {
                self.expect_exact_args(s::STD_DICT_REMOVE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::DictRemove, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
