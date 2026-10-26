use super::*;

#[test]
fn invalid_client_state_is_rejected_before_launch_and_does_not_consume_process_slots() {
    let registry = Registry::default();
    let processes = Arc::new(PipedProcesses::default());
    let mut settings = Settings::for_version(fpas_acp::schema::Version::V1);
    settings.client_info = Some(fpas_acp::protocol::v1::Implementation::new("test", "1"));
    let limits = Limits {
        queue_bytes: 1,
        ..Limits::default()
    };
    for _ in 0..65 {
        let result = registry.connect_process(
            settings.clone(),
            limits.clone(),
            AcpLaunch {
                command: "fpas-acp-invalid-settings-must-not-launch".into(),
                arguments: Vec::new(),
                directory: None,
                environment: Vec::new(),
            },
            processes.clone(),
        );
        assert_eq!(result, Err(Fault::LimitExceeded("ClientStateBytes", 1)));
    }
}
