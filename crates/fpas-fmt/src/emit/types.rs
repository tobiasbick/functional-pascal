//! Type expressions and formal parameters.

use fpas_parser::{FormalParam, QualifiedId, TypeExpr, TypeParam};

use super::Emitter;
use super::wrap::{emit_wrapped_semicolon_paren_list, measure_emit};

/// Formats a type expression.
#[must_use]
pub(crate) fn format_type_expr(ty: &TypeExpr) -> String {
    let mut emitter = Emitter::new();
    emit_type_expr(&mut emitter, ty);
    emitter.finish()
}

/// Formats canonical generic type parameters (` of (T: Comparable)`).
#[must_use]
pub(crate) fn format_type_params(params: &[TypeParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut emitter = Emitter::new();
    emitter.write(" of (");
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_type_param(&mut emitter, param);
    }
    emitter.write(")");
    emitter.finish()
}

pub(crate) fn emit_type_expr(emitter: &mut Emitter, ty: &TypeExpr) {
    match ty {
        TypeExpr::Named { id, arguments, .. } => {
            emit_qualified_id(emitter, id);
            if !arguments.is_empty() {
                emitter.write(" of (");
                for (index, argument) in arguments.iter().enumerate() {
                    if index > 0 {
                        emitter.write(", ");
                    }
                    emit_type_expr(emitter, argument);
                }
                emitter.write(")");
            }
        }
        TypeExpr::Array(inner, ..) => {
            emitter.write("array of (");
            emit_type_expr(emitter, inner);
            emitter.write(")");
        }
        TypeExpr::Channel(inner, ..) => {
            emitter.write("channel of (");
            emit_type_expr(emitter, inner);
            emitter.write(")");
        }
        TypeExpr::Task(inner, ..) => {
            emitter.write("task of (");
            emit_type_expr(emitter, inner);
            emitter.write(")");
        }
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => {
            let return_text = format_type_expr(return_type);
            emit_formal_params_in_parens(emitter, "function(", params, &format!(": {return_text}"));
        }
        TypeExpr::ProcedureType { params, .. } => {
            emit_formal_params_in_parens(emitter, "procedure(", params, "");
        }
        TypeExpr::Result {
            ok_type, err_type, ..
        } => {
            emitter.write("Result of (");
            emit_type_expr(emitter, ok_type);
            emitter.write(", ");
            emit_type_expr(emitter, err_type);
            emitter.write(")");
        }
        TypeExpr::Option { inner_type, .. } => {
            emitter.write("Option of (");
            emit_type_expr(emitter, inner_type);
            emitter.write(")");
        }
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => {
            emitter.write("dict of (");
            emit_type_expr(emitter, key_type);
            emitter.write(", ");
            emit_type_expr(emitter, value_type);
            emitter.write(")");
        }
    }
}

pub(crate) fn emit_qualified_id(emitter: &mut Emitter, id: &QualifiedId) {
    for (index, part) in id.parts.iter().enumerate() {
        if index > 0 {
            emitter.write(".");
        }
        emitter.write(part);
    }
}

pub(crate) fn emit_type_param(emitter: &mut Emitter, param: &TypeParam) {
    emitter.write(&param.name);
    if let Some(constraint) = &param.constraint {
        emitter.write(": ");
        emitter.write(constraint);
    }
}

/// Emits formal parameters inside parentheses, wrapping after `;` when over max width.
pub(crate) fn emit_formal_params_in_parens(
    emitter: &mut Emitter,
    open_prefix: &str,
    params: &[FormalParam],
    close_suffix: &str,
) {
    let items: Vec<String> = params
        .iter()
        .map(|param| measure_emit(|inner| emit_formal_param(inner, param)))
        .collect();
    emit_wrapped_semicolon_paren_list(emitter, open_prefix, &items, close_suffix);
}

fn emit_formal_param(emitter: &mut Emitter, param: &FormalParam) {
    if param.mutable {
        emitter.write("mutable ");
    }
    emitter.write(&param.name);
    emitter.write(": ");
    emit_type_expr(emitter, &param.type_expr);
}

#[cfg(test)]
mod tests {
    use super::format_type_expr;
    use fpas_parser::parse;

    fn type_from_var(source: &str) -> String {
        use fpas_parser::Stmt;

        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let Stmt::Var(var) = &program.body[0] else {
            panic!("expected var stmt");
        };
        format_type_expr(&var.type_expr)
    }

    #[test]
    fn named_and_array_types() {
        assert_eq!(
            type_from_var(r#"program T; begin var X: integer := 0; end program;"#),
            "integer"
        );
        assert_eq!(
            type_from_var(r#"program T; begin var X: MyLib.Utils.Id := 0; end program;"#),
            "MyLib.Utils.Id"
        );
        assert_eq!(
            type_from_var(r#"program T; begin var X: array of (integer) := []; end program;"#),
            "array of (integer)"
        );
    }

    #[test]
    fn result_option_dict_types() {
        assert_eq!(
            type_from_var(
                r#"program T; begin var X: result of (integer, string) := Result.Ok(0); end program;"#
            ),
            "Result of (integer, string)"
        );
        assert_eq!(
            type_from_var(
                r#"program T; begin var X: option of (integer) := Option.None; end program;"#
            ),
            "Option of (integer)"
        );
        assert_eq!(
            type_from_var(
                r#"program T; begin var X: dict of (string, integer) := [:]; end program;"#
            ),
            "dict of (string, integer)"
        );
        assert_eq!(
            type_from_var(r#"program T; begin var X: channel of (string) := Value; end program;"#),
            "channel of (string)"
        );
        assert_eq!(
            type_from_var(
                r#"program T; begin var X: array of (TASK OF (result of (integer, string))) := []; end program;"#
            ),
            "array of (task of (Result of (integer, string)))"
        );
        assert_eq!(
            type_from_var(r#"program T; begin var X: task := Value; end program;"#),
            "task"
        );
    }

    #[test]
    fn function_and_procedure_types() {
        assert_eq!(
            type_from_var(
                r#"program T; begin var F: function(X: integer): integer := Add; end program;"#
            ),
            "function(X: integer): integer"
        );
        assert_eq!(
            type_from_var(
                r#"program T; begin var P: procedure(Msg: string) := WriteLn; end program;"#
            ),
            "procedure(Msg: string)"
        );
        assert_eq!(
            type_from_var(
                r#"program T; begin var F: function(A: integer; mutable B: integer): boolean := Check; end program;"#
            ),
            "function(A: integer; mutable B: integer): boolean"
        );
    }

    #[test]
    fn long_formal_param_list_wraps() {
        assert_eq!(
            type_from_var(
                r#"program T; begin var F: function(AlphaParameter: integer; BetaParameter: integer; GammaParameter: integer; DeltaParameter: integer): boolean := Check; end program;"#,
            ),
            "function(\n  AlphaParameter: integer;\n  BetaParameter: integer;\n  GammaParameter: integer;\n  DeltaParameter: integer\n): boolean"
        );
    }
}
