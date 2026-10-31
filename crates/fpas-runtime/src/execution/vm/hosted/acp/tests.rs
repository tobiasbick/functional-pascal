//! Public FPAS behavior on a single worker and ordinary debugger lane.

#[test]
fn production_messages_progress_with_one_fpas_worker() {
    let source = include_str!("../../../../../../../tests/stdlib/acp/connection_test.fpas");
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("ACP connection fixture compiles");
    let mut vm = crate::execution::vm::Vm::new(executable);
    vm.pool_size = 1;
    vm.run()
        .expect("ACP receive leaves the sole worker available for its peer");
}

#[test]
fn vm_completion_and_failure_close_unreleased_native_connections() {
    use fpas_acp::{
        connection::{Fault, Lifecycle, Limits},
        schema::Version,
    };
    for (body, succeeds) in [("", true), ("panic('expected shutdown fixture');", false)] {
        let source = format!("program Cleanup; uses Std.Ai.Acp as Acp; begin {body} end.");
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:?}");
        let executable = fpas_compiler::compile(&program).unwrap();
        let mut vm = crate::execution::vm::Vm::new(executable);
        let (id, endpoint_id) = vm
            .hosted
            .acp
            .connect_messages(
                fpas_acp::connection::Settings::for_version(Version::V1),
                Limits::default(),
                vm.hosted.piped_processes.clone(),
            )
            .unwrap();
        let connection = vm.hosted.acp.connection(id).unwrap();
        let endpoint = vm.hosted.acp.endpoint(endpoint_id).unwrap();
        assert_eq!(vm.run().is_ok(), succeeds);
        assert_eq!(connection.state(), Ok(Lifecycle::Closed));
        assert!(endpoint.output_closed());
        assert!(matches!(vm.hosted.acp.connection(id), Err(Fault::Closed)));
        assert!(matches!(
            vm.hosted.acp.endpoint(endpoint_id),
            Err(Fault::Closed)
        ));
    }
}

