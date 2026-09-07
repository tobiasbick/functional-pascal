//! Task-start intrinsics participate in the same feature flag as spawn opcodes.

use fpas_bytecode::{Intrinsic, NO_REGISTER, Opcode, TaskIntrinsic, ValidationErrorKind};

use super::support::{abc, minimal_executable, replace_root_code, return_unit};

#[test]
fn every_intrinsic_requires_exactly_its_declared_task_spawn_effect() {
    for intrinsic in Intrinsic::all() {
        let starts_task = matches!(
            intrinsic,
            Intrinsic::Task(TaskIntrinsic::StartTaskInGroup | TaskIntrinsic::StartSupervisedTask)
        );
        assert_eq!(intrinsic.starts_task(), starts_task, "{intrinsic:?}");
        let mut image = minimal_executable();
        image.functions[0].flags.uses_spawn_tasks = starts_task;
        replace_root_code(
            &mut image,
            vec![
                abc(Opcode::Intrinsic, NO_REGISTER, intrinsic.into(), 0, 0),
                return_unit(),
            ],
        );
        image.clone().verify().expect("matching spawn flag");
        image.functions[0].flags.uses_spawn_tasks = !starts_task;
        assert!(
            matches!(
                image.verify().expect_err("mismatched spawn flag").kind,
                ValidationErrorKind::SpawnFlag { declared, emitted }
                    if declared == !starts_task && emitted == starts_task
            ),
            "{intrinsic:?}"
        );
    }
}
