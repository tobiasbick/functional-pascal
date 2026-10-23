use super::percent;

#[test]
fn percent_rounds_to_one_decimal_and_handles_empty_totals() {
    assert_eq!(percent(1, 3), 33.3);
    assert_eq!(percent(2, 3), 66.7);
    assert_eq!(percent(5, 0), 0.0);
}
