//! Live-image compatibility classes without replacement.

use super::*;
use crate::vm::debug::{DebugErrorKind, LiveImageClassification, LiveImageUpdateClass};
use fpas_bytecode::{
    DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureKind, DebugCaptureSource,
    DebugTypeId, EnumLayout, FunctionId, GlobalInfo, Opcode, RecordField, RecordLayout, Register,
    StringId,
};

fn pair_executable() -> VerifiedExecutable {
    executable(
        vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::CallDirect, NO_REGISTER, 1, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 4, 1, debug(&[(0, 1), (1, 2), (2, 3)])),
            function("helper", 4, 6, 1, debug(&[(4, 10), (5, 11)])),
        ],
        Vec::new(),
        vec![(0, 1), (4, 10)],
    )
}

fn patch_helper_body(current: &VerifiedExecutable) -> VerifiedExecutable {
    let mut image = current.clone().into_unverified();
    image.functions[1].register_count = 2;
    image.code[4] = abc(Opcode::LoadUnit, 1, 0, 0);
    image.verify().expect("helper body candidate")
}

fn owner_binding() -> DebugBinding {
    DebugBinding {
        name: StringId::new(1),
        type_name: StringId::new(1),
        ty: DebugTypeId::new(0),
        register: Register::new(0).expect("register"),
        kind: DebugBindingKind::Local,
        mutable: false,
        scope: 0,
        declaration: None,
        hidden: false,
        cell_backed: false,
        initializer: None,
    }
}

fn one_capture() -> DebugCaptureSource {
    DebugCaptureSource {
        binding: DebugBindingId::new(0),
        ty: DebugTypeId::new(0),
        kind: DebugCaptureKind::Value,
    }
}

fn spawn_pair_executable() -> VerifiedExecutable {
    let mut spawner = function("helper", 1, 4, 1, debug(&[(1, 10)]));
    spawner.flags.uses_spawn_tasks = true;
    let mut task = function("task", 4, 5, 0, debug(&[(4, 20)]));
    task.name = StringId::new(3);
    executable(
        vec![
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("function constant"),
            abc(Opcode::SpawnDetachedTask, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 1, 0, debug(&[(0, 1)])), spawner, task],
        vec![Constant::Function {
            function: FunctionId::new(2),
            task_bound: false,
        }],
        vec![(0, 1), (1, 10), (4, 20)],
    )
}

fn classify_current(
    current: VerifiedExecutable,
    candidate: VerifiedExecutable,
) -> LiveImageClassification {
    let session = DebugSession::new(current).expect("debug session");
    session.classify_live_image(&candidate)
}

#[test]
fn identical_images_are_unchanged_and_do_not_replace_the_live_executable() {
    let current = pair_executable();
    let session = DebugSession::new(current.clone()).expect("debug session");
    let before = session.stack(0, 1).expect("stack").items[0].id;
    let classification = session.classify_live_image(&current);
    assert_eq!(classification.class, LiveImageUpdateClass::Unchanged);
    assert!(classification.accepted);
    assert_eq!(
        session.classify_current_live_image().class,
        LiveImageUpdateClass::Unchanged
    );
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, before);
    assert_eq!(session.last_stop().reason, DebugStopReason::Entry);
}

#[test]
fn inactive_helper_body_is_accepted_without_replacing_the_image() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let session = DebugSession::new(current).expect("debug session");
    let before = session.stack(0, 1).expect("stack").items[0].id;
    let classification = session.classify_live_image(&candidate);
    assert_eq!(
        classification.class,
        LiveImageUpdateClass::InactiveFunctionBody
    );
    assert!(classification.accepted);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, before);
}

#[test]
fn active_helper_body_is_rejected() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    let bound = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("helper breakpoint");
    assert!(bound.is_verified());
    let _ = stopped(session.continue_execution().expect("stop in helper"));
    let classification = session.classify_live_image(&candidate);
    assert_eq!(
        classification.class,
        LiveImageUpdateClass::ActiveFunctionBody
    );
    assert!(!classification.accepted);
}

