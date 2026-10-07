//! Missing declaration keywords, with recovery that preserves written visibility.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (FP2015).

use crate::ast::{Decl, Stmt, Visibility};
use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_MISSING_DECLARATION_KEYWORD;
use fpas_lexer::Token;

impl Parser {
    /// Diagnoses a removed declaration group and recovers its next definition.
    pub(in crate::parser::decl) fn recover_unprefixed_declaration(
        &mut self,
        previous: Option<&Decl>,
        visibility: Visibility,
        allow_member_visibility: bool,
    ) -> Option<Decl> {
        if !matches!(self.current_token(), Token::Ident(_)) {
            return None;
        }
        let keyword = match self.peek_token() {
            Token::Equal => "type",
            Token::Colon => match previous {
                Some(Decl::Const(_)) => "const",
                Some(Decl::MutableVar(_)) => "mutable var",
                _ => "var",
            },
            _ => return None,
        };
        let public = visibility == Visibility::Public
            || previous.is_some_and(|declaration| declaration.visibility() == Visibility::Public);
        self.report_missing_declaration_keyword(keyword, public);
        Some(match keyword {
            "type" => Decl::TypeDef(self.parse_type_def(visibility, allow_member_visibility)),
            "const" => Decl::Const(self.parse_const_def(visibility)),
            "mutable var" => Decl::MutableVar(self.parse_var_def(visibility)),
            _ => Decl::Var(self.parse_var_def(visibility)),
        })
    }

    /// Recovers a local binding without extending local declaration kinds.
    pub(in crate::parser) fn recover_unprefixed_variable(
        &mut self,
        previous: Option<&Stmt>,
    ) -> Option<Stmt> {
        if !matches!(self.current_token(), Token::Ident(_)) || self.peek_token() != &Token::Colon {
            return None;
        }
        let mutable = matches!(previous, Some(Stmt::MutableVar(_)));
        self.report_missing_declaration_keyword(if mutable { "mutable var" } else { "var" }, false);
        let definition = self.parse_var_def(Visibility::Private);
        Some(if mutable {
            Stmt::MutableVar(definition)
        } else {
            Stmt::Var(definition)
        })
    }

    fn report_missing_declaration_keyword(&mut self, keyword: &str, public: bool) {
        let Token::Ident(name) = self.current_token() else {
            return;
        };
        let modifier = if public { "public " } else { "" };
        let example = if keyword == "type" {
            format!("{modifier}type {name} = integer;")
        } else {
            format!("{modifier}{keyword} {name}: integer := 0;")
        };
        self.error_with_code(
            PARSE_MISSING_DECLARATION_KEYWORD,
            &format!("Each declaration requires its own `{keyword}` keyword"),
            &format!("Repeat the complete declaration prefix: `{example}`. Repeat `public` for every exported declaration."),
            self.current_span(),
        );
    }
}
