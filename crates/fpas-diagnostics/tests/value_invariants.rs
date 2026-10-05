//! Invariant tests for diagnostic codes, locations, and spans.

use fpas_diagnostics::{
    DiagnosticCode, DiagnosticStage, SourceLocation, SourceLocationError, SourceSpan,
    SourceSpanError,
};

#[test]
fn dynamic_diagnostic_codes_accept_the_full_range_and_reject_overflow() {
    assert_eq!(
        DiagnosticCode::try_from(0).map(DiagnosticCode::value),
        Ok(0)
    );
    assert_eq!(
        DiagnosticCode::try_from(DiagnosticCode::MAX_VALUE).map(DiagnosticCode::value),
        Ok(DiagnosticCode::MAX_VALUE)
    );

    assert_eq!(
        DiagnosticCode::try_from(DiagnosticCode::MAX_VALUE + 1).map_err(|error| error.value()),
        Err(DiagnosticCode::MAX_VALUE + 1)
    );
}

#[test]
fn stage_boundaries_are_derived_from_the_code() {
    for (value, expected) in [
        (999, DiagnosticStage::Internal),
        (1000, DiagnosticStage::Lex),
        (1999, DiagnosticStage::Lex),
        (2000, DiagnosticStage::Parse),
        (2001, DiagnosticStage::Parse),
        (2999, DiagnosticStage::Parse),
        (3000, DiagnosticStage::Sema),
        (3999, DiagnosticStage::Sema),
        (4000, DiagnosticStage::Compile),
        (4099, DiagnosticStage::Compile),
        (4100, DiagnosticStage::Project),
        (4999, DiagnosticStage::Project),
        (5000, DiagnosticStage::Runtime),
        (5999, DiagnosticStage::Runtime),
        (6000, DiagnosticStage::Internal),
    ] {
        assert_eq!(DiagnosticCode::new(value).stage(), expected);
    }
}

#[test]
fn dynamic_source_locations_reject_zero_coordinates() {
    assert_eq!(
        SourceLocation::try_new(0, 1),
        Err(SourceLocationError::ZeroLine)
    );
    assert_eq!(
        SourceLocation::try_new(1, 0),
        Err(SourceLocationError::ZeroColumn)
    );
}

#[test]
fn source_id_rebasing_preserves_validated_coordinates() {
    let location = SourceLocation::new(4, 7).with_source_id(9);
    let span = SourceSpan::new(10, 5, 4, 7).with_source_id(9);

    assert_eq!(
        (location.line(), location.column(), location.source_id()),
        (4, 7, 9)
    );
    assert_eq!(span.location(), location);
    assert_eq!((span.offset(), span.length(), span.end()), (10, 5, 15));
}

#[test]
fn source_spans_accept_maximum_end_and_reject_overflow() {
    assert_eq!(
        SourceSpan::try_new(usize::MAX, 0, 1, 1).map(SourceSpan::end),
        Ok(usize::MAX)
    );
    assert_eq!(
        SourceSpan::try_new(usize::MAX - 1, 1, 1, 1).map(SourceSpan::end),
        Ok(usize::MAX)
    );
    assert_eq!(
        SourceSpan::try_new(usize::MAX, 1, 1, 1),
        Err(SourceSpanError::EndOverflow {
            offset: usize::MAX,
            length: 1,
        })
    );
}
