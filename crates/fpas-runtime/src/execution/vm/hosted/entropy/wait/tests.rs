//! Capacity, validation and publication races do not consume or replace entropy early.

use super::*;
use crate::{CryptoIntrinsic as Crypto, RandomIntrinsic as Random};

fn worker() -> Worker {
    let (program, errors) =
        fpas_parser::parse("program E; uses Std.Crypto as C; begin discard C.RandomBytes(0); end.");
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap()
}
fn suspended(worker: &mut Worker) -> Wait {
    let Some(TaskSuspension::Entropy(wait)) = worker.task_suspension.take() else {
        panic!("entropy wait");
    };
    *wait
}
#[test]
fn invalid_arguments_and_seeded_draws_never_enter_the_entropy_queue() {
    let mut worker = worker();
    *worker.hosted.entropy_source.provider.lock().unwrap() =
        Some(Arc::new(|_| panic!("unexpected OS entropy")));
    let permits = (0..LIMIT)
        .map(|_| reserve(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    worker
        .start_entropy(
            Intrinsic::Crypto(Crypto::RandomBytes),
            &[Value::Integer(-1)],
            None,
        )
        .unwrap();
    worker
        .start_entropy(
            Intrinsic::Crypto(Crypto::RandomInt),
            &[Value::Integer(2), Value::Integer(1)],
            None,
        )
        .unwrap();
    worker
        .start_entropy(
            Intrinsic::Random(Random::SetSeed),
            &[Value::Integer(42)],
            None,
        )
        .unwrap();
    worker
        .start_entropy(
            Intrinsic::Random(Random::RandomInt),
            &[Value::Integer(7), Value::Integer(7)],
            None,
        )
        .unwrap();
    assert!(worker.task_suspension.is_none());
    worker
        .start_entropy(Intrinsic::Random(Random::Randomize), &[], None)
        .unwrap();
    assert!(matches!(
        suspended(&mut worker).state,
        State::Queued(Request::Seed)
    ));
    drop(permits);
    assert_eq!(worker.hosted.entropy_jobs.load(Ordering::Acquire), 0);
}
#[test]
fn an_automatic_seed_never_overwrites_a_concurrent_explicit_seed_even_after_source_failure() {
    let call = random::Call::Real;
    let mut state = random::state::RandomState::new();
    assert!(call.ready(&mut state).is_none());
    random::Call::SetSeed(42).ready(&mut state);
    let mut expected = random::state::RandomState::new();
    random::Call::SetSeed(42).ready(&mut expected);
    assert_eq!(
        call.finish(&mut state, Err(getrandom::Error::UNSUPPORTED)),
        Ok(call.ready(&mut expected).unwrap())
    );
    assert_eq!(call.ready(&mut state), call.ready(&mut expected));
}
#[test]
fn deferred_abandonment_joins_then_discards_native_buffers_and_releases_admission() {
    let worker = worker();
    let state = worker.hosted.clone();
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let released = Mutex::new(released);
    *state.entropy_source.provider.lock().unwrap() = Some(Arc::new(move |bytes| {
        entered.send(()).unwrap();
        released
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        bytes.fill(17);
        Ok(())
    }));
    let job = Job::start(
        reserve(state.clone()).unwrap(),
        Request::Crypto(crypto::Request::Bytes(131072)),
        SourceLocation::new(1, 1),
    )
    .unwrap();
    entry
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    native::defer_joins(|| drop(job));
    assert_eq!(state.entropy_jobs.load(Ordering::Acquire), 1);
    release.send(()).unwrap();
    native::join_retired(&state);
    assert_eq!(state.entropy_jobs.load(Ordering::Acquire), 0);
    assert!(state.retired_native_jobs.lock().unwrap().is_empty());
}

#[test]
fn cancelling_rejection_sampling_stops_before_the_next_os_sample() {
    let worker = worker();
    let state = worker.hosted.clone();
    let (entered, entry) = mpsc::channel();
    let first = AtomicBool::new(true);
    *state.entropy_source.provider.lock().unwrap() = Some(Arc::new(move |bytes| {
        bytes.fill(0);
        if first.swap(false, Ordering::AcqRel) {
            entered.send(()).unwrap();
        }
        Ok(())
    }));
    // Zero is always in the rejected prefix for a six-element range.
    let job = Job::start(
        reserve(state.clone()).unwrap(),
        Request::Crypto(crypto::Request::Integer(0, 5)),
        SourceLocation::new(1, 1),
    )
    .unwrap();
    entry
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    drop(job);
    assert_eq!(state.entropy_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn queued_crypto_work_starts_only_after_admission_and_completed_results_keep_their_slot() {
    let mut worker = worker();
    let called = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = called.clone();
    *worker.hosted.entropy_source.provider.lock().unwrap() = Some(Arc::new(move |bytes| {
        count.fetch_add(1, Ordering::AcqRel);
        bytes.fill(17);
        Ok(())
    }));
    let mut permits = (0..LIMIT)
        .map(|_| reserve(worker.hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    worker
        .start_entropy(
            Intrinsic::Crypto(Crypto::RandomBytes),
            &[Value::Integer(131072)],
            None,
        )
        .unwrap();
    let wait = suspended(&mut worker);
    assert!(matches!(&wait.state, State::Queued(_)));
    assert_eq!(called.load(Ordering::Acquire), 0);
    drop(permits.pop());
    if !worker.poll_entropy(wait).unwrap() {
        let mut wait = suspended(&mut worker);
        let State::Running(job) = &mut wait.state else {
            panic!("admitted entropy");
        };
        job.thread.take().unwrap().join().unwrap();
        assert_eq!(worker.hosted.entropy_jobs.load(Ordering::Acquire), LIMIT);
        assert!(worker.poll_entropy(wait).unwrap());
    }
    assert_eq!(called.load(Ordering::Acquire), 1);
    assert_eq!(
        worker.hosted.entropy_jobs.load(Ordering::Acquire),
        LIMIT - 1
    );
    drop(permits);
    assert_eq!(worker.hosted.entropy_jobs.load(Ordering::Acquire), 0);
}
