//! Checks resource lifetimes and publication guards over reachable bytecode.

use std::collections::{HashMap, VecDeque};

use crate::{DecodedInstruction, Executable, FunctionId, FunctionInfo, InstructionAddress, Opcode};

use super::{ValidationError, ValidationErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Scope {
    Root,
    Resources,
    Tasks,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct State {
    scope: usize,
    // Possible initializer phases at this point. Merges retain every path.
    phase: u8,
}

const CLEAN: u8 = 1;
const UNPROTECTED_CALL: u8 = 2;
const ARMED: u8 = 4;

pub(super) fn validate(
    executable: &Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    decoded: &[DecodedInstruction],
) -> Result<(), ValidationError> {
    let start = function.code.start.get() as usize;
    let end = function.code.end.get() as usize;
    if !decoded[start..end].iter().any(|instruction| {
        matches!(
            instruction.opcode(),
            Opcode::EnterCleanupScope
                | Opcode::BeginResourceInit
                | Opcode::ArmResourceInit
                | Opcode::RegisterCleanup
                | Opcode::ExitCleanupScope
        )
    }) {
        return Ok(());
    }
    let error = |address: usize, reason| {
        ValidationError::instruction(
            executable,
            function_id,
            InstructionAddress::new(address as u32),
            Some(decoded[address].opcode()),
            ValidationErrorKind::CleanupState { reason },
        )
    };
    // Interned parent links bound storage by code size rather than nesting times code size.
    let mut scopes = vec![(0, Scope::Root)];
    let mut intern = HashMap::new();
    let mut states: Vec<Option<State>> = vec![None; end - start];
    let mut queue = VecDeque::from([(start, State::default())]);
    while let Some((address, mut state)) = queue.pop_front() {
        if let Some(previous) = states[address - start] {
            if previous.scope != state.scope || (previous.phase == 0) != (state.phase == 0) {
                return Err(error(
                    address,
                    "control-flow merge has different lifetime or initializer state",
                ));
            }
            let merged = previous.phase | state.phase;
            if merged == previous.phase {
                continue;
            }
            state.phase = merged;
        }
        states[address - start] = Some(state);
        let instruction = decoded[address];
        match instruction.opcode() {
            Opcode::EnterCleanupScope | Opcode::EnterTaskScope => {
                let kind = if instruction.opcode() == Opcode::EnterCleanupScope {
                    Scope::Resources
                } else {
                    Scope::Tasks
                };
                let parent = state.scope;
                state.scope = *intern.entry((parent, kind)).or_insert_with(|| {
                    scopes.push((parent, kind));
                    scopes.len() - 1
                });
            }
            Opcode::BeginResourceInit => {
                if state.phase != 0 || scopes[state.scope].1 != Scope::Resources {
                    return Err(error(
                        address,
                        "resource initializer requires its innermost resource lifetime",
                    ));
                }
                state.phase = CLEAN;
            }
            Opcode::ArmResourceInit => {
                if state.phase == 0 {
                    return Err(error(address, "resource producer has no initializer guard"));
                }
                state.phase = ARMED;
            }
            Opcode::RegisterCleanup => {
                if state.phase == 0 || scopes[state.scope].1 != Scope::Resources {
                    return Err(error(
                        address,
                        "resource publication has no matching initializer and lifetime",
                    ));
                }
                if state.phase & UNPROTECTED_CALL != 0 {
                    return Err(error(
                        address,
                        "resource publication follows an unprotected call",
                    ));
                }
                state.phase = 0;
            }
            Opcode::ExitCleanupScope => {
                if scopes[state.scope].1 != Scope::Resources {
                    return Err(error(
                        address,
                        "resource lifetime exit does not match lexical nesting",
                    ));
                }
                state.scope = scopes[state.scope].0;
                state.phase = 0;
            }
            Opcode::ExitTaskScope | Opcode::CancelTaskScope => {
                if scopes[state.scope].1 != Scope::Tasks {
                    return Err(error(
                        address,
                        "task scope exit crosses a live resource lifetime",
                    ));
                }
                state.scope = scopes[state.scope].0;
            }
            Opcode::Return | Opcode::TailCall | Opcode::TailCallValue => {
                if state.scope != 0 || state.phase != 0 {
                    return Err(error(
                        address,
                        "return or frame replacement abandons a live resource lifetime",
                    ));
                }
            }
            Opcode::CallDirect | Opcode::CallValue | Opcode::Intrinsic
                if state.phase & CLEAN != 0 && instruction.abc().a != crate::NO_REGISTER =>
            {
                state.phase = (state.phase & !CLEAN) | UNPROTECTED_CALL;
            }
            _ => {}
        }
        let mut push = |target: usize| queue.push_back((target, state));
        match instruction.opcode() {
            Opcode::Return | Opcode::Panic => {}
            Opcode::Jump => push(instruction.abx().bx as usize),
            Opcode::BranchIfFalse | Opcode::BranchIfTrue => {
                push(instruction.abx().bx as usize);
                push(address + 1);
            }
            Opcode::BranchIfEqualInteger
            | Opcode::BranchIfNotEqualInteger
            | Opcode::BranchIfLessInteger
            | Opcode::BranchIfGreaterInteger
            | Opcode::BranchIfLessEqualInteger
            | Opcode::BranchIfGreaterEqualInteger => {
                push(decoded[address + 1].abx().bx as usize);
                push(address + 2);
            }
            Opcode::ForLoop => {
                push(decoded[address + 1].abx().bx as usize);
                push(decoded[address + 2].abx().bx as usize);
            }
            _ => push(address + 1),
        }
    }
    Ok(())
}
