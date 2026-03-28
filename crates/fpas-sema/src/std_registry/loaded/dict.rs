use super::super::define_builtin_std;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_dict(checker: &mut Checker) {
    let placeholder = Ty::Function(FunctionTy {
        params: vec![],
        return_type: Box::new(Ty::Error),
    });
    for name in [
        s::STD_DICT_LENGTH,
        s::STD_DICT_CONTAINS_KEY,
        s::STD_DICT_KEYS,
        s::STD_DICT_VALUES,
        s::STD_DICT_REMOVE,
    ] {
        define_builtin_std(checker, name, placeholder.clone());
    }
}
