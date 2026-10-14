//! Type expressions and formal parameters.

use fpas_parser::{FormalParam, ParamMode, QualifiedId, TypeExpr, TypeParam};

use super::Emitter;

/// Formats a type expression.
#[must_use]
pub(crate) fn format_type_expr(ty: &TypeExpr) -> String {
    let mut emitter = Emitter::new();
    emit_type_expr(&mut emitter, ty);
    emitter.finish()
}

/// Formats generic type parameters (`<T>`, `<T: Comparable>`).
#[must_use]
pub(crate) fn format_type_params(params: &[TypeParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut emitter = Emitter::new();
    emitter.write("<");
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_type_param(&mut emitter, param);
    }
    emitter.write(">");
    emitter.finish()
}

/// Emits the canonical type spelling, including parenthesized `Result` arguments.
pub(crate) fn emit_type_expr(emitter: &mut Emitter, ty: &TypeExpr) {
    match ty {
        TypeExpr::Named { id, .. } => emit_qualified_id(emitter, id),
        TypeExpr::Application { id, arguments, .. } => {
            emit_qualified_id(emitter, id);
            emitter.write(" of ");
            if arguments.len() > 1 {
                emitter.write("(");
            }
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    emitter.write(", ");
                }
                emit_type_expr(emitter, argument);
            }
            if arguments.len() > 1 {
                emitter.write(")");
            }
        }
        TypeExpr::Array(inner, ..) => {
            emitter.write("array of ");
            emit_type_expr(emitter, inner);
        }
        TypeExpr::Channel(inner, ..) => {
            emitter.write("channel of ");
            emit_type_expr(emitter, inner);
        }
        TypeExpr::Task(inner, ..) => {
            emitter.write("task of ");
            emit_type_expr(emitter, inner);
        }
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => {
            emit_formal_params_in_parens(emitter, "function(", params, "");
            emitter.write(": ");
            emit_type_expr(emitter, return_type);
        }
        TypeExpr::ProcedureType { params, .. } => {
            emit_formal_params_in_parens(emitter, "procedure(", params, "");
        }
        TypeExpr::Result {
            ok_type, err_type, ..
        } => {
            emitter.write("result of (");
            emit_type_expr(emitter, ok_type);
            emitter.write(", ");
            emit_type_expr(emitter, err_type);
            emitter.write(")");
        }
        TypeExpr::Option { inner_type, .. } => {
            emitter.write("option of ");
            emit_type_expr(emitter, inner_type);
        }
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => {
            emitter.write("dict of ");
            emit_type_expr(emitter, key_type);
            emitter.write(" to ");
            emit_type_expr(emitter, value_type);
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

/// Emits formal parameters, preserving their authored semicolon-separated line groups.
pub(crate) fn emit_formal_params_in_parens(
    emitter: &mut Emitter,
    open_prefix: &str,
    params: &[FormalParam],
    close_suffix: &str,
) {
    let group = params
        .first()
        .zip(params.last())
        .and_then(|(first, last)| emitter.layout.enclosing(first.span, last.span));
    let multiline = group.is_some_and(|g| emitter.layout.breaks_between(g.open, g.close));
    let base = emitter.line_indent();
    emitter.write(open_prefix);
    if multiline {
        emitter.write_line_end();
    }
    emitter.at_indent(base + crate::style::INDENT_WIDTH, |inner| {
        for (index, param) in params.iter().enumerate() {
            if index == 0 && multiline {
                inner.write_current_indent();
            }
            if index > 0 {
                inner.write(";");
                if inner.layout.break_before(param.span.offset) {
                    inner.write_line_end();
                    inner.write_current_indent();
                } else {
                    inner.write(" ");
                }
            }
            emit_formal_param(inner, param);
        }
    });
    if multiline {
        emitter.newline_to_indent(base);
    }
    emitter.write(")");
    emitter.write(close_suffix);
}

fn emit_formal_param(emitter: &mut Emitter, param: &FormalParam) {
    if param.mode == ParamMode::Var {
        emitter.write("var ");
    }
    emitter.write(&param.name);
    emitter.write(": ");
    emit_type_expr(emitter, &param.type_expr);
}

#[cfg(test)]
mod tests {
    use super::format_type_expr;
    use fpas_parser::parse;

    fn type_from_const(source: &str) -> String {
        use fpas_parser::Stmt;

        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let Stmt::Const(binding) = &program.body[0] else {
            panic!("expected const stmt");
        };
        format_type_expr(binding.type_expr.as_ref().expect("written annotation"))
    }

    #[test]
    fn named_and_array_types() {
        assert_eq!(
            type_from_const("program T; begin const X: integer := 0; end."),
            "integer"
        );
        assert_eq!(
            type_from_const("program T; begin const X: MyLib.Utils.Id := 0; end."),
            "MyLib.Utils.Id"
        );
        assert_eq!(
            type_from_const("program T; begin const X: array of integer := []; end."),
            "array of integer"
        );
    }

    #[test]
    fn result_option_dict_types() {
        assert_eq!(
            type_from_const("program T; begin const X: result of (integer, string) := Ok(0); end."),
            "result of (integer, string)"
        );
        assert_eq!(
            type_from_const("program T; begin const X: option of integer := None; end."),
            "option of integer"
        );
        assert_eq!(
            type_from_const("program T; begin const X: dict of string to integer := [:]; end."),
            "dict of string to integer"
        );
        assert_eq!(
            type_from_const("program T; begin const X: channel of string := Value; end."),
            "channel of string"
        );
        assert_eq!(
            type_from_const(
                "program T; begin const X: array of TASK OF result of (integer, string) := []; end."
            ),
            "array of task of result of (integer, string)"
        );
        assert_eq!(
            type_from_const("program T; begin const X: task := Value; end."),
            "task"
        );
    }

    #[test]
    fn function_and_procedure_types() {
        assert_eq!(
            type_from_const("program T; begin const F: function(X: integer): integer := Add; end."),
            "function(X: integer): integer"
        );
        assert_eq!(
            type_from_const(
                "program T; begin const F: procedure(var   X: integer; Y: integer) := Step; end."
            ),
            "procedure(var X: integer; Y: integer)"
        );
        assert_eq!(
            type_from_const("program T; begin const P: procedure(Msg: string) := WriteLn; end."),
            "procedure(Msg: string)"
        );
        assert_eq!(
            type_from_const(
                "program T; begin const F: function(A: integer; B: integer): boolean := Check; end."
            ),
            "function(A: integer; B: integer): boolean"
        );
    }

    #[test]
    fn ast_only_long_formal_param_list_stays_compact() {
        assert_eq!(
            type_from_const(
                "program T; begin const F: function(AlphaParameter: integer; BetaParameter: integer; GammaParameter: integer; DeltaParameter: integer): boolean := Check; end.",
            ),
            "function(AlphaParameter: integer; BetaParameter: integer; GammaParameter: integer; DeltaParameter: integer): boolean"
        );
    }
}
