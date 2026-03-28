use super::super::{define_builtin_std, define_const, define_func, p};
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

pub(super) fn register_std_math(checker: &mut Checker) {
    define_const(checker, s::STD_MATH_PI, Ty::Real);
    define_func(
        checker,
        s::STD_MATH_SQRT,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_POW,
        vec![p("Base", Ty::Real, false), p("Exp", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_FLOOR,
        vec![p("R", Ty::Real, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_MATH_CEIL,
        vec![p("R", Ty::Real, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_MATH_ROUND,
        vec![p("R", Ty::Real, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_MATH_SIN,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_COS,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_LOG,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );

    define_builtin_std(
        checker,
        s::STD_MATH_ABS,
        Ty::Function(FunctionTy {
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_MIN,
        Ty::Function(FunctionTy {
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_MAX,
        Ty::Function(FunctionTy {
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
}
