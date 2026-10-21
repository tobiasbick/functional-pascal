//! Positive collection threshold overrides shared by every execution command.
use fpas_bytecode::CollectionSchedule;
use std::num::NonZeroUsize;

pub(super) fn parse(
    args: &[String],
    index: &mut usize,
    schedule: &mut CollectionSchedule,
    seen: &mut u8,
) -> Result<bool, String> {
    let (name, inline) = args[*index]
        .split_once('=')
        .map_or((args[*index].as_str(), None), |(name, value)| {
            (name, Some(value))
        });
    let (slot, bit) = match name {
        "--gc-cell-threshold" => (&mut schedule.allocations, 1),
        "--gc-mutation-threshold" => (&mut schedule.mutations, 2),
        "--gc-poll-interval" => (&mut schedule.safe_points, 4),
        _ => return Ok(false),
    };
    if *seen & bit != 0 {
        return Err(format!("Duplicate `{name}` option."));
    }
    let value = match inline {
        Some(value) => value,
        None => {
            *index += 1;
            args.get(*index)
                .map(String::as_str)
                .ok_or_else(|| format!("Missing positive integer after `{name}`."))?
        }
    };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("`{name}` requires a positive decimal integer."));
    }
    *slot = value.parse::<NonZeroUsize>().map_err(|_| {
        format!("`{name}` requires a positive decimal integer within the supported range.")
    })?;
    *seen |= bit;
    Ok(true)
}

#[cfg(test)]
mod tests;
