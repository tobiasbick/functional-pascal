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

fn patch_helper_with_shifted_sequence_point(current: &VerifiedExecutable) -> VerifiedExecutable {
    let mut image = current.clone().into_unverified();
    image.code.insert(4, abc(Opcode::LoadUnit, 0, 0, 0));
    image.functions[1].code =
        fpas_bytecode::CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(7));
    for point in &mut image.functions[1].debug.sequence_points {
        point.instruction = InstructionAddress::new(point.instruction.get() + 1);
    }
    image.source_map.runs[1].instruction_start = InstructionAddress::new(5);
    image.source_map.runs.insert(
        1,
        SourceRun {
            instruction_start: InstructionAddress::new(4),
            source: SourceId::new(0),
            line: 9,
            column: 3,
        },
    );
    image.verify().expect("shifted helper body candidate")
}

fn inactive_between_active_functions_pair() -> (VerifiedExecutable, VerifiedExecutable) {
    let target = function("helper", 3, 5, 1, debug(&[(3, 20), (4, 21)]));
    let mut current = executable(
        vec![
            abc(Opcode::CallDirect, NO_REGISTER, 2, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 2, 0, debug(&[(0, 1), (1, 2)])),
            function("helper", 2, 3, 1, debug(&[(2, 10)])),
            target,
        ],
        Vec::new(),
        vec![(0, 1), (2, 10), (3, 20)],
    )
    .into_unverified();
    current.strings = StringTable::new(vec![
        "root".to_string(),
        "helper".to_string(),
        "test.fpas".to_string(),
        "boom".to_string(),
        "target".to_string(),
    ]);
    current.functions[2].name = StringId::new(4);
    let current = current.verify().expect("active-shift current");

    let mut candidate = current.clone().into_unverified();
    candidate.code.insert(2, abc(Opcode::LoadUnit, 0, 0, 0));
    candidate.functions[1].code =
        fpas_bytecode::CodeRange::new(InstructionAddress::new(2), InstructionAddress::new(4));
    candidate.functions[1].debug.sequence_points = vec![point(2, 9), point(3, 10)];
    candidate.functions[2].code =
        fpas_bytecode::CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(6));
    for point in &mut candidate.functions[2].debug.sequence_points {
        point.instruction = InstructionAddress::new(point.instruction.get() + 1);
    }
    candidate.source_map.runs = vec![
        SourceRun {
            instruction_start: InstructionAddress::new(0),
            source: SourceId::new(0),
            line: 1,
            column: 3,
        },
        SourceRun {
            instruction_start: InstructionAddress::new(2),
            source: SourceId::new(0),
            line: 9,
            column: 3,
        },
        SourceRun {
            instruction_start: InstructionAddress::new(3),
            source: SourceId::new(0),
            line: 10,
            column: 3,
        },
        SourceRun {
            instruction_start: InstructionAddress::new(4),
            source: SourceId::new(0),
            line: 20,
            column: 3,
        },
    ];
    let candidate = candidate.verify().expect("active-shift candidate");
    (current, candidate)
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
fn reordered_function_ids_are_rejected() {
    let (current, _) = inactive_between_active_functions_pair();
    let mut image = current.clone().into_unverified();
    let helper_name = image.functions[1].name;
    image.functions[1].name = image.functions[2].name;
    image.functions[2].name = helper_name;
    let candidate = image.verify().expect("reordered candidate");
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
fn unchanged_replace_keeps_the_current_version_and_image() {
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
    assert_eq!(result.version, 1);
    assert!(!result.rollback_available);
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
}

#[test]
fn inactive_body_replace_commits_one_shared_version() {
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
    assert!(result.applied);
    assert_eq!(result.version, 2);
    assert!(result.rollback_available);
    assert_ne!(live_ptr(&session), before);
    assert_ne!(session.stack(0, 1).expect("stack").items[0].id, frame);
    assert!(session.test_workers_share_live_image());
    assert_eq!(session.test_retained_live_image_count(), 2);
}

#[test]
fn rollback_restores_the_previous_image_as_a_new_bounded_version() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    let original = live_ptr(&session);
    session
        .replace_live_image(&candidate)
        .expect("candidate commit");
    let replacement = live_ptr(&session);
    let result = session.rollback_live_image().expect("rollback");
    assert_eq!(result.class, LiveImageUpdateClass::InactiveFunctionBody);
    assert!(result.applied);
    assert_eq!(result.version, 3);
    assert!(result.rollback_available);
    assert_eq!(live_ptr(&session), original);
    assert_ne!(live_ptr(&session), replacement);
    assert!(session.test_workers_share_live_image());
    assert_eq!(session.test_retained_live_image_count(), 2);
}

#[test]
fn repeated_rollbacks_never_retain_more_than_two_session_images() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    session
        .replace_live_image(&candidate)
        .expect("initial candidate commit");
    for expected_version in 3..=10 {
        let result = session.rollback_live_image().expect("bounded rollback");
        assert_eq!(result.version, expected_version);
        assert_eq!(session.test_retained_live_image_count(), 2);
        assert!(session.test_workers_share_live_image());
    }
}

#[test]
fn rollback_without_a_previous_image_is_actionable_and_atomic() {
    let mut session = DebugSession::new(pair_executable()).expect("debug session");
    let before = live_ptr(&session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let error = session
        .rollback_live_image()
        .expect_err("missing rollback image");
    assert_eq!(error.kind, DebugErrorKind::LiveImageRollbackUnavailable);
    assert_eq!(session.live_image_version(), 1);
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.stack(0, 1).expect("stack").items[0].id, frame);
}

#[test]
fn changed_inactive_prefix_remaps_the_active_instruction_and_continues() {
    let (current, candidate) = inactive_between_active_functions_pair();
    let mut session = DebugSession::new(current).expect("debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 20,
            column: None,
        })
        .expect("target breakpoint");
    let stop = stopped(session.continue_execution().expect("target stop"));
    assert_eq!(stop.instruction, 3);
    let result = session
        .replace_live_image(&candidate)
        .expect("shifted inactive prefix");
    assert!(result.applied);
    assert_eq!(session.last_stop().instruction, 4);
    assert!(matches!(
        session
            .continue_execution()
            .expect("continue remapped image"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn source_breakpoints_rebind_to_changed_inactive_body_metadata() {
    let current = pair_executable();
    let candidate = patch_helper_with_shifted_sequence_point(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    let bound = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("helper breakpoint");
    assert_eq!(bound.instruction, Some(4));
    session
        .replace_live_image(&candidate)
        .expect("helper replacement");
    let stop = stopped(session.continue_execution().expect("rebound breakpoint"));
    assert_eq!(stop.instruction, 5);
    assert_eq!(stop.location.expect("source location").line, 10);
}

#[test]
fn recording_rejects_a_real_image_commit_without_relabeling_events() {
    let current = pair_executable();
    let candidate = patch_helper_body(&current);
    let mut session = DebugSession::new(current).expect("debug session");
    session.start_recording();
    let before = live_ptr(&session);
    let events = session.recording_events().to_vec();
    let error = session
        .replace_live_image(&candidate)
        .expect_err("recording must retain its image identity");
    assert_eq!(error.kind, DebugErrorKind::InvalidState);
    assert_eq!(session.live_image_version(), 1);
    assert_eq!(live_ptr(&session), before);
    assert_eq!(session.recording_events(), events);
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
