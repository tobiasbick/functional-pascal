use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Intrinsic, Op, SourceLocation, Value};
use fpas_diagnostics::codes::COMPILE_INTRINSIC_ARITY_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};
use fpas_std::std_symbols as s;

use super::super::Compiler;

impl Compiler {
    pub(in super::super) fn compile_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let resolved = self.qualify_name(name);
        let name = if resolved != name {
            resolved.to_string()
        } else {
            name.to_string()
        };
        let name = name.as_str();
        match name {
            s::STD_CONSOLE_WRITE_LN => {
                if args.is_empty() {
                    self.emit_constant(Value::Str(String::new()), location);
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
                return Ok(());
            }
            s::STD_CONSOLE_WRITE => {
                for arg in args {
                    self.compile_expr(arg)?;
                    self.emit(Op::Print, location);
                }
                self.emit(Op::Unit, location);
                return Ok(());
            }
            s::STD_STR_FORMAT => {
                if args.is_empty() {
                    return Err(compile_error(
                        COMPILE_INTRINSIC_ARITY_MISMATCH,
                        "Format requires at least one argument (the template string)",
                        "Use: Format('template %d', Value)",
                        Span {
                            offset: 0,
                            length: 0,
                            line: location.line,
                            column: location.column,
                        },
                    ));
                }
                // Stack layout consumed by StrFormat: template, arg1..argN, N
                self.compile_expr(&args[0])?;
                for arg in &args[1..] {
                    self.compile_expr(arg)?;
                }
                let arg_count = (args.len() - 1) as i64;
                self.emit_constant(Value::Integer(arg_count), location);
                self.emit(Op::Intrinsic(u16::from(Intrinsic::StrFormat)), location);
                return Ok(());
            }
            _ => {}
        }

        if self.compile_std_library_call(name, args, location)? {
            return Ok(());
        }

        if let Some((type_name, variant_info)) = self.find_enum_variant_with_data(name) {
            for arg in args {
                self.compile_expr(arg)?;
            }
            let type_idx = self.chunk.add_constant(Value::Str(type_name));
            let variant_idx = self.chunk.add_constant(Value::Str(variant_info.name));
            self.emit(
                Op::MakeEnum(
                    type_idx,
                    variant_idx,
                    u8::try_from(args.len()).unwrap_or(u8::MAX),
                ),
                location,
            );
            return Ok(());
        }

        if let Some(local_ref) = self.resolve_local(name) {
            for arg in args {
                self.compile_expr(arg)?;
            }
            match local_ref {
                super::super::LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
                super::super::LocalRef::Enclosing(depth, slot) => {
                    self.emit(Op::GetEnclosing(depth, slot), location)
                }
            };
            let arity = u8::try_from(args.len()).unwrap_or(u8::MAX);
            self.emit(Op::CallValue(arity), location);
            return Ok(());
        }

        for arg in args {
            self.compile_expr(arg)?;
        }
        let name_idx = self.chunk.add_constant(Value::Str(name.into()));
        let arity = u8::try_from(args.len()).unwrap_or(u8::MAX);
        self.emit(Op::Call(name_idx, arity), location);
        Ok(())
    }

    pub(in super::super) fn compile_method_call(
        &mut self,
        designator: &Designator,
        qualified_method: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let receiver = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };
        self.compile_designator_read(&receiver)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        let total_args = u8::try_from(args.len() + 1).unwrap_or(u8::MAX);
        let name_idx = self.chunk.add_constant(Value::Str(qualified_method.into()));
        self.emit(Op::Call(name_idx, total_args), location);
        Ok(())
    }
}
