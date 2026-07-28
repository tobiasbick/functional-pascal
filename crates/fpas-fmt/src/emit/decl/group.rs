//! Declaration block grouping (const / var / type sections).

use fpas_parser::{Decl, Visibility};

use crate::comments::{CommentMap, emit_leading_comments};

use super::super::Emitter;
use super::item::{emit_const_def, emit_decl, emit_type_def, emit_var_def};

/// Appends formatted declarations to `emitter`.
pub(crate) fn emit_decls(emitter: &mut Emitter, decls: &[Decl], comments: &CommentMap) {
    let mut index = 0;
    while index < decls.len() {
        let run_end = decl_run_end(decls, index);
        emit_decl_run(emitter, &decls[index..run_end], comments);
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
    /// `true` for `const` / `var` / `mutable var` / `type` lists.
    block: bool,
    visibility: Visibility,
}

fn decl_run_key(decl: &Decl) -> DeclRunKey {
    let kind = decl_run_kind(decl);
    let block = matches!(
        kind,
        DeclRunKind::Const | DeclRunKind::Var | DeclRunKind::MutableVar | DeclRunKind::Type
    );
    DeclRunKey {
        kind,
        block,
        visibility: decl.visibility(),
    }
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

fn emit_decl_run(emitter: &mut Emitter, decls: &[Decl], comments: &CommentMap) {
    let key = decl_run_key(&decls[0]);
    if !key.block {
        for (index, decl) in decls.iter().enumerate() {
            if index > 0 && decl_run_kind(decl) == DeclRunKind::Routine {
                emitter.blank_line();
            }
            emit_decl(emitter, decl, comments);
        }
        return;
    }

    match key.kind {
        DeclRunKind::Const => {
            emit_block_header(emitter, key.visibility, "const");
            emitter.with_indent(|inner| {
                for decl in decls {
                    let Decl::Const(def) = decl else {
                        continue;
                    };
                    emit_leading_comments(inner, comments, def.span.offset, false);
                    emit_const_def(inner, def, true, comments);
                }
            });
        }
        DeclRunKind::Var => {
            emit_block_header(emitter, key.visibility, "var");
            emitter.with_indent(|inner| {
                for decl in decls {
                    let Decl::Var(def) = decl else {
                        continue;
                    };
                    emit_leading_comments(inner, comments, def.span.offset, false);
                    emit_var_def(inner, "var", def, true, comments);
                }
            });
        }
        DeclRunKind::MutableVar => {
            emit_block_header(emitter, key.visibility, "mutable var");
            emitter.with_indent(|inner| {
                for decl in decls {
                    let Decl::MutableVar(def) = decl else {
                        continue;
                    };
                    emit_leading_comments(inner, comments, def.span.offset, false);
                    emit_var_def(inner, "mutable var", def, true, comments);
                }
            });
        }
        DeclRunKind::Type => {
            emit_block_header(emitter, key.visibility, "type");
            emitter.with_indent(|inner| {
                for decl in decls {
                    let Decl::TypeDef(def) = decl else {
                        continue;
                    };
                    emit_leading_comments(inner, comments, def.span.offset, false);
                    emit_type_def(inner, def, comments, true);
                }
            });
        }
        DeclRunKind::Routine => unreachable!("block grouping excludes routines"),
    }
}

fn emit_block_header(emitter: &mut Emitter, visibility: Visibility, keyword: &str) {
    if visibility == Visibility::Public {
        emitter.write("public ");
    }
    emitter.writeln(keyword);
}
