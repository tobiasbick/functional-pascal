use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, Op, SourceLocation, Value};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` input and output calls.
    pub(super) fn compile_console_io_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONSOLE_WRITE_LN => {
                if args.is_empty() {
                    self.emit_constant(Value::Str(String::new()), location)?;
                    self.emit(Op::PrintLn, location);
                } else {
                    for (index, arg) in args.iter().enumerate() {
                        self.compile_expr(arg)?;
                        if index + 1 == args.len() {
                            self.emit(Op::PrintLn, location);
                        } else {
                            self.emit(Op::Print, location);
                        }
                    }
                }
                self.emit(Op::Unit, location);
                Ok(true)
            }
            s::STD_CONSOLE_WRITE => {
                for arg in args {
                    self.compile_expr(arg)?;
                    self.emit(Op::Print, location);
                }
                self.emit(Op::Unit, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_LN => {
                self.expect_zero_args(s::STD_CONSOLE_READ_LN, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadLn), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ => {
                self.expect_zero_args(s::STD_CONSOLE_READ, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::Read), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadKey), location);
                Ok(true)
            }
            s::STD_CONSOLE_KEY_PRESSED => {
                self.expect_zero_args(s::STD_CONSOLE_KEY_PRESSED, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::KeyPressed), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadKeyEvent), location);
                Ok(true)
            }
            s::STD_CONSOLE_EVENT_PENDING => {
                self.expect_zero_args(s::STD_CONSOLE_EVENT_PENDING, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::EventPending), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadEvent), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT_TIMEOUT => {
                self.expect_exact_args(s::STD_CONSOLE_READ_EVENT_TIMEOUT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(
                    Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_POLL_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_POLL_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::PollEvent), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
