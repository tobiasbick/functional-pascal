use super::*;

#[test]
fn debugger_names_use_the_public_reserved_keyword_safe_api() {
    assert_eq!(
        Intrinsic::Array(ArrayIntrinsic::Map).debugger_name(),
        "Std.Arrays.Map"
    );
    assert_eq!(
        Intrinsic::Console(ConsoleIntrinsic::Read).debugger_name(),
        "Std.Console.ReadText"
    );
    assert_eq!(
        Intrinsic::Net(NetIntrinsic::WriteWithCancellation).debugger_name(),
        "Std.Net.SendBytesWithCancellation"
    );
    assert_eq!(
        Intrinsic::Result(ResultIntrinsic::Unwrap).debugger_name(),
        "Std.Results.Unwrap"
    );
}

mod variants;

use variants::ALL_INTRINSICS;

#[test]
fn intrinsic_round_trip_encode_decode() {
    for &intr in ALL_INTRINSICS {
        let encoded: u16 = intr.into();
        let decoded = Intrinsic::from_u16(encoded);
        assert_eq!(
            decoded,
            Some(intr),
            "round-trip failed for {intr:?} (discriminant {encoded}): from_u16 returned {decoded:?}"
        );
    }
}

#[test]
fn all_intrinsics_list_is_complete() {
    let count_in_list = ALL_INTRINSICS.len();
    let mut found_via_probe = 0usize;
    for raw in 0..=u16::MAX {
        if Intrinsic::from_u16(raw).is_some() {
            found_via_probe += 1;
        }
    }
    assert_eq!(
        count_in_list, found_via_probe,
        "ALL_INTRINSICS has {count_in_list} entries but from_u16 recognises {found_via_probe} — \
         a variant was added without updating ALL_INTRINSICS"
    );
}

#[test]
fn intrinsic_wire_values_are_globally_unique() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    let intrinsic_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/intrinsic");
    let mut by_value: HashMap<u16, Vec<String>> = HashMap::new();

    fn scan_dir(
        dir: &std::path::Path,
        by_value: &mut HashMap<u16, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, by_value)?;
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if matches!(file_name, "mod.rs" | "tests.rs") {
                continue;
            }
            let source = fs::read_to_string(&path)?;
            let parent = dir
                .parent()
                .ok_or_else(|| format!("{} has no parent", dir.display()))?;
            let label = path.strip_prefix(parent)?.display().to_string();
            for line in source.lines() {
                let Some((lhs, rhs)) = line.split_once('=') else {
                    continue;
                };
                if !rhs.trim_end().ends_with(',') {
                    continue;
                }
                let Some(value_text) = rhs.trim().strip_suffix(',') else {
                    continue;
                };
                let Ok(value) = value_text.trim().parse::<u16>() else {
                    continue;
                };
                let variant = lhs.rsplit("::").next().unwrap_or(lhs).trim();
                if variant.is_empty() || variant.starts_with("//") {
                    continue;
                }
                by_value
                    .entry(value)
                    .or_default()
                    .push(format!("{label}:{variant}"));
            }
        }
        Ok(())
    }

    scan_dir(&intrinsic_root, &mut by_value)?;

    let duplicates: Vec<_> = by_value
        .iter()
        .filter(|(_, owners)| owners.len() > 1)
        .collect();
    assert!(
        duplicates.is_empty(),
        "duplicate intrinsic wire values: {duplicates:?}"
    );
    Ok(())
}
