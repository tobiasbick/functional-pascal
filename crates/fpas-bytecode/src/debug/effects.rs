//! Shared effect classification for controlled debugger calls.

use crate::{
    ArrayIntrinsic, DictIntrinsic, FunctionId, Intrinsic, Opcode, OptionIntrinsic, ResultIntrinsic,
    VerifiedExecutable,
};

/// Effect categories relevant to detached debugger evaluation.
///
/// Writes are safe only because the debugger executes against a fully detached value graph. Every
/// other category is denied at the controlled-call boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugEffectSet(u16);

impl DebugEffectSet {
    /// No observable effect.
    pub const EMPTY: Self = Self(0);
    /// Mutation of globals, cells, or aggregates inside the detached sandbox.
    pub const SANDBOX_WRITE: Self = Self(1 << 0);
    /// Console, file-system, process, or other externally observable host access.
    pub const HOST_IO: Self = Self(1 << 1);
    /// Time, randomness, arguments, environment, or another nondeterministic observation.
    pub const NONDETERMINISTIC: Self = Self(1 << 2);
    /// An operation that may block the debugger worker.
    pub const BLOCKING: Self = Self(1 << 3);
    /// Task, scheduler, or channel interaction.
    pub const TASK: Self = Self(1 << 4);
    /// A first-class call target that static bytecode metadata cannot identify.
    pub const DYNAMIC_CALL: Self = Self(1 << 5);
    /// An operation whose effect contract is not known.
    pub const UNKNOWN: Self = Self(1 << 6);

    /// Return whether all effects are valid in a detached debugger sandbox.
    #[must_use]
    pub const fn is_debug_safe(self) -> bool {
        self.0 & !Self::SANDBOX_WRITE.0 == 0
    }

    /// Return whether this set includes the requested category.
    #[must_use]
    pub const fn contains(self, category: Self) -> bool {
        self.0 & category.0 == category.0
    }

    /// Combine two effect sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Local and transitive effects for one dense executable function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionEffectSummary {
    /// Dense function identity whose code was analyzed.
    pub function: FunctionId,
    /// Effects caused directly by the function body.
    pub local: DebugEffectSet,
    /// Local effects plus every statically identified direct callee.
    pub transitive: DebugEffectSet,
}

