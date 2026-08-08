//! Case-insensitive matching for qualified and imported short type names.

pub(super) fn matches(candidate: &str, requested: &str) -> bool {
    candidate.eq_ignore_ascii_case(requested)
        || candidate
            .rsplit_once('.')
            .is_some_and(|(_, short)| short.eq_ignore_ascii_case(requested))
}
