//! FPAS execution, runtime values, memory ownership, and native host operations.

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "runtime fixtures fail fast on invalid test setup"
    )
)]

pub mod profile;
mod value;

/// Native implementations of the standard library.
pub mod stdlib;

/// Virtual machine, host integration and source-level debugging.
pub mod execution;

// Execution clients use the same portable code types as the compiler.
pub use fpas_code::*;
pub use value::{
    CellSnapshot, CellUpdateError, CollectionLimits, CollectionSchedule, CollectionStats,
    DomainAccess, DomainId, DomainMaintenance, DomainPhase, DomainSession, EnumConstructionError,
    EnumValue, ExecutionAccess, ExecutionResume, ExecutionRoots, FunctionValue,
    GraphClassification, MaintenanceDeferral, MaintenanceOutcome, MaintenancePolicy, MemoryDomain,
    OwnedCell, OwnershipError, PublishedWait, RecordValue, ReferenceRoot, ReferenceStep,
    RegisteredRoots, RootKind, RootValues, RuntimeEnumLayout, RuntimeErrorVariant,
    RuntimeRecordLayout, SharedArray, SharedDict, SharedEnum, SharedFunction, SharedRecord,
    SharedReference, SharedStr, TraceRoots, Value, VariableReference, WeakOwnedCell,
    classify_values, defer_maintenance, inspect_values, managed_value_buffer, release_values,
    summarized_transferable, validate_local,
};
