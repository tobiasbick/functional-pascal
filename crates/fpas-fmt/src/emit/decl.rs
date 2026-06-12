//! Declarations (`const`, `var`, `type`, routines).

use fpas_parser::{
    ConstDef, Decl, EnumMember, EnumType, FieldDef, FuncBody, FunctionDecl, ProcedureDecl,
    RecordMethod, RecordType, TypeBody, TypeDef, VarDef, Visibility,
};

use super::Emitter;
use super::expr::emit_expr;
use super::stmt::emit_stmts_in_block;
use super::types::{emit_formal_params_in_parens, emit_type_expr, format_type_params};

/// Formats a declaration list (unit declarations or program `type` / top-level decls).
#[must_use]
pub(crate) fn format_decls(decls: &[Decl]) -> String {
    let mut emitter = Emitter::new();
    emit_decls(&mut emitter, decls);
    emitter.finish()
}

/// Appends formatted declarations to `emitter`.
pub(crate) fn emit_decls(emitter: &mut Emitter, decls: &[Decl]) {
    emit_decl_list(emitter, decls);
}

fn emit_decl_list(emitter: &mut Emitter, decls: &[Decl]) {
    let mut index = 0;
    while index < decls.len() {
        let run_end = decl_run_end(decls, index);
        emit_decl_run(emitter, &decls[index..run_end]);
        index = run_end;
        if index < decls.len() {
            emitter.blank_line();
        }
    }
}

fn decl_run_end(decls: &[Decl], start: usize) -> usize {
    let key = decl_run_key(&decls[start]);
    let mut end = start + 1;
    while end < decls.len() && decl_run_key(&decls[end]) == key {
        end += 1;
    }
    end
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclRunKind {
    Const,
    Var,
    MutableVar,
    Type,
    Routine,
}

/// Groups consecutive declarations for block emission (`const` / `var` / `type` sections).
#[derive(Clone, Copy, PartialEq, Eq)]
struct DeclRunKey {
    kind: DeclRunKind,
    /// `true` only for public `const` / `var` / `mutable var` / `type` lists.
    block: bool,
}

fn decl_run_key(decl: &Decl) -> DeclRunKey {
    let kind = decl_run_kind(decl);
    let block = matches!(
        kind,
        DeclRunKind::Const | DeclRunKind::Var | DeclRunKind::MutableVar | DeclRunKind::Type
    ) && decl.visibility() == Visibility::Public;
    DeclRunKey { kind, block }
}

fn decl_run_kind(decl: &Decl) -> DeclRunKind {
    match decl {
        Decl::Const(_) => DeclRunKind::Const,
        Decl::Var(_) => DeclRunKind::Var,
        Decl::MutableVar(_) => DeclRunKind::MutableVar,
        Decl::TypeDef(_) => DeclRunKind::Type,
        Decl::Function(_) | Decl::Procedure(_) => DeclRunKind::Routine,
    }
}

fn emit_decl_run(emitter: &mut Emitter, decls: &[Decl]) {
    let key = decl_run_key(&decls[0]);
    if !key.block {
        for (index, decl) in decls.iter().enumerate() {
            if index > 0 && decl_run_kind(decl) == DeclRunKind::Routine {
                emitter.blank_line();
            }
            emit_decl(emitter, decl, index + 1 == decls.len());
        }
        return;
    }

    match key.kind {
        DeclRunKind::Const => {
            emitter.writeln("const");
            emitter.with_indent(|inner| {
                for (index, decl) in decls.iter().enumerate() {
                    let Decl::Const(def) = decl else {
                        continue;
                    };
                    emit_const_def(inner, def, index + 1 == decls.len(), true);
                }
            });
        }
        DeclRunKind::Var => {
            emitter.writeln("var");
            emitter.with_indent(|inner| {
                for (index, decl) in decls.iter().enumerate() {
                    let Decl::Var(def) = decl else {
                        continue;
                    };
                    emit_var_def(inner, "var", def, index + 1 == decls.len());
                }
            });
        }
        DeclRunKind::MutableVar => {
            emitter.writeln("mutable var");
            emitter.with_indent(|inner| {
                for (index, decl) in decls.iter().enumerate() {
                    let Decl::MutableVar(def) = decl else {
                        continue;
                    };
                    emit_var_def(inner, "mutable var", def, index + 1 == decls.len());
                }
            });
        }
        DeclRunKind::Type => {
            emitter.writeln("type");
            emitter.with_indent(|inner| {
                for (index, decl) in decls.iter().enumerate() {
                    let Decl::TypeDef(def) = decl else {
                        continue;
                    };
                    emit_type_def(inner, def, index + 1 == decls.len());
                }
            });
        }
        DeclRunKind::Routine => {
            for (index, decl) in decls.iter().enumerate() {
                if index > 0 {
                    emitter.blank_line();
                }
                emit_decl(emitter, decl, index + 1 == decls.len());
            }
        }
    }
}

fn emit_decl(emitter: &mut Emitter, decl: &Decl, is_last: bool) {
    match decl {
        Decl::Const(def) => emit_const_def(emitter, def, is_last, false),
        Decl::Var(def) => emit_var_def(emitter, "var", def, is_last),
        Decl::MutableVar(def) => emit_var_def(emitter, "mutable var", def, is_last),
        Decl::TypeDef(def) => emit_type_def(emitter, def, is_last),
        Decl::Function(function) => emit_function_decl(emitter, function, is_last),
        Decl::Procedure(procedure) => emit_procedure_decl(emitter, procedure, is_last),
    }
}

fn emit_visibility(emitter: &mut Emitter, visibility: Visibility) {
    if visibility == Visibility::Private {
        emitter.write("private ");
    }
}

fn emit_const_def(emitter: &mut Emitter, def: &ConstDef, is_last: bool, in_const_block: bool) {
    write_decl_line_start(emitter);
    emit_visibility(emitter, def.visibility);
    if !in_const_block {
        emitter.write("const ");
    }
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0);
    finish_decl_line(emitter, is_last);
}

