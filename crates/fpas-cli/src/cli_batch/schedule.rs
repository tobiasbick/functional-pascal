//! Seed request recovery and honest coverage when a worker cannot publish state.

use serde_json::{Value, json};

pub(super) fn preserve_request(report: &mut Value, args: &[String]) {
    let args: Vec<_> = args.iter().take_while(|arg| *arg != "--").collect();
    if args
        .iter()
        .filter(|arg| arg.as_str() == "--schedule-seed")
        .count()
        != 1
    {
        return;
    }
    let mut seeds = args.windows(2).filter(|pair| pair[0] == "--schedule-seed");
    let seed = seeds
        .next()
        .and_then(|pair| crate::cli_input::parse_schedule_seed(pair[1]).ok());
    if seeds.next().is_some() || seed.is_none() {
        return;
    }
    if report["schedule"]["mode"] != "seeded" {
        report["schedule"] = json!(fpas_runtime::execution::DebugScheduleReport::requested(
            seed
        ));
    }
    normalize(report);
}

pub(super) fn normalize(report: &mut Value) {
    fpas_debug::batch::schedule_coverage(report);
}