/// Analyze verified bytecode and close direct-call effects to a deterministic fixed point.
#[must_use]
pub fn analyze_debug_effects(executable: &VerifiedExecutable) -> Vec<FunctionEffectSummary> {
    let image = executable.executable();
    let mut local = vec![DebugEffectSet::default(); image.functions.len()];
    let mut edges = vec![Vec::<usize>::new(); image.functions.len()];
    for (index, function) in image.functions.iter().enumerate() {
        if function.flags.uses_spawn_tasks {
            local[index] = local[index].union(DebugEffectSet::TASK);
        }
        for address in function.code.start.get()..function.code.end.get() {
            let Some(instruction) = usize::try_from(address)
                .ok()
                .and_then(|address| image.code.get(address))
            else {
                local[index] = local[index].union(DebugEffectSet::UNKNOWN);
                continue;
            };
            let Ok(opcode) = instruction.opcode() else {
                local[index] = local[index].union(DebugEffectSet::UNKNOWN);
                continue;
            };
            match opcode {
                Opcode::CallDirect => match instruction.abc_operands() {
                    Ok(operands) => edges[index].push(usize::from(operands.b)),
                    Err(_) => local[index] = local[index].union(DebugEffectSet::UNKNOWN),
                },
                Opcode::CallValue => {
                    local[index] = local[index].union(DebugEffectSet::DYNAMIC_CALL);
                }
                Opcode::Intrinsic => match instruction.abc_operands() {
                    Ok(operands) => {
                        let effects = Intrinsic::from_u16(operands.b)
                            .map_or(DebugEffectSet::UNKNOWN, intrinsic_debug_effects);
                        local[index] = local[index].union(effects);
                    }
                    Err(_) => local[index] = local[index].union(DebugEffectSet::UNKNOWN),
                },
                Opcode::CellWrite
                | Opcode::StoreGlobal
                | Opcode::IndexSet
                | Opcode::StoreField
                | Opcode::UpdateRecord
                | Opcode::ArrayPop
                | Opcode::ArrayPush
                | Opcode::StoreGlobalIndexPath => {
                    local[index] = local[index].union(DebugEffectSet::SANDBOX_WRITE);
                }
                Opcode::SpawnTask | Opcode::SpawnDetachedTask | Opcode::Yield => {
                    local[index] = local[index].union(DebugEffectSet::TASK);
                }
                _ => {}
            }
        }
    }
    let mut transitive = local.clone();
    loop {
        let mut changed = false;
        for index in 0..transitive.len() {
            let mut closed = transitive[index];
            for target in &edges[index] {
                closed = closed.union(
                    transitive
                        .get(*target)
                        .copied()
                        .unwrap_or(DebugEffectSet::UNKNOWN),
                );
            }
            if closed != transitive[index] {
                transitive[index] = closed;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    local
        .into_iter()
        .zip(transitive)
        .enumerate()
        .map(|(index, (local, transitive))| FunctionEffectSummary {
            function: FunctionId::new(u16::try_from(index).unwrap_or(u16::MAX)),
            local,
            transitive,
        })
        .collect()
}

/// Classify one stable intrinsic for debugger-call policy enforcement.
#[must_use]
pub const fn intrinsic_debug_effects(intrinsic: Intrinsic) -> DebugEffectSet {
    match intrinsic {
        Intrinsic::Str(_)
        | Intrinsic::Conv(_)
        | Intrinsic::Parse(_)
        | Intrinsic::Math(_)
        | Intrinsic::Path(_)
        | Intrinsic::Json(_)
        | Intrinsic::Toml(_) => DebugEffectSet::EMPTY,
        Intrinsic::Array(operation) => array_effects(operation),
        Intrinsic::Dict(operation) => dict_effects(operation),
        Intrinsic::Result(operation) => result_effects(operation),
        Intrinsic::Option(operation) => option_effects(operation),
        Intrinsic::Args(_) => DebugEffectSet::NONDETERMINISTIC,
        Intrinsic::Random(_) => DebugEffectSet::NONDETERMINISTIC,
        Intrinsic::Env(_) => DebugEffectSet::HOST_IO.union(DebugEffectSet::NONDETERMINISTIC),
        Intrinsic::Fs(_) | Intrinsic::Console(_) | Intrinsic::Test(_) => DebugEffectSet::HOST_IO,
        Intrinsic::Proc(_) => DebugEffectSet::HOST_IO
            .union(DebugEffectSet::BLOCKING)
            .union(DebugEffectSet::NONDETERMINISTIC),
        Intrinsic::Net(_) => DebugEffectSet::HOST_IO
            .union(DebugEffectSet::BLOCKING)
            .union(DebugEffectSet::NONDETERMINISTIC),
        Intrinsic::Http(_) => DebugEffectSet::HOST_IO.union(DebugEffectSet::NONDETERMINISTIC),
        Intrinsic::Task(_) => DebugEffectSet::TASK.union(DebugEffectSet::BLOCKING),
        Intrinsic::Time(_) => DebugEffectSet::NONDETERMINISTIC.union(DebugEffectSet::BLOCKING),
    }
}

const fn array_effects(operation: ArrayIntrinsic) -> DebugEffectSet {
    match operation {
        ArrayIntrinsic::Map
        | ArrayIntrinsic::Filter
        | ArrayIntrinsic::Reduce
        | ArrayIntrinsic::Find
        | ArrayIntrinsic::FindIndex
        | ArrayIntrinsic::Any
        | ArrayIntrinsic::All
        | ArrayIntrinsic::FlatMap
        | ArrayIntrinsic::ForEach => DebugEffectSet::DYNAMIC_CALL,
        _ => DebugEffectSet::EMPTY,
    }
}

const fn dict_effects(operation: DictIntrinsic) -> DebugEffectSet {
    match operation {
        DictIntrinsic::Map | DictIntrinsic::Filter => DebugEffectSet::DYNAMIC_CALL,
        _ => DebugEffectSet::EMPTY,
    }
}

const fn result_effects(operation: ResultIntrinsic) -> DebugEffectSet {
    match operation {
        ResultIntrinsic::Map | ResultIntrinsic::AndThen | ResultIntrinsic::OrElse => {
            DebugEffectSet::DYNAMIC_CALL
        }
        _ => DebugEffectSet::EMPTY,
    }
}

const fn option_effects(operation: OptionIntrinsic) -> DebugEffectSet {
    match operation {
        OptionIntrinsic::Map | OptionIntrinsic::AndThen | OptionIntrinsic::OrElse => {
            DebugEffectSet::DYNAMIC_CALL
        }
        _ => DebugEffectSet::EMPTY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_intrinsic_has_an_explicit_effect_class() {
        for intrinsic in Intrinsic::all() {
            assert!(!intrinsic_debug_effects(intrinsic).contains(DebugEffectSet::UNKNOWN));
        }
    }

    #[test]
    fn representative_intrinsic_policies_are_stable() {
        assert!(
            intrinsic_debug_effects(Intrinsic::Math(crate::MathIntrinsic::Sqrt)).is_debug_safe()
        );
        assert!(
            intrinsic_debug_effects(Intrinsic::Fs(crate::FsIntrinsic::ReadText))
                .contains(DebugEffectSet::HOST_IO)
        );
        assert!(
            intrinsic_debug_effects(Intrinsic::Random(crate::RandomIntrinsic::Random))
                .contains(DebugEffectSet::NONDETERMINISTIC)
        );
        assert!(
            intrinsic_debug_effects(Intrinsic::Task(crate::TaskIntrinsic::Wait))
                .contains(DebugEffectSet::TASK)
        );
    }
}
