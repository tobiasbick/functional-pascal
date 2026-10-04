//! Contextual instantiation and complete signatures for intrinsic callbacks.
//!
//! **Documentation:** `docs/pascal/std/collections/array/higher-order.md`.

use crate::check::Checker;
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

/// Check original argument nodes once before specialized result derivation.
pub(super) fn prepare(c: &mut Checker, name: &str, args: &[&Expr]) -> Vec<(usize, Option<Ty>)> {
    let index = match name {
        s::STD_ARRAY_REDUCE | s::STD_DICT_REDUCE | s::STD_STR_REDUCE => 2,
        s::STD_ARRAY_MAP
        | s::STD_ARRAY_FILTER
        | s::STD_ARRAY_FLAT_MAP
        | s::STD_ARRAY_FIND
        | s::STD_ARRAY_FIND_INDEX
        | s::STD_ARRAY_ANY
        | s::STD_ARRAY_ALL
        | s::STD_ARRAY_FOR_EACH
        | s::STD_DICT_MAP
        | s::STD_DICT_FILTER
        | s::STD_STR_MAP
        | s::STD_STR_FILTER
        | s::STD_OPTION_MAP
        | s::STD_OPTION_AND_THEN
        | s::STD_OPTION_OR_ELSE
        | s::STD_RESULT_MAP
        | s::STD_RESULT_AND_THEN
        | s::STD_RESULT_OR_ELSE => 1,
        _ => return Vec::new(),
    };
    if args.len() != index + 1 {
        return Vec::new();
    }
    let mut previous = Vec::new();
    let mut types = Vec::new();
    for arg in &args[..index] {
        let ty = c.check_expr(arg);
        previous.push((
            Checker::expr_lookup_key(arg),
            c.prechecked_arguments
                .insert(Checker::expr_lookup_key(arg), ty.clone()),
        ));
        types.push(ty);
    }
    let (parameters, result) = expected_callback(name, &types);
    let params = parameters
        .into_iter()
        .enumerate()
        .map(|(index, ty)| ParamTy {
            name: format!("Value{}", index + 1),
            mutable: false,
            ty,
        })
        .collect();
    let expected = if name == s::STD_ARRAY_FOR_EACH {
        Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params,
            variadic: false,
        })
    } else {
        Ty::Function(FunctionTy {
            pure: true,
            type_params: Vec::new(),
            params,
            return_type: Box::new(result),
            variadic: false,
        })
    };
    let actual = c.check_expr_with_expected(args[index], &expected);
    c.check_type_compat(
        &expected,
        &actual,
        &format!("callback of `{name}`"),
        args[index].span(),
    );
    let key = Checker::expr_lookup_key(args[index]);
    previous.push((key, c.prechecked_arguments.insert(key, actual)));
    previous
}

/// Restore temporary checked-expression entries after one intrinsic call.
pub(super) fn restore(c: &mut Checker, previous: Vec<(usize, Option<Ty>)>) {
    for (key, value) in previous {
        if let Some(value) = value {
            c.prechecked_arguments.insert(key, value);
        } else {
            c.prechecked_arguments.remove(&key);
        }
    }
}

fn expected_callback(name: &str, types: &[Ty]) -> (Vec<Ty>, Ty) {
    let input = &types[0];
    let element = match input {
        Ty::Array(inner) | Ty::Option(inner) | Ty::Result(inner, _) => (**inner).clone(),
        Ty::Dict(_, value) => (**value).clone(),
        Ty::String => Ty::String,
        _ => Ty::Error,
    };
    let key = match input {
        Ty::Dict(key, _) => (**key).clone(),
        _ => Ty::Error,
    };
    let error = match input {
        Ty::Result(_, error) => (**error).clone(),
        _ => Ty::Error,
    };
    match name {
        s::STD_ARRAY_FILTER
        | s::STD_ARRAY_FIND
        | s::STD_ARRAY_FIND_INDEX
        | s::STD_ARRAY_ANY
        | s::STD_ARRAY_ALL
        | s::STD_STR_FILTER => (vec![element], Ty::Boolean),
        s::STD_ARRAY_FLAT_MAP => (vec![element], Ty::Array(Box::new(Ty::Error))),
        s::STD_ARRAY_REDUCE | s::STD_STR_REDUCE => {
            (vec![types[1].clone(), element], types[1].clone())
        }
        s::STD_DICT_FILTER => (vec![key, element], Ty::Boolean),
        s::STD_DICT_REDUCE => (vec![types[1].clone(), key, element], types[1].clone()),
        s::STD_STR_MAP => (vec![Ty::String], Ty::String),
        s::STD_OPTION_AND_THEN => (vec![element], Ty::Option(Box::new(Ty::Error))),
        s::STD_OPTION_OR_ELSE => (vec![], input.clone()),
        s::STD_RESULT_AND_THEN => (
            vec![element],
            Ty::Result(Box::new(Ty::Error), Box::new(error)),
        ),
        s::STD_RESULT_OR_ELSE => (
            vec![error],
            Ty::Result(Box::new(element), Box::new(Ty::Error)),
        ),
        _ => (vec![element], Ty::Error),
    }
}
