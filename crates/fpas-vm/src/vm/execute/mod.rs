mod aggregates;
mod closure;
mod concurrency;
mod control_calls;
mod enums;
pub(crate) mod io;
mod numeric;
mod result_option;
mod stack_scope;
mod transition;

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;
use transition::{ExecutionContext, ExecutionTransition};

/// Outcome of executing a single instruction via [`Worker::exec_one`].
pub(super) enum StepResult {
    /// Normal instruction executed; continue.
    Continue,
    /// `Op::Return` was decoded — caller must handle stack frame.
    Return,
    /// `Op::Halt` was decoded.
    Halt,
    /// The current spawned task was saved in a cooperative wait queue.
    Suspended,
}

impl Worker {
    /// Fetch, decode and execute the next instruction.
    ///
    /// Dispatch is a single top-level `match` on [`Op`] so hot loops do not walk
    /// cascaded `try_exec_*` handlers. Category helpers still own the opcode
    /// bodies. `Return`/`Halt` are signalled via [`StepResult`]; `Panic` returns
    /// `Err`.
    pub(super) fn exec_one(
        &mut self,
        fallback_line: SourceLocation,
    ) -> Result<StepResult, VmError> {
        let ip = self.ip;
        let op = self.shared.chunk.code()[ip];
        // `code` and `locations` stay the same length for every emitted chunk.
        let line = self
            .shared
            .chunk
            .locations()
            .get(ip)
            .copied()
            .unwrap_or(fallback_line);
        self.current_location = line;
        self.ip = ip + 1;

        let handled = match op {
            Op::Constant(_)
            | Op::Unit
            | Op::Pop
            | Op::Dup
            | Op::GetLocal(_)
            | Op::SetLocal(_)
            | Op::SetLocalPop(_)
            | Op::IncLocal(_)
            | Op::DecLocal(_)
            | Op::GetGlobal(_)
            | Op::SetGlobal(_)
            | Op::GetEnclosing(_, _)
            | Op::SetEnclosing(_, _) => self.try_exec_stack_scope(op, line)?,

            Op::AddInt
            | Op::SubInt
            | Op::MulInt
            | Op::DivInt
            | Op::ModInt
            | Op::NegateInt
            | Op::Shl
            | Op::Shr
            | Op::IntToReal => self.try_exec_int_ops(op, line)?,

            Op::AddReal | Op::SubReal | Op::MulReal | Op::DivReal | Op::NegateReal => {
                self.try_exec_real_ops(op, line)?
            }

            Op::ConcatStr
            | Op::EqInt
            | Op::NeqInt
            | Op::LtInt
            | Op::GtInt
            | Op::LeInt
            | Op::GeInt
            | Op::EqReal
            | Op::NeqReal
            | Op::LtReal
            | Op::GtReal
            | Op::LeReal
            | Op::GeReal
            | Op::EqStr
            | Op::NeqStr
            | Op::LtStr
            | Op::GtStr
            | Op::LeStr
            | Op::GeStr => self.try_exec_comparisons(op, line)?,

            Op::BitAnd
            | Op::BitOr
            | Op::BitXor
            | Op::EqBool
            | Op::NeqBool
            | Op::Not
            | Op::And
            | Op::Or => self.try_exec_bitwise_bool(op, line)?,

            Op::AddDyn
            | Op::SubDyn
            | Op::MulDyn
            | Op::DivDyn
            | Op::NegateDyn
            | Op::EqDyn
            | Op::NeqDyn
            | Op::LtDyn
            | Op::GtDyn
            | Op::LeDyn
            | Op::GeDyn => self.try_exec_dynamic_ops(op, line)?,

            Op::Jump(_)
            | Op::JumpIfFalse(_)
            | Op::JumpIfTrue(_)
            | Op::JumpIfLocalGt(_, _, _)
            | Op::JumpIfLocalLt(_, _, _)
            | Op::Call(_, _)
            | Op::CallValue(_) => self.try_exec_control_calls(op, line)?,

            Op::MakeClosure(_, _) | Op::MakeCell | Op::CellGet | Op::CellSet => {
                self.try_exec_closure_ops(op, line)?
            }

            Op::SpawnTask(_) | Op::SpawnDetachedTask(_) | Op::Yield => {
                self.try_exec_concurrency(op, line)?
            }

            Op::MakeArray(_)
            | Op::MakeDict(_)
            | Op::IndexGet
            | Op::IndexSet
            | Op::GlobalIndexSet(_, _)
            | Op::Contains
            | Op::MakeRecord(_, _)
            | Op::FieldGet(_)
            | Op::FieldSet(_)
            | Op::UpdateRecord(_)
            | Op::ArrayPushLocal(_, _)
            | Op::ArrayPopLocal(_, _) => self.try_exec_aggregates(op, line)?,

            Op::MakeOk
            | Op::MakeErr
            | Op::MakeSome
            | Op::MakeNone
            | Op::IsResultOk
            | Op::IsOptionSome
            | Op::UnwrapOk
            | Op::UnwrapErr
            | Op::UnwrapSome => self.try_exec_result_option(op, line)?,

            Op::MakeEnum(_, _, _) | Op::IsVariant(_, _) | Op::EnumField(_) => {
                self.try_exec_enums(op, line)?
            }

            Op::Print | Op::PrintLn | Op::Intrinsic(_) => self.try_exec_io(op, line)?,

            Op::Return => return Ok(StepResult::Return),
            Op::Halt => return Ok(StepResult::Halt),
            Op::Panic => {
                let val = self.pop(line)?;
                return Err(runtime_error(
                    RUNTIME_PROGRAM_PANIC,
                    format!("panic: {val}"),
                    "Remove the panic or guard the failing condition before calling panic.",
                    line,
                ));
            }
        };

        if !handled {
            return Err(internal_error(
                format!("Unhandled opcode in VM dispatcher: {op:?}"),
                "This indicates a VM dispatch bug. Please report it.",
                line,
            ));
        }

        if self.task_suspended {
            return Ok(StepResult::Suspended);
        }
        self.maybe_timeslice_yield();
        Ok(StepResult::Continue)
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            let context = if self.current_task_id == 0 {
                ExecutionContext::Main
            } else {
                ExecutionContext::SpawnedTask
            };
            match self.advance_execution(context, self.current_location)? {
                ExecutionTransition::Continue => {}
                ExecutionTransition::Cancelled => {
                    if self.current_task_retain_result {
                        self.shared.cancel_retained_task(self.current_task_id);
                    }
                    return Ok(());
                }
                ExecutionTransition::Suspended => {
                    self.task_suspended = false;
                    if !self.pick_next_task() {
                        return Ok(());
                    }
                }
                ExecutionTransition::Completed(return_value) => {
                    if self.current_task_id == 0 {
                        return Ok(());
                    }
                    if self.current_task_retain_result {
                        self.shared
                            .store_task_result(self.current_task_id, return_value);
                    }
                    if !self.pick_next_task() {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn pick_next_task(&mut self) -> bool {
        if let Some(task) = self.shared.try_dequeue_task() {
            self.load_task(task);
            true
        } else {
            false
        }
    }
}
