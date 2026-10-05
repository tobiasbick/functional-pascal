//! Diagnostic allocation and documentation consistency.

use super::*;
use crate::DiagnosticStage;
use std::collections::HashSet;

#[test]
fn reference_has_exactly_one_example_pair_for_every_allocated_code() {
    let reference = include_str!("../../../../docs/pascal/tools/diagnostics.md");
    let rows = reference
        .lines()
        .filter_map(|line| {
            let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
            (cells.len() == 6 && cells[1].starts_with("FP")).then_some(cells)
        })
        .collect::<Vec<_>>();
    let allocated = ALL_CODE_INVENTORIES
        .iter()
        .flat_map(|codes| codes.iter())
        .map(ToString::to_string)
        .collect::<HashSet<_>>();
    assert_eq!(
        rows.len(),
        allocated.len(),
        "catalog must cover only allocated codes"
    );
    for code in &allocated {
        let matching = rows
            .iter()
            .filter(|cells| cells[1] == code)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "missing or duplicate catalog row: {code}"
        );
        let row = matching[0];
        assert!(
            !row[2].is_empty() && !row[3].is_empty() && !row[4].is_empty(),
            "missing meaning or example: {code}"
        );
    }
    for row in rows {
        assert!(
            allocated.contains(row[1]),
            "unallocated code in catalog: {}",
            row[1]
        );
    }
}

#[test]
fn allocated_codes_are_unique() {
    let mut seen = HashSet::new();
    for stage_codes in ALL_CODE_INVENTORIES {
        for code in stage_codes.iter().copied() {
            assert!(
                seen.insert(code.value()),
                "duplicate diagnostic code allocation detected: {code}",
            );
        }
    }
}

#[test]
fn allocated_codes_match_stage_ranges() {
    for code in PROJECT_ALLOCATED_CODES {
        assert_eq!(code.stage(), DiagnosticStage::Project);
    }
    for code in LEX_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Lex,
            "lex catalog code {code} is outside the lex range"
        );
    }
    for code in PARSE_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Parse,
            "parse catalog code {code} is outside the parse range"
        );
    }
    for code in SEMA_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Sema,
            "sema catalog code {code} is outside the sema range"
        );
    }
    for code in COMPILE_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Compile,
            "compile catalog code {code} is outside the compile range"
        );
    }
    for code in RUNTIME_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Runtime,
            "runtime catalog code {code} is outside the runtime range"
        );
    }
    for code in INTERNAL_ALLOCATED_CODES {
        assert_eq!(
            code.stage(),
            DiagnosticStage::Internal,
            "internal catalog code {code} is outside the internal range"
        );
    }
}
