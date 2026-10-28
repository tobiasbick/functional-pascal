use crate::cli_input::{ResolvedCli, resolve_cli_config};
use fpas_runtime::CollectionSchedule;
use std::path::Path;

fn parse(parts: &[&str]) -> Result<ResolvedCli, String> {
    resolve_cli_config(
        &parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<_>>(),
        Path::new("/project"),
    )
}
fn schedule(config: ResolvedCli) -> CollectionSchedule {
    match config {
        ResolvedCli::Run(config) => config.collection_schedule,
        ResolvedCli::Test(config) => config.collection_schedule,
        ResolvedCli::Debug(config) => config.collection_schedule,
        _ => panic!("execution command"),
    }
}
#[test]
fn all_execution_commands_use_defaults_and_individual_or_combined_overrides() {
    for command in ["run", "test", "debug"] {
        assert_eq!(
            schedule(parse(&[command, "app_test.fpas"]).unwrap()),
            CollectionSchedule::default()
        );
        let config = schedule(
            parse(&[
                command,
                "app_test.fpas",
                "--gc-cell-threshold=3",
                "--gc-mutation-threshold",
                "7",
                "--gc-poll-interval=11",
            ])
            .unwrap(),
        );
        assert_eq!(
            (
                config.allocations.get(),
                config.mutations.get(),
                config.safe_points.get()
            ),
            (3, 7, 11)
        );
        let config = schedule(parse(&[command, "app_test.fpas", "--gc-poll-interval=1"]).unwrap());
        assert_eq!(
            config.allocations,
            CollectionSchedule::default().allocations
        );
        assert_eq!(config.mutations, CollectionSchedule::default().mutations);
        assert_eq!(config.safe_points.get(), 1);
    }
    for protocol in ["jsonl", "dap"] {
        assert_eq!(
            schedule(
                parse(&[
                    "debug",
                    "app_test.fpas",
                    "--protocol",
                    protocol,
                    "--gc-cell-threshold=5"
                ])
                .unwrap()
            )
            .allocations
            .get(),
            5
        );
    }
}
#[test]
fn every_threshold_rejects_invalid_missing_duplicate_or_unsupported_values() {
    for name in [
        "--gc-cell-threshold",
        "--gc-mutation-threshold",
        "--gc-poll-interval",
    ] {
        for value in ["0", "-1", "1.5", "abc", "", "+1", "184467440737095516160"] {
            let option = format!("{name}={value}");
            for command in ["run", "test", "debug"] {
                let error = parse(&[command, "app_test.fpas", &option]).unwrap_err();
                assert!(error.contains(name), "{error}");
            }
        }
        assert!(
            parse(&["run", "app_test.fpas", name])
                .unwrap_err()
                .contains(name)
        );
        let first = format!("{name}=1");
        assert!(
            parse(&["run", "app_test.fpas", &first, name, "2"])
                .unwrap_err()
                .contains("Duplicate")
        );
        for command in ["check", "build", "fmt"] {
            assert!(parse(&[command, "app_test.fpas", &first]).is_err());
        }
    }
}
#[test]
fn collection_options_after_the_separator_remain_program_arguments() {
    let ResolvedCli::Run(config) =
        parse(&["run", "app_test.fpas", "--", "--gc-cell-threshold=0"]).unwrap()
    else {
        panic!("run")
    };
    assert_eq!(config.collection_schedule, CollectionSchedule::default());
    assert_eq!(config.program_args, ["--gc-cell-threshold=0"]);
}
