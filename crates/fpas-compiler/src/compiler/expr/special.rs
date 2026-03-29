use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, FormalParam, FuncBody};

use super::super::Compiler;

impl Compiler {
    pub(super) fn compile_try_expr(
        &mut self,
        inner: &Expr,
        location: (u32, u32),
    ) -> Result<(), CompileError> {
        self.compile_expr(inner)?;
        self.emit(Op::Dup, location);
        self.emit(Op::IsResultOk, location);
        let jump_ok = self.chunk.code.len();
        self.emit(Op::JumpIfTrue(0), location);
        self.emit(Op::Return, location);

        let ok_addr = self.chunk.code.len() as u32;
        self.patch_jump(jump_ok, ok_addr, location)?;
        self.emit(Op::UnwrapOk, location);
        Ok(())
    }

    /// Compile an anonymous function expression (lambda / closure).
    ///
    /// Generates a unique internal name, compiles the body like a named function,
    /// then pushes a `Value::Function` on the stack. If the body references
    /// enclosing variables, they are captured by value at creation time
    /// (closure).
    ///
    /// **Documentation:** `docs/pascal/04-functions.md`
    pub(in crate::compiler) fn compile_function_expr(
        &mut self,
        params: &[FormalParam],
        body: &FuncBody,
        location: (u32, u32),
    ) -> Result<(), CompileError> {
        let lambda_name = format!("$lambda_{}", self.next_lambda_id);
        self.next_lambda_id += 1;
        let arity = params.len() as u8;

        let jump_over = self.emit(Op::Jump(0), location);

        let code_start = self.chunk.len();
        self.chunk
            .functions
            .insert(lambda_name.clone(), (code_start, arity));

        let saved_locals = std::mem::take(&mut self.locals);
        let saved_next_slot = self.next_slot;
        let saved_scope_depth = self.scope_depth;
        self.next_slot = 0;
        self.scope_depth = 0;
        self.enclosing_locals.push(saved_locals.clone());

        self.begin_scope();

        for param in params {
            self.add_local(&param.name);
        }

        if let FuncBody::Block { nested, stmts } = body {
            for decl in nested {
                self.compile_decl(decl)?;
            }
            for stmt in stmts {
                self.compile_stmt(stmt)?;
            }
        }

        self.emit(Op::Unit, location);
        self.emit(Op::Return, location);

        self.end_scope(location);
        let body_end = self.chunk.len();

        self.enclosing_locals.pop();
        self.locals = saved_locals;
        self.next_slot = saved_next_slot;
        self.scope_depth = saved_scope_depth;

        let mut captures: Vec<(u16, u16)> = Vec::new();
        for index in code_start..body_end {
            match self.chunk.code[index] {
                Op::GetEnclosing(depth, slot) | Op::SetEnclosing(depth, slot) => {
                    let key = (depth, slot);
                    if !captures.contains(&key) {
                        captures.push(key);
                    }
                }
                _ => {}
            }
        }

        if captures.is_empty() {
            let after = self.chunk.len() as u32;
            self.patch_jump(jump_over, after, location)?;

            self.emit_constant(
                Value::Function {
                    name: lambda_name,
                    captures: vec![],
                },
                location,
            );
            return Ok(());
        }

        for index in code_start..body_end {
            let replacement = match self.chunk.code[index] {
                Op::GetEnclosing(depth, slot) => {
                    let Some(capture_index) = captures
                        .iter()
                        .position(|capture| *capture == (depth, slot))
                    else {
                        unreachable!("capture set must include every rewritten enclosing access");
                    };
                    Some(Op::GetLocal(arity as u16 + capture_index as u16))
                }
                Op::SetEnclosing(depth, slot) => {
                    let Some(capture_index) = captures
                        .iter()
                        .position(|capture| *capture == (depth, slot))
                    else {
                        unreachable!("capture set must include every rewritten enclosing access");
                    };
                    Some(Op::SetLocal(arity as u16 + capture_index as u16))
                }
                _ => None,
            };
            if let Some(op) = replacement {
                self.chunk.code[index] = op;
            }
        }

        let after = self.chunk.len() as u32;
        self.patch_jump(jump_over, after, location)?;

        for &(depth, slot) in &captures {
            if depth == 1 {
                self.emit(Op::GetLocal(slot), location);
            } else {
                self.emit(Op::GetEnclosing(depth - 1, slot), location);
            }
        }

        self.emit_constant(
            Value::Function {
                name: lambda_name,
                captures: vec![],
            },
            location,
        );
        self.emit(Op::MakeClosure(captures.len() as u8), location);
        Ok(())
    }
}