#[test]
fn record_layout_changes_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.records.push(RecordLayout {
        name: StringId::new(3),
        fields: vec![RecordField {
            name: StringId::new(1),
            ty: DebugTypeId::new(0),
        }],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    let candidate = image.verify().expect("record layout candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::RecordLayout);
    assert!(!classification.accepted);
}

#[test]
fn enum_layout_changes_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.enums.push(EnumLayout {
        name: StringId::new(3),
    });
    let candidate = image.verify().expect("enum layout candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::EnumLayout);
    assert!(!classification.accepted);
}

#[test]
fn global_layout_changes_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.globals.push(GlobalInfo {
        name: StringId::new(3),
        ty: DebugTypeId::new(0),
        mutable: true,
        initializer: None,
    });
    let candidate = image.verify().expect("global layout candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::GlobalLayout);
    assert!(!classification.accepted);
}

#[test]
fn capture_count_changes_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.functions[0].debug.bindings.push(owner_binding());
    image.functions[1].capture_count = 1;
    image.functions[1].debug.lexical_owner = Some(FunctionId::new(0));
    image.functions[1].debug.capture_sources = vec![one_capture()];
    let candidate = image.verify().expect("capture candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::ClosureCapture);
    assert!(!classification.accepted);
}

#[test]
fn spawn_flag_changes_are_rejected_as_task_identity() {
    let current = spawn_pair_executable();
    let mut image = current.clone().into_unverified();
    image.code[2] = abc(Opcode::LoadUnit, 0, 0, 0);
    image.functions[1].flags.uses_spawn_tasks = false;
    let candidate = image.verify().expect("task identity candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::TaskIdentity);
    assert!(!classification.accepted);
}

#[test]
fn added_functions_are_rejected() {
    let current = pair_executable();
    let mut extra = function("extra", 6, 7, 1, debug(&[(6, 20)]));
    extra.name = StringId::new(3);
    let mut image = current.clone().into_unverified();
    image.code.push(abc(Opcode::Return, NO_REGISTER, 0, 0));
    image.functions.push(extra);
    image.source_map.runs.push(SourceRun {
        instruction_start: InstructionAddress::new(6),
        source: SourceId::new(0),
        line: 20,
        column: 3,
    });
    let candidate = image.verify().expect("function set candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::FunctionSet);
    assert!(!classification.accepted);
}

#[test]
fn added_capturing_functions_are_rejected_as_anonymous_closures() {
    let current = pair_executable();
    let mut extra = function("extra", 6, 7, 1, debug(&[(6, 20)]));
    extra.name = StringId::new(3);
    extra.capture_count = 1;
    extra.debug.lexical_owner = Some(FunctionId::new(0));
    extra.debug.capture_sources = vec![one_capture()];
    let mut image = current.clone().into_unverified();
    image.functions[0].debug.bindings.push(owner_binding());
    image.code.push(abc(Opcode::Return, NO_REGISTER, 0, 0));
    image.functions.push(extra);
    image.source_map.runs.push(SourceRun {
        instruction_start: InstructionAddress::new(6),
        source: SourceId::new(0),
        line: 20,
        column: 3,
    });
    let candidate = image.verify().expect("anonymous closure candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::AnonymousClosure);
    assert!(!classification.accepted);
}

#[test]
fn entry_renames_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.functions[0].name = StringId::new(3);
    let candidate = image.verify().expect("entry candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::EntryPoint);
    assert!(!classification.accepted);
}

#[test]
fn debug_metadata_only_changes_are_rejected() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.source_map.runs[0].line = 99;
    let candidate = image.verify().expect("debug metadata candidate");
    let classification = classify_current(current, candidate);
    assert_eq!(classification.class, LiveImageUpdateClass::DebugMetadata);
    assert!(!classification.accepted);
}

#[test]
fn proven_subset_names_accepted_and_rejected_classes() {
    assert_eq!(
        LiveImageUpdateClass::ACCEPTED,
        &[
            LiveImageUpdateClass::Unchanged,
            LiveImageUpdateClass::InactiveFunctionBody
        ]
    );
    assert!(
        LiveImageUpdateClass::REJECTED
            .iter()
            .all(|class| !class.is_accepted())
    );
}

fn live_ptr(session: &DebugSession) -> *const VerifiedExecutable {
    session.live_executable() as *const VerifiedExecutable
}

#[test]
fn unchanged_replace_is_accepted_without_applying_a_new_image() {
    let current = pair_executable();
    let mut session = DebugSession::new(current.clone()).expect("debug session");
    let before = live_ptr(&session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let result = session
        .replace_live_image(&current)
        .expect("unchanged replace");
    assert_eq!(result.class, LiveImageUpdateClass::Unchanged);
    assert!(result.accepted);
    assert!(!result.applied);
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
}

#[test]
fn inactive_body_replace_is_accepted_without_committing() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    let before = live_ptr(&session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let result = session
        .replace_live_image(&candidate)
        .expect("inactive body replace");
    assert_eq!(result.class, LiveImageUpdateClass::InactiveFunctionBody);
    assert!(result.accepted);
    assert!(!result.applied);
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
}

#[test]
fn incompatible_replace_is_rejected_before_the_live_image_changes() {
    let current = pair_executable();
    let mut image = current.clone().into_unverified();
    image.records.push(RecordLayout {
        name: StringId::new(3),
        fields: vec![RecordField {
            name: StringId::new(1),
            ty: DebugTypeId::new(0),
        }],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    let candidate = image.verify().expect("record layout candidate");
    let mut session = DebugSession::new(current).expect("debug session");
    let before = live_ptr(&session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let reason = session.last_stop().reason;
    let error = session
        .replace_live_image(&candidate)
        .expect_err("incompatible replace");
    assert_eq!(error.kind, DebugErrorKind::LiveImageIncompatible);
    assert!(error.message.contains("record_layout"), "{error:?}");
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
    assert_eq!(session.last_stop().reason, reason);
}

#[test]
fn active_body_replace_is_rejected_before_the_live_image_changes() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    let bound = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("helper breakpoint");
    assert!(bound.is_verified());
    let _ = stopped(session.continue_execution().expect("stop in helper"));
    let before = live_ptr(&session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let error = session
        .replace_live_image(&candidate)
        .expect_err("active body replace");
    assert_eq!(error.kind, DebugErrorKind::LiveImageIncompatible);
    assert!(error.message.contains("active_function_body"), "{error:?}");
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
}
