//! Type expressions and formal parameters.

use fpas_parser::{FormalParam, QualifiedId, TypeExpr, TypeParam};

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

pub(crate) fn emit_type_expr(emitter: &mut Emitter, ty: &TypeExpr) {
    match ty {
        TypeExpr::Named { id, .. } => emit_qualified_id(emitter, id),
        TypeExpr::Array(inner, ..) => {
            emitter.write("array of ");
            emit_type_expr(emitter, inner);
        }
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => {
            emitter.write("function(");
            emit_formal_params(emitter, params);
            emitter.write("): ");
            emit_type_expr(emitter, return_type);
        }
        TypeExpr::ProcedureType { params, .. } => {
            emitter.write("procedure(");
            emit_formal_params(emitter, params);
            emitter.write(")");
        }
        TypeExpr::Result {
            ok_type, err_type, ..
        } => {
            emitter.write("result of ");
            emit_type_expr(emitter, ok_type);
            emitter.write(", ");
            emit_type_expr(emitter, err_type);
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

pub(crate) fn emit_formal_params(emitter: &mut Emitter, params: &[FormalParam]) {
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            emitter.write("; ");
        }
        emit_formal_param(emitter, param);
    }
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
            type_from_var("program T; begin var X: integer := 0; end."),
            "integer"
        );
        assert_eq!(
            type_from_var("program T; begin var X: MyLib.Utils.Id := 0; end."),
            "MyLib.Utils.Id"
        );
        assert_eq!(
            type_from_var("program T; begin var X: array of integer := []; end."),
            "array of integer"
        );
    }

    #[test]
    fn result_option_dict_types() {
        assert_eq!(
            type_from_var("program T; begin var X: result of integer, string := Ok(0); end."),
            "result of integer, string"
        );
        assert_eq!(
            type_from_var("program T; begin var X: option of integer := None; end."),
            "option of integer"
        );
        assert_eq!(
            type_from_var("program T; begin var X: dict of string to integer := [:]; end."),
            "dict of string to integer"
        );
    }

    #[test]
    fn function_and_procedure_types() {
        assert_eq!(
            type_from_var("program T; begin var F: function(X: integer): integer := Add; end."),
            "function(X: integer): integer"
        );
        assert_eq!(
            type_from_var("program T; begin var P: procedure(Msg: string) := WriteLn; end."),
            "procedure(Msg: string)"
        );
        assert_eq!(
            type_from_var(
                "program T; begin var F: function(A: integer; mutable B: integer): boolean := Check; end."
            ),
            "function(A: integer; mutable B: integer): boolean"
        );
    }
}
