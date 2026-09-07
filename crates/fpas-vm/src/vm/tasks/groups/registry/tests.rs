//! Ownership, close ordering, admission limits, and resource lifetime regressions.

use super::*;
use crate::vm::cancellation::CancellationRegistry;

fn returned(task: u64, message: &str) -> Option<GroupFailure> {
    GroupFailure::returned(task, &Value::result_error(Value::Str(message.into())))
}

#[test]
fn close_waits_for_every_child_and_releases_membership_and_token() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    let token = groups.enroll(group, 0, 1).expect("first child");
    groups.enroll(group, 1, 2).expect("nested child");
    groups.begin_close(group, 0).expect("close");
    assert!(cancellations.is_cancelled(token).expect("live token"));
    assert!(groups.complete(1, None));
    assert!(groups.take_closed(group).expect("pending close").is_none());
    assert!(groups.enroll(group, 0, 3).is_err());
    assert!(groups.complete(2, None));
    let closed = groups.take_closed(group).expect("close").expect("joined");
    assert_eq!(closed.tasks, vec![1, 2]);
    assert!(closed.failures.is_empty());
    assert!(cancellations.is_cancelled(token).is_err());
    assert!(!groups.complete(1, None));
    let state = groups.state.lock().expect("state");
    assert!(state.groups.is_empty());
    assert!(state.membership.is_empty());
}

#[test]
fn reports_keep_registration_order_and_first_terminal_outcome() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    for task in 1..=3 {
        groups.enroll(group, 0, task).expect("child");
    }
    groups.complete(3, returned(3, "third"));
    groups.complete(1, returned(1, "cancelled"));
    groups.complete(1, returned(1, "replacement"));
    groups.complete(2, None);
    assert!(
        !cancellations
            .is_cancelled(groups.token(group).expect("token"))
            .expect("state")
    );
    groups.begin_close(group, 0).expect("close");
    let closed = groups.take_closed(group).expect("close").expect("joined");
    let reports: Vec<_> = closed
        .failures
        .iter()
        .map(|failure| (failure.task, failure.kind, failure.message.as_str()))
        .collect();
    assert_eq!(
        reports,
        vec![
            (1, GroupFailureKind::ReturnedError, "cancelled"),
            (3, GroupFailureKind::ReturnedError, "third")
        ]
    );
}

#[test]
fn only_owner_can_close_and_only_owner_or_members_can_enroll() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(10, || cancellations.create_owned())
        .expect("group");
    groups.enroll(group, 10, 11).expect("child");
    assert!(groups.enroll(group, 20, 21).is_err());
    assert!(groups.begin_close(group, 11).is_err());
    assert!(groups.begin_close(group, 20).is_err());
    groups
        .enroll(group, 11, 12)
        .expect("nested child after rejected closes");
    assert_eq!(groups.state.lock().expect("state").membership.len(), 2);
}

#[test]
fn cancellation_seals_admission_but_does_not_release_children() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    groups.enroll(group, 0, 1).expect("child");
    assert!(groups.cancel(group).expect("first cancel"));
    assert!(!groups.cancel(group).expect("second cancel"));
    assert!(groups.enroll(group, 0, 2).is_err());
    assert!(groups.take_closed(group).is_err());
    assert!(groups.complete(1, None));
    assert!(groups.token(group).is_ok());
}

#[test]
fn child_limit_counts_completed_children_until_close() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    for task in 1..=1024 {
        groups.enroll(group, 0, task).expect("child within limit");
        groups.complete(task, None);
    }
    assert!(groups.enroll(group, 0, 1025).is_err());
    groups.begin_close(group, 0).expect("close");
    assert_eq!(
        groups
            .take_closed(group)
            .expect("close")
            .expect("joined")
            .tasks
            .len(),
        1024
    );
}

#[test]
fn live_group_limit_is_checked_before_allocating_source_and_recovers_after_close() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let mut ids = Vec::new();
    for _ in 0..4096 {
        ids.push(
            groups
                .create(0, || cancellations.create_owned())
                .expect("group within limit"),
        );
    }
    assert!(
        groups
            .create(0, || panic!("must check capacity before source creation"))
            .is_err()
    );
    groups.begin_close(ids[0], 0).expect("close");
    assert!(groups.take_closed(ids[0]).expect("close").is_some());
    assert!(groups.create(0, || cancellations.create_owned()).is_ok());
}

#[test]
fn repeated_close_is_empty_and_wrong_resource_types_are_rejected() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    groups.begin_close(group, 0).expect("close");
    groups.take_closed(group).expect("close").expect("joined");
    groups.begin_close(group, 0).expect("repeat close");
    let closed = groups
        .take_closed(group)
        .expect("repeat close")
        .expect("joined");
    assert!(closed.tasks.is_empty() && closed.failures.is_empty());
    assert!(groups.begin_close(1, 0).is_err());
    assert!(groups.take_closed(1).is_err());
}

#[test]
fn shutdown_cancels_all_groups_without_pretending_pending_children_completed() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    let group = groups
        .create(0, || cancellations.create_owned())
        .expect("group");
    let token = groups.enroll(group, 0, 1).expect("child");
    groups.shutdown();
    assert!(cancellations.is_cancelled(token).expect("live token"));
    assert!(groups.take_closed(group).expect("pending close").is_none());
    assert!(groups.enroll(group, 0, 2).is_err());
    drop(groups);
    assert!(cancellations.is_cancelled(token).is_err());
}

#[test]
fn returned_failure_messages_are_bounded_by_unicode_characters() {
    let failure = returned(1, &"ä".repeat(5000)).expect("failure");
    assert_eq!(failure.message.chars().count(), 4096);
    assert_eq!(failure.code, 0);
    assert!(GroupFailure::returned(1, &Value::Integer(7)).is_none());
}

#[test]
fn enrollment_racing_close_is_either_owned_or_rejected_without_partial_membership() {
    let cancellations = CancellationRegistry::new();
    let groups = GroupRegistry::default();
    for _ in 0..200 {
        let group = groups
            .create(0, || cancellations.create_owned())
            .expect("group");
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let registration = scope.spawn(|| {
                barrier.wait();
                groups.enroll(group, 0, 1)
            });
            barrier.wait();
            groups.begin_close(group, 0).expect("close");
            match registration.join().expect("registration thread") {
                Ok(token) => {
                    assert!(cancellations.is_cancelled(token).expect("live token"));
                    assert!(groups.take_closed(group).expect("pending close").is_none());
                    assert!(groups.complete(1, None));
                    assert_eq!(
                        groups
                            .take_closed(group)
                            .expect("close")
                            .expect("joined")
                            .tasks,
                        vec![1]
                    );
                }
                Err(_) => {
                    assert!(!groups.complete(1, None));
                    assert!(
                        groups
                            .take_closed(group)
                            .expect("close")
                            .expect("joined")
                            .tasks
                            .is_empty()
                    );
                }
            }
        });
        let state = groups.state.lock().expect("state");
        assert!(state.groups.is_empty() && state.membership.is_empty());
    }
}
