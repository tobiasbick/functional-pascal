use super::*;

#[test]
fn abandoned_transports_reap_children_and_return_their_registry_slots() {
    let processes = Arc::new(PipedProcesses::default());
    let launch = AcpLaunch {
        command: std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        arguments: vec!["--list".into()],
        directory: None,
        environment: Vec::new(),
    };
    for _ in 0..65 {
        let transport = AcpProcess::start(processes.clone(), &launch, Limits::default()).unwrap();
        let handle = transport.handle;
        drop(transport);
        assert!(processes.with(handle, |_| Ok(())).is_err());
    }
}

#[test]
fn abandoned_transport_inside_executor_closes_outside_async_context() {
    let processes = Arc::new(PipedProcesses::default());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    runtime.block_on(async {
        let transport = AcpProcess::start(
            processes.clone(),
            &AcpLaunch {
                command: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                arguments: vec!["--list".into()],
                directory: None,
                environment: Vec::new(),
            },
            Limits::default(),
        )
        .unwrap();
        let handle = transport.handle;
        drop(transport);
        tokio::time::timeout(Duration::from_secs(10), async {
            while processes.with(handle, |_| Ok(())).is_ok() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("abandoned process must release its slot");
    });
}
