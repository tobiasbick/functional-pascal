//! Batch-specific option validation; target discovery remains shared.

use super::types::DebugBatchConfig;

pub(super) fn parse_option(
    args: &[String],
    index: &mut usize,
    batch: &mut DebugBatchConfig,
) -> Result<(), String> {
    let option = args[*index].clone();
    let value = super::options::take_option_value(
        args,
        index,
        &option,
        &format!("Missing value after `{option}`."),
    )?;
    match option.as_str() {
        "--break" | "--print" | "--trace" => {
            let values = if option == "--break" {
                &mut batch.breakpoints
            } else if option == "--print" {
                &mut batch.expressions
            } else {
                &mut batch.trace
            };
            if values.len() >= 64 || value.is_empty() || value.len() > 4096 {
                return Err(format!(
                    "`{option}` accepts at most 64 nonempty values of at most 4096 UTF-8 bytes."
                ));
            }
            values.push(value.into());
        }
        "--trace-limit" => {
            let value = value
                .parse::<usize>()
                .ok()
                .filter(|n| (1..=10_000).contains(n))
                .ok_or("`--trace-limit` requires an integer from 1 to 10000.")?;
            if batch.trace_limit.replace(value).is_some() {
                return Err("Duplicate --trace-limit.".into());
            }
        }
        "--task" => {
            let value = value
                .parse::<u64>()
                .map_err(|_| "`--task` requires a non-negative task ID.")?;
            if batch.task.replace(value).is_some() {
                return Err("Duplicate --task.".into());
            }
        }
        "--frame" => {
            let value = value
                .parse::<usize>()
                .map_err(|_| "`--frame` requires a non-negative depth.")?;
            if batch.frame.replace(value).is_some() {
                return Err("Duplicate --frame.".into());
            }
        }
        _ => unreachable!("batch option selected by the owning parser"),
    }
    Ok(())
}
