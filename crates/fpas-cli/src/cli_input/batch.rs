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
        "--break" | "--print" => {
            let values = if option == "--break" {
                &mut batch.breakpoints
            } else {
                &mut batch.expressions
            };
            if values.len() >= 64 || value.is_empty() || value.len() > 4096 {
                return Err(format!(
                    "`{option}` accepts at most 64 nonempty values of at most 4096 UTF-8 bytes."
                ));
            }
            values.push(value.into());
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