fn emit_var_def(emitter: &mut Emitter, keyword: &str, def: &VarDef, is_last: bool) {
    write_decl_line_start(emitter);
    emit_visibility(emitter, def.visibility);
    emitter.write(keyword);
    emitter.write(" ");
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0);
    finish_decl_line(emitter, is_last);
}

fn emit_type_def(emitter: &mut Emitter, def: &TypeDef, _is_last: bool) {
    write_decl_line_start(emitter);
    emit_visibility(emitter, def.visibility);
    emitter.write(&def.name);
    emitter.write(" = ");
    emit_type_body(emitter, &def.body);
    emitter.write(";\n");
}

fn emit_type_body(emitter: &mut Emitter, body: &TypeBody) {
    match body {
        TypeBody::Alias(type_expr) => emit_type_expr(emitter, type_expr),
        TypeBody::Record(record) => emit_record_type(emitter, record),
        TypeBody::Enum(enum_type) => emit_enum_type(emitter, enum_type),
    }
}

fn emit_record_type(emitter: &mut Emitter, record: &RecordType) {
    emitter.write("record\n");
    emitter.with_indent(|inner| {
        for field in &record.fields {
            emit_field_def(inner, field);
        }
        if !record.fields.is_empty() && !record.methods.is_empty() {
            inner.write("\n");
        }
        for (index, method) in record.methods.iter().enumerate() {
            if index > 0 {
                inner.write("\n");
            }
            emit_record_method(inner, method);
        }
    });
    write_decl_line_start(emitter);
    emitter.write("end");
}

fn emit_field_def(emitter: &mut Emitter, field: &FieldDef) {
    write_decl_line_start(emitter);
    emitter.write(&field.name);
    emitter.write(": ");
    emit_type_expr(emitter, &field.type_expr);
    if let Some(default_value) = &field.default_value {
        emitter.write(" := ");
        emit_expr(emitter, default_value, 0);
    }
    emitter.write(";\n");
}

fn emit_record_method(emitter: &mut Emitter, method: &RecordMethod) {
    match method {
        RecordMethod::Function(function) => {
            write_decl_line_start(emitter);
            emit_function_header(
                emitter,
                &function.name,
                &function.type_params,
                &function.params,
            );
            emitter.write(": ");
            emit_type_expr(emitter, &function.return_type);
            emitter.write(";\n");
            emit_func_body(emitter, &function.body);
        }
        RecordMethod::Procedure(procedure) => {
            write_decl_line_start(emitter);
            emit_procedure_header(
                emitter,
                &procedure.name,
                &procedure.type_params,
                &procedure.params,
            );
            emitter.write(";\n");
            emit_func_body(emitter, &procedure.body);
        }
    }
}

fn emit_enum_type(emitter: &mut Emitter, enum_type: &EnumType) {
    emitter.write("enum\n");
    emitter.with_indent(|inner| {
        for (index, member) in enum_type.members.iter().enumerate() {
            emit_enum_member(inner, member, index + 1 == enum_type.members.len());
        }
    });
    write_decl_line_start(emitter);
    emitter.write("end");
}

fn emit_enum_member(emitter: &mut Emitter, member: &EnumMember, _is_last: bool) {
    write_decl_line_start(emitter);
    emitter.write(&member.name);
    if !member.fields.is_empty() {
        emitter.write("(");
        for (index, field) in member.fields.iter().enumerate() {
            if index > 0 {
                emitter.write("; ");
            }
            emitter.write(&field.name);
            emitter.write(": ");
            emit_type_expr(emitter, &field.type_expr);
        }
        emitter.write(")");
    } else if let Some(value) = member.value {
        emitter.write(" = ");
        emitter.write(&value.to_string());
    }
    emitter.write(";\n");
}

fn emit_function_decl(emitter: &mut Emitter, function: &FunctionDecl, _is_last: bool) {
    write_decl_line_start(emitter);
    emit_visibility(emitter, function.visibility);
    emit_function_header(
        emitter,
        &function.name,
        &function.type_params,
        &function.params,
    );
    emitter.write(": ");
    emit_type_expr(emitter, &function.return_type);
    emitter.write(";\n");
    emit_func_body(emitter, &function.body);
}

