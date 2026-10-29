//! Borrowed register-window execution for non-hosted standard-library intrinsics.

mod unary;

use crate::{
    AbcOperands, ConsoleIntrinsic, Intrinsic, IntrinsicOwner, NO_REGISTER, NetIntrinsic,
    ProcIntrinsic, Register, SourceLocation, Value,
};

use super::scalar::register;
use crate::execution::vm::hosted::callbacks::CallbackOutcome;
use crate::execution::vm::worker::Worker;
use crate::execution::vm::{VmError, diagnostics};

impl Worker {
    /// Bytecode dispatch may enter callback frames; immediate host calls finish synchronously.
    pub(in crate::execution::vm) fn execute_intrinsic<const INLINE_CALLBACKS: bool>(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        if operands.auxiliary == 1 && self.execute_unary_intrinsic(operands)? {
            return Ok(());
        }
        let intrinsic = Intrinsic::from_u16(operands.b).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified intrinsic identifier {} is unknown", operands.b),
            )
        })?;
        self.check_scheduling_effect(intrinsic)?;
        let owner = intrinsic.owner();
        let start = self
            .base
            .checked_add(usize::from(operands.c))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        let end = start
            .checked_add(usize::from(operands.auxiliary))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        if self.registers.get(start..end).is_none() {
            return Err(self.intrinsic_window_error(operands));
        }
        let location = self.intrinsic_location();
        let destination = (operands.a != NO_REGISTER)
            .then(|| register(operands.a))
            .transpose()?;
        if self.debug_tasks
            && INLINE_CALLBACKS
            && intrinsic == Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout)
        {
            let arguments = self.registers[start..end].to_vec();
            if let Some(value) = self.debug_console_wait(&arguments, destination, location)?
                && let Some(destination) = destination
            {
                self.write(destination, value)?;
            }
            return Ok(());
        }
        if INLINE_CALLBACKS
            && !self.capture_callback
            && (self.debug_tasks || self.task_id != 0)
            && let Intrinsic::Acp(operation) = intrinsic
            && crate::execution::vm::hosted::acp::native::supported(operation)
        {
            let arguments = self.registers[start..end].to_vec();
            self.start_acp_native(operation, &arguments, destination)?;
            if !self.debug_tasks && self.task_suspension.is_some() {
                self.park_pool_suspension()?;
            }
            return Ok(());
        }
        if INLINE_CALLBACKS
            && !self.capture_callback
            && (self.debug_tasks || self.task_id != 0)
            && self.debug_schedule.is_none()
            && crate::execution::vm::hosted::console_io::supported_intrinsic(intrinsic)
        {
            let arguments = self.registers[start..end].to_vec();
            self.start_console_io(intrinsic, &arguments, destination)?;
            if !self.debug_tasks && self.task_suspension.is_some() {
                self.park_pool_suspension()?;
            }
            return Ok(());
        }
        if INLINE_CALLBACKS
            && !self.capture_callback
            && (self.debug_tasks || self.task_id != 0)
            && let Intrinsic::Net(operation) = intrinsic
            && matches!(
                operation,
                NetIntrinsic::Accept
                    | NetIntrinsic::Close
                    | NetIntrinsic::Listen
                    | NetIntrinsic::ListenTls
                    | NetIntrinsic::SetTimeout
                    | NetIntrinsic::AcceptWithCancellation
                    | NetIntrinsic::Connect
                    | NetIntrinsic::ConnectTls
                    | NetIntrinsic::ConnectWithCancellation
                    | NetIntrinsic::ConnectTlsWithCancellation
                    | NetIntrinsic::Read
                    | NetIntrinsic::ReadWithCancellation
                    | NetIntrinsic::Write
                    | NetIntrinsic::WriteWithCancellation
            )
        {
            let arguments = self.registers[start..end].to_vec();
            if let Some(value) = self.start_net_wait(operation, &arguments, destination)?
                && let Some(destination) = destination
            {
                self.write(destination, value)?;
            }
            if !self.debug_tasks && self.task_suspension.is_some() {
                self.park_pool_suspension()?;
            }
            return Ok(());
        }
        if INLINE_CALLBACKS
            && !self.capture_callback
            && (self.debug_tasks || self.task_id != 0)
            && let Intrinsic::Proc(operation) = intrinsic
            && matches!(
                operation,
                ProcIntrinsic::Run
                    | ProcIntrinsic::Start
                    | ProcIntrinsic::Close
                    | ProcIntrinsic::RunInteractive
                    | ProcIntrinsic::RunCapture
                    | ProcIntrinsic::RunWithOptions
                    | ProcIntrinsic::RunCaptureWithOptions
            )
        {
            let arguments = self.registers[start..end].to_vec();
            if let Some(value) = self.start_proc_wait(operation, &arguments, destination)?
                && let Some(destination) = destination
            {
                self.write(destination, value)?;
            }
            if !self.debug_tasks && self.task_suspension.is_some() {
                self.park_pool_suspension()?;
            }
            return Ok(());
        }
        if INLINE_CALLBACKS
            && !self.capture_callback
            && (self.debug_tasks || self.task_id != 0)
            && let Intrinsic::Fs(operation) = intrinsic
        {
            let arguments = self.registers[start..end].to_vec();
            if let Some(value) = self.start_fs_wait(operation, &arguments, destination)?
                && let Some(destination) = destination
            {
                self.write(destination, value)?;
            }
            if !self.debug_tasks && self.task_suspension.is_some() {
                self.park_pool_suspension()?;
            }
            return Ok(());
        }
        if intrinsic.requires_mutable_dispatch() {
            let arguments = self.registers[start..end].to_vec();
            if let Some(result) = self.task_intrinsic(intrinsic, &arguments, destination)? {
                if let (Some(value), Some(destination)) = (result, destination) {
                    self.write(destination, value)?;
                }
                return Ok(());
            }
            let result = self.execute_borrowed_intrinsic(intrinsic, owner, &arguments, location)?;
            return self.store_intrinsic_result(intrinsic, destination, result);
        }
        if INLINE_CALLBACKS && owner == IntrinsicOwner::Callback {
            let arguments = self.registers[start..end].to_vec();
            let absolute_destination =
                destination.map(|register| self.base + usize::from(register.get()));
            if let Some(outcome) = self.execute_callback_intrinsic(
                intrinsic,
                &arguments,
                location,
                absolute_destination,
            )? {
                return match outcome {
                    CallbackOutcome::Complete(value) => {
                        self.store_intrinsic_result(intrinsic, destination, Some(value))
                    }
                    CallbackOutcome::Deferred => Ok(()),
                };
            }
            return Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Callback intrinsic {intrinsic:?} has no resumable implementation"),
            ));
        }
        let result = self.execute_borrowed_intrinsic(
            intrinsic,
            owner,
            &self.registers[start..end],
            location,
        )?;
        self.store_intrinsic_result(intrinsic, destination, result)
    }

    pub(in crate::execution::vm) fn execute_debug_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
    ) -> Result<Value, VmError> {
        let access = self.execution_access()?;
        let result = self.execute_debug_intrinsic_owned(intrinsic, arguments);
        let extra = match &result {
            Ok(value) => std::slice::from_ref(value),
            Err(_) => &[],
        };
        if self.publish_execution(extra).is_ok() {
            access.safe_point();
        }
        result
    }

    fn execute_debug_intrinsic_owned(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
    ) -> Result<Value, VmError> {
        let base = 1_usize;
        let required = base.saturating_add(arguments.len());
        self.base = 0;
        self.reset_registers(required.max(1));
        for (offset, value) in arguments.iter().enumerate() {
            self.store_register(base + offset, value.clone())?;
        }
        self.execute_intrinsic::<false>(AbcOperands {
            a: 0,
            b: intrinsic.into(),
            c: u16::try_from(base).map_err(|_| {
                self.intrinsic_window_error(AbcOperands {
                    a: 0,
                    b: intrinsic.into(),
                    c: u16::MAX,
                    auxiliary: 0,
                })
            })?,
            auxiliary: u8::try_from(arguments.len()).map_err(|_| {
                self.intrinsic_window_error(AbcOperands {
                    a: 0,
                    b: intrinsic.into(),
                    c: 1,
                    auxiliary: u8::MAX,
                })
            })?,
        })?;
        Ok(self.registers[0].clone())
    }

    fn execute_borrowed_intrinsic(
        &self,
        intrinsic: Intrinsic,
        owner: IntrinsicOwner,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        match owner {
            IntrinsicOwner::Standard => crate::stdlib::run_intrinsic_borrowed(
                intrinsic,
                arguments,
                location,
                self.layouts.as_ref(),
            ),
            IntrinsicOwner::Hosted => self.execute_hosted_intrinsic(intrinsic, arguments, location),
            IntrinsicOwner::Callback => self
                .execute_callback_intrinsic_sync(intrinsic, arguments, location)?
                .ok_or_else(|| {
                    diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        format!(
                            "Callback intrinsic {intrinsic:?} was not handled by its owning module"
                        ),
                    )
                }),
            IntrinsicOwner::Task => Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Task intrinsic {intrinsic:?} bypassed mutable dispatch"),
            )),
        }
    }

    pub(in crate::execution::vm) fn store_intrinsic_result(
        &mut self,
        intrinsic: Intrinsic,
        destination: Option<Register>,
        result: Option<Value>,
    ) -> Result<(), VmError> {
        let Some(destination) = destination else {
            return Ok(());
        };
        let value = result.ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Intrinsic {intrinsic:?} did not produce its verified result"),
            )
        })?;
        self.write(destination, value)
    }

    pub(in crate::execution::vm) fn intrinsic_location(&self) -> SourceLocation {
        self.executable
            .executable()
            .source_map
            .lookup(self.current_address)
            .map_or_else(
                || SourceLocation::new(1, 1),
                |run| SourceLocation::new_with_source(run.line, run.column, run.source.get()),
            )
    }

    fn intrinsic_window_error(&self, operands: AbcOperands) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!(
                "Intrinsic register window {}..+{} is outside the current frame",
                operands.c, operands.auxiliary
            ),
        )
    }
}
