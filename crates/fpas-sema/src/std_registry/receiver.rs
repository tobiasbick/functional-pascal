//! First-parameter shapes of polymorphic standard-library intrinsics.
//!
//! The ordinary call checker remains responsible for every remaining argument.

use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Whether an intrinsic with a placeholder signature accepts this receiver type.
///
/// `None` means the name is not a polymorphic intrinsic and its registered
/// function or procedure signature supplies the first parameter instead.
pub(crate) fn builtin_accepts_receiver(name: &str, receiver: &Ty) -> Option<bool> {
    let is = |known: &str| name.eq_ignore_ascii_case(known);
    let accepts = if [
        s::STD_ARRAY_LENGTH,
        s::STD_ARRAY_SORT,
        s::STD_ARRAY_REVERSE,
        s::STD_ARRAY_CONTAINS,
        s::STD_ARRAY_INDEX_OF,
        s::STD_ARRAY_SLICE,
        s::STD_ARRAY_PUSH,
        s::STD_ARRAY_POP,
        s::STD_ARRAY_MAP,
        s::STD_ARRAY_FILTER,
        s::STD_ARRAY_REDUCE,
        s::STD_ARRAY_CONCAT,
        s::STD_ARRAY_FIND,
        s::STD_ARRAY_FIND_INDEX,
        s::STD_ARRAY_ANY,
        s::STD_ARRAY_ALL,
        s::STD_ARRAY_FLAT_MAP,
        s::STD_ARRAY_FOR_EACH,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Array(_))
    } else if is(s::STD_ARRAY_FILL) {
        true
    } else if is(s::STD_TEST_ASSERT_EQUALS) {
        matches!(receiver, Ty::Integer | Ty::Real | Ty::Boolean | Ty::String)
    } else if [
        s::STD_DICT_LENGTH,
        s::STD_DICT_CONTAINS_KEY,
        s::STD_DICT_KEYS,
        s::STD_DICT_VALUES,
        s::STD_DICT_REMOVE,
        s::STD_DICT_GET,
        s::STD_DICT_MERGE,
        s::STD_DICT_MAP,
        s::STD_DICT_FILTER,
        s::STD_DICT_REDUCE,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Dict(_, _))
    } else if [
        s::STD_RESULT_UNWRAP,
        s::STD_RESULT_UNWRAP_OR,
        s::STD_RESULT_IS_OK,
        s::STD_RESULT_IS_ERR,
        s::STD_RESULT_MAP,
        s::STD_RESULT_AND_THEN,
        s::STD_RESULT_OR_ELSE,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Result(_, _))
    } else if [
        s::STD_OPTION_UNWRAP,
        s::STD_OPTION_UNWRAP_OR,
        s::STD_OPTION_IS_SOME,
        s::STD_OPTION_IS_NONE,
        s::STD_OPTION_MAP,
        s::STD_OPTION_AND_THEN,
        s::STD_OPTION_OR_ELSE,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Option(_))
    } else if [s::STD_STR_MAP, s::STD_STR_FILTER, s::STD_STR_REDUCE]
        .iter()
        .any(|known| is(known))
    {
        matches!(receiver, Ty::String)
    } else if [
        s::STD_MATH_ABS,
        s::STD_MATH_MIN,
        s::STD_MATH_MAX,
        s::STD_MATH_SIGN,
        s::STD_MATH_CLAMP,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Integer | Ty::Real)
    } else if [
        s::STD_TASK_SEND,
        s::STD_TASK_TRY_SEND,
        s::STD_TASK_SEND_WITH_CANCELLATION,
        s::STD_TASK_SEND_WITH_TIMEOUT,
        s::STD_TASK_RECEIVE,
        s::STD_TASK_TRY_RECEIVE,
        s::STD_TASK_RECEIVE_WITH_CANCELLATION,
        s::STD_TASK_RECEIVE_WITH_TIMEOUT,
        s::STD_TASK_CLOSE_CHANNEL,
        s::STD_TASK_RECEIVE_CASE,
        s::STD_TASK_SEND_CASE,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Channel(_))
    } else if [s::STD_TASK_WAIT, s::STD_TASK_TASK_CASE]
        .iter()
        .any(|known| is(known))
    {
        matches!(receiver, Ty::Task(_))
    } else if [
        s::STD_TASK_WAIT_ALL,
        s::STD_TASK_WAIT_ANY,
        s::STD_TASK_WAIT_ANY_WITH_TIMEOUT,
        s::STD_TASK_WAIT_ANY_WITH_CANCELLATION,
    ]
    .iter()
    .any(|known| is(known))
    {
        matches!(receiver, Ty::Array(inner) if matches!(inner.as_ref(), Ty::Task(_)))
    } else if [
        s::STD_TASK_START_TASK_IN_GROUP,
        s::STD_TASK_START_SUPERVISED_TASK,
    ]
    .iter()
    .any(|known| is(known))
    {
        record_is(receiver, s::STD_TASK_TASK_GROUP)
    } else if is(s::STD_TASK_TIMER_CASE) || is(s::STD_TASK_CREATE_CHANNEL) {
        matches!(receiver, Ty::Integer)
    } else if is(s::STD_TASK_CANCELLATION_CASE) {
        record_is(receiver, s::STD_TASK_CANCELLATION_TOKEN)
    } else {
        return None;
    };
    Some(accepts)
}

fn record_is(ty: &Ty, name: &str) -> bool {
    matches!(ty, Ty::Record(record) if record.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::builtin_accepts_receiver;
    use crate::check::Checker;
    use crate::scope::SymbolKind;
    use crate::std_registry::register_single_std_unit;
    use crate::types::Ty;
    use fpas_std::std_symbols as s;

    #[test]
    fn colliding_map_names_filter_by_first_parameter() {
        let numbers = Ty::Array(Box::new(Ty::Integer));
        let words = Ty::String;
        assert_eq!(
            builtin_accepts_receiver(s::STD_ARRAY_MAP, &numbers),
            Some(true)
        );
        assert_eq!(
            builtin_accepts_receiver(s::STD_STR_MAP, &numbers),
            Some(false)
        );
        assert_eq!(builtin_accepts_receiver(s::STD_STR_MAP, &words), Some(true));
    }

    #[test]
    fn every_polymorphic_intrinsic_has_receiver_metadata() {
        let mut checker = Checker::new();
        for unit in fpas_std::STD_UNITS_INTRINSIC {
            register_single_std_unit(&mut checker, unit);
            for (name, symbol) in checker.scopes.root_symbols_with_prefix(&format!("{unit}.")) {
                if symbol.kind == SymbolKind::BuiltinStd {
                    assert!(
                        builtin_accepts_receiver(&name, &Ty::Integer).is_some(),
                        "missing first-parameter metadata for {name}"
                    );
                }
            }
        }
    }
}