fn emit_procedure_decl(emitter: &mut Emitter, procedure: &ProcedureDecl, _is_last: bool) {
    write_decl_line_start(emitter);
    emit_visibility(emitter, procedure.visibility);
    emit_procedure_header(
        emitter,
        &procedure.name,
        &procedure.type_params,
        &procedure.params,
    );
    emitter.write(";\n");
    emit_func_body(emitter, &procedure.body);
}

fn emit_function_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("function {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

fn emit_procedure_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("procedure {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

fn emit_func_body(emitter: &mut Emitter, body: &FuncBody) {
    let FuncBody::Block { nested, stmts } = body;
    for (index, decl) in nested.iter().enumerate() {
        emit_decl(emitter, decl, index + 1 == nested.len());
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts));
    write_decl_line_start(emitter);
    emitter.write("end;\n");
}

fn write_decl_line_start(emitter: &mut Emitter) {
    for _ in 0..emitter.indent_level() {
        emitter.write(crate::style::INDENT);
    }
}

fn finish_decl_line(emitter: &mut Emitter, _is_last: bool) {
    emitter.write(";\n");
}

#[cfg(test)]
mod tests {
    use super::format_decls;
    use fpas_parser::parse_compilation_unit;

    fn format_unit_decls(source: &str) -> String {
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let fpas_parser::CompilationUnit::Unit(unit) = unit else {
            panic!("expected unit");
        };
        format_decls(&unit.declarations)
    }

    fn format_program_decls(source: &str) -> String {
        let (program, errors) = fpas_parser::parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        format_decls(&program.declarations)
    }

    #[test]
    fn record_one_field() {
        let formatted =
            format_program_decls("program T; type IdBox = record Value: integer; end; begin end.");
        assert_eq!(
            formatted,
            "type\n  IdBox = record\n    Value: integer;\n  end;\n"
        );
    }

    #[test]
    fn record_five_fields() {
        let formatted = format_program_decls(
            "program T; type Person = record Id: integer; Name: string; Age: integer; Active: boolean; Score: real; end; begin end.",
        );
        assert!(formatted.contains("Person = record\n"));
        assert!(formatted.contains("Id: integer;\n"));
        assert!(formatted.contains("Score: real;\n"));
    }

    #[test]
    fn record_with_defaults_and_methods() {
        let formatted = format_program_decls(
            "program T;
type
  Point = record
    X: integer;
    Y: integer;
    function Sum(Self: Point): integer;
    begin
      return Self.X + Self.Y
    end;
  end;
begin
end.",
        );
        assert!(
            formatted.contains("X: integer;\n    Y: integer;\n\n    function Sum"),
            "formatted:\n{formatted}"
        );
        assert!(formatted.contains("return Self.X + Self.Y"));
        assert!(formatted.contains("end;\n  end;\n"));
    }

    #[test]
    fn enum_and_alias() {
        let formatted = format_program_decls(
            "program T; type Color = enum Red; Green; Blue; end; IntAlias = integer; begin end.",
        );
        assert!(formatted.contains("Color = enum\n    Red;\n    Green;\n    Blue;\n  end;\n"));
        assert!(formatted.contains("IntAlias = integer;\n"));
    }

    #[test]
    fn unit_function_with_private() {
        let formatted = format_unit_decls(
            "unit MyApp.Utils; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then begin return Min end else begin return Value end end; private function Hidden(): integer; begin return 0 end;",
        );
        assert!(formatted.contains("function Clamp"));
        assert!(formatted.contains("private function Hidden"));
    }

    #[test]
    fn unit_private_vars_and_consts_are_not_block_grouped() {
        let formatted = format_unit_decls(
            "unit U; private mutable var A: integer := 1; private mutable var B: integer := 2; private const C: integer := 3; private const D: integer := 4;",
        );
        assert!(formatted.contains("private mutable var A: integer := 1;\n"));
        assert!(formatted.contains("private const C: integer := 3;\n"));
        assert!(
            !formatted.contains("mutable var\n"),
            "formatted:\n{formatted}"
        );
        assert!(
            !formatted.contains("const\n  private"),
            "formatted:\n{formatted}"
        );
    }

    #[test]
    fn unit_shell_private_state_round_trip() {
        let source = include_str!("../../../../apps/ide/src/shell.fpas");
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let fpas_parser::CompilationUnit::Unit(unit) = unit else {
            panic!("expected unit");
        };
        let formatted = format_decls(&unit.declarations);
        let (_, errors) = parse_compilation_unit(&format!(
            "unit Ide.Shell;\nuses Ide.Menu, Ide.Status, Ide.Theme, Std.Console, Std.Tui;\n\n{formatted}"
        ));
        assert!(
            errors.is_empty(),
            "{errors:?}\n--- formatted ---\n{formatted}"
        );
        assert!(formatted.contains("private mutable var DesktopView"));
        assert!(!formatted.contains("mutable var\n  private"));
    }
}
