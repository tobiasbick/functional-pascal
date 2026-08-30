//! Local and shared storage access for executable global slots.

use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use fpas_bytecode::{Intrinsic, IntrinsicOwner, Opcode, Value, VerifiedExecutable};

use super::worker::Worker;

/// Borrowed global slots selected without synchronization for eligible runs.
pub(super) enum GlobalSlots<'a> {
    Local(&'a [Option<Value>]),
    Shared(RwLockReadGuard<'a, Vec<Option<Value>>>),
}

impl Deref for GlobalSlots<'_> {
    type Target = [Option<Value>];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Local(slots) => slots,
            Self::Shared(slots) => slots,
        }
    }
}

/// Mutably borrowed global slots selected without synchronization for eligible runs.
pub(super) enum GlobalSlotsMut<'a> {
    Local(&'a mut [Option<Value>]),
    Shared(RwLockWriteGuard<'a, Vec<Option<Value>>>),
}

impl Deref for GlobalSlotsMut<'_> {
    type Target = [Option<Value>];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Local(slots) => slots,
            Self::Shared(slots) => slots,
        }
    }
}

impl DerefMut for GlobalSlotsMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Local(slots) => slots,
            Self::Shared(slots) => slots,
        }
    }
}

impl Worker {
    #[inline(always)]
    /// Borrow the active global slots for a read.
    pub(super) fn global_slots(&self) -> GlobalSlots<'_> {
        if let Some(slots) = self.local_globals.as_deref() {
            GlobalSlots::Local(slots)
        } else {
            GlobalSlots::Shared(
                self.globals
                    .read()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        }
    }

    #[inline(always)]
    /// Borrow the active global slots for a write.
    pub(super) fn global_slots_mut(&mut self) -> GlobalSlotsMut<'_> {
        if let Some(slots) = self.local_globals.as_deref_mut() {
            GlobalSlotsMut::Local(slots)
        } else {
            GlobalSlotsMut::Shared(
                self.globals
                    .write()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        }
    }

    /// Enable worker-owned slots for a proven single-worker execution.
    pub(super) fn with_local_globals(mut self, enabled: bool) -> Self {
        if enabled {
            self.local_globals = Some(vec![None; self.executable.executable().globals.len()]);
        }
        self
    }
}

/// Return whether normal execution can keep globals worker-local.
pub(super) fn can_use_local_globals(executable: &VerifiedExecutable) -> bool {
    let image = executable.executable();
    !image
        .functions
        .iter()
        .any(|function| function.flags.uses_spawn_tasks)
        && !image.code.iter().any(|instruction| {
            instruction.opcode().is_ok_and(|opcode| {
                opcode == Opcode::Intrinsic
                    && Intrinsic::from_u16(instruction.abc_payload().b)
                        .is_some_and(|intrinsic| intrinsic.owner() == IntrinsicOwner::Callback)
            })
        })
}