#[test]
fn cancelled_close_waits_for_native_connection_retirement_before_task_completion() {
    use crate::execution::vm::{
        TaskResultPoll,
        hosted::proc::AcpLaunch,
        tasks::{TaskScheduler, TaskSuspension, pool},
        worker::Worker,
    };
    use fpas_acp::connection::{Limits, Settings};
    use std::{
        sync::{Arc, mpsc},
        time::Duration,
    };

    let marker = std::env::temp_dir().join(format!(
        "fpas-acp-retirement-{}-{}.ready",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let (program, errors) = fpas_parser::parse("program T; begin null; end.");
    assert!(errors.is_empty());
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    let mut worker = Worker::new(image)
        .unwrap()
        .with_scheduler(Some(scheduler.clone()));
    let limits = Limits {
        shutdown_millis: 5000,
        ..Limits::default()
    };
    let id = worker
        .hosted
        .acp
        .connect_process_controlled(
            Settings::for_version(fpas_acp::schema::Version::V1),
            limits,
            AcpLaunch {
                command: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                arguments: vec![
                    "--ignored".into(),
                    "--exact".into(),
                    "execution::vm::hosted::acp::tests::close_peer".into(),
                    "--nocapture".into(),
                ],
                directory: None,
                environment: vec![(
                    "FPAS_ACP_CLOSE_MARKER".into(),
                    Some(marker.to_string_lossy().into_owned()),
                )],
            },
            worker.hosted.piped_processes.clone(),
            || false,
        )
        .expect("connection");
    let connection = worker.hosted.acp.connection(id).unwrap();
    assert!(connection.close().unwrap());
    assert!(!connection.is_closed(), "the subprocess is gated");
    worker.task_id = 7;
    worker.retain_result = true;
    worker.task_suspension = Some(TaskSuspension::Acp(Box::new(super::wait::Wait::Close {
        connection: connection.clone(),
        first: true,
        release: false,
        destination: None,
    })));
    scheduler.register_result(7);
    let template = worker.pool_template();
    scheduler.enqueue(worker.take_task_state());
    scheduler.request_cancel();
    let pool_scheduler = scheduler.clone();
    let (finished, completion) = mpsc::channel();
    let pool = std::thread::spawn(move || {
        let result = pool::pool_loop(&template, pool_scheduler);
        finished.send(result).unwrap();
    });
    let timer_scheduler = scheduler.clone();
    let timer = std::thread::spawn(move || timer_scheduler.timer_loop());
    let premature = completion.recv_timeout(Duration::from_millis(100)).is_ok();
    let pending = matches!(scheduler.poll_result(7), TaskResultPoll::Pending);
    std::fs::write(&marker, b"ready").unwrap();
    let completed = completion.recv_timeout(Duration::from_secs(10));
    if completed.is_err() {
        scheduler.request_cancel();
    }
    pool.join().unwrap();
    timer.join().unwrap();
    let _ = std::fs::remove_file(&marker);
    assert!(
        !premature && pending,
        "completion must wait for physical ACP retirement"
    );
    completed
        .expect("retirement completes")
        .expect("cancelled task is contained");
    assert!(connection.is_closed());
    assert!(matches!(
        scheduler.poll_result(7),
        TaskResultPoll::Failed(_)
    ));
}

#[test]
fn closing_process_cleanup_leaves_the_single_worker_available_to_its_peer() {
    let marker = std::env::temp_dir().join(format!("fpas-acp-close-{}.ready", std::process::id()));
    assert!(!marker.exists());
    let source = r#"program CloseProgress;
uses Std.Ai.Acp as Acp, Std.Os.Args as Args, Std.Io.Fs as Fs, Std.Tasks as Tasks, Std.Test as Test;
function Close(Connection: Acp.Connection; Started: channel of boolean): boolean;
begin
  discard Tasks.Send(Started, true).Unwrap();
  discard Acp.Close(Connection).Unwrap();
  return true;
end function;
function Unblock(Marker: string): boolean;
begin return Fs.WriteText(Marker, 'ready').Unwrap(); end function;
begin
  const Options := Acp.DefaultOptions();
  const Limits := Options.Limits with ShutdownMillis := 5000; end with;
  const Connection := Acp.ConnectProcess(Acp.Launch(
    Command := Args.ParamStr(0),
    Arguments := ['--ignored', '--exact', 'execution::vm::hosted::acp::tests::close_peer', '--nocapture'],
    WorkingDirectory := None,
    Environment := [Acp.EnvironmentVariable(Name := 'FPAS_ACP_CLOSE_MARKER', Value := Some(Args.ParamStr(1)))]
  ), Options with Limits := Limits; end with).Unwrap();
  scope
    const Started: channel of boolean := Tasks.CreateChannel(1);
    const Closing := go Close(Connection, Started);
    discard Tasks.Receive(Started).Unwrap();
    const Progress := go Unblock(Args.ParamStr(1));
    Test.AssertTrue(Tasks.Wait(Closing));
    Test.AssertTrue(Tasks.Wait(Progress));
  end scope;
  const Event := Acp.Receive(Connection, Some(10000), None).Unwrap();
  case Event.Payload of
    when Acp.EventPayload.Closed(const Status):
      Test.AssertTrue(Status.ProcessExit is Some(Acp.ProcessExitStatus.Exited(0)));
    when Acp.EventPayload.Response(_, _), Acp.EventPayload.RequestFailed(_, _), Acp.EventPayload.Notification(_), Acp.EventPayload.Request(_, _), Acp.EventPayload.RequestCancelled(_), Acp.EventPayload.UnknownNotification(_, _), Acp.EventPayload.V2Response(_, _), Acp.EventPayload.V2Notification(_), Acp.EventPayload.V2Request(_, _), Acp.EventPayload.V2Initialized(_, _), Acp.EventPayload.McpResponse(_, _), Acp.EventPayload.V2McpResponse(_, _):
      panic('Expected closed process');
  end case;
  Test.AssertTrue(Acp.Release(Connection).Unwrap());
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).unwrap();
    let mut vm = crate::execution::vm::Vm::with_args(
        executable,
        vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            marker.to_string_lossy().into_owned(),
        ],
    );
    vm.pool_size = 1;
    let result = vm.run();
    let _ = std::fs::remove_file(&marker);
    result.expect("the queued task must release the peer before forced process termination");
}

#[test]
#[ignore = "subprocess fixture, launched by the owning close-progress test"]
fn close_peer() {
    let marker = std::path::PathBuf::from(std::env::var_os("FPAS_ACP_CLOSE_MARKER").unwrap());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !marker.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "parent did not release its child"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
