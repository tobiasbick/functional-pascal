use super::super::define_builtin_std;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_array(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
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
        s::STD_ARRAY_FILL,
        s::STD_ARRAY_FIND,
        s::STD_ARRAY_FIND_INDEX,
        s::STD_ARRAY_ANY,
        s::STD_ARRAY_ALL,
        s::STD_ARRAY_FLAT_MAP,
        s::STD_ARRAY_FOR_EACH,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}
