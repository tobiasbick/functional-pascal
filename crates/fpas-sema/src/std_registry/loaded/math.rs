use super::super::{define_builtin_std, define_const, define_func, define_proc, p};
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
    define_func(
        checker,
        s::STD_MATH_TAN,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_ARC_SIN,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_ARC_COS,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_ARC_TAN,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_ARC_TAN2,
        vec![p("Y", Ty::Real, false), p("X", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_EXP,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_LOG10,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_LOG2,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(
        checker,
        s::STD_MATH_TRUNC,
        vec![p("R", Ty::Real, false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_MATH_FRAC,
        vec![p("R", Ty::Real, false)],
        Ty::Real,
    );
    define_func(checker, s::STD_MATH_RANDOM, vec![], Ty::Real);
    define_func(
        checker,
        s::STD_MATH_RANDOM_INT,
        vec![p("Lo", Ty::Integer, false), p("Hi", Ty::Integer, false)],
        Ty::Integer,
    );
    define_proc(checker, s::STD_MATH_RANDOMIZE, vec![]);

    define_builtin_std(
        checker,
        s::STD_MATH_ABS,
        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_MIN,
        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_MAX,
        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_SIGN,
        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
    define_builtin_std(
        checker,
        s::STD_MATH_CLAMP,
        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: vec![],
            return_type: Box::new(Ty::Error),
        }),
    );
}
