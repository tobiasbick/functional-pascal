use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_channel_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CHANNEL_MAKE => {
                self.expect_zero_args(s::STD_CHANNEL_MAKE, args, location)?;
                self.emit_intrinsic(Intrinsic::ChannelMake, location);
                Ok(true)
            }
            s::STD_CHANNEL_MAKE_BUFFERED => {
                self.expect_exact_args(s::STD_CHANNEL_MAKE_BUFFERED, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ChannelMakeBuffered, location);
                Ok(true)
            }
            s::STD_CHANNEL_SEND => {
                self.expect_exact_args(s::STD_CHANNEL_SEND, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::ChannelSend, location);
                Ok(true)
            }
            s::STD_CHANNEL_RECEIVE => {
                self.expect_exact_args(s::STD_CHANNEL_RECEIVE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ChannelRecv, location);
                Ok(true)
            }
            s::STD_CHANNEL_TRY_RECEIVE => {
                self.expect_exact_args(s::STD_CHANNEL_TRY_RECEIVE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ChannelTryRecv, location);
                Ok(true)
            }
            s::STD_CHANNEL_CLOSE => {
                self.expect_exact_args(s::STD_CHANNEL_CLOSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ChannelClose, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
