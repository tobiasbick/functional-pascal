//! Missing declaration keywords, with recovery that preserves written visibility.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (FP2015).

use crate::ast::{Decl, Stmt, Visibility};
use crate::parser::Parser;
use crate::parser::foreign::{BindingContext, BindingKeyword};
use fpas_diagnostics::codes::PARSE_MISSING_DECLARATION_KEYWORD;
use fpas_diagnostics::{Applicability, ForeignHabit};
use fpas_lexer::Token;

impl Parser {
    /// Diagnoses declarations without their keyword once per group and recovers them.
    pub(in crate::parser::decl) fn recover_unprefixed_declaration(
        &mut self,
        previous: Option<&Decl>,
        visibility: Visibility,
        allow_member_visibility: bool,
    ) -> Option<Vec<Decl>> {
        if !matches!(self.current_token(), Token::Ident(_)) {
            return None;
        }
        let keyword = match self.peek_token() {
            Token::Equal if matches!(previous, Some(Decl::Const(_))) => BindingKeyword::Const,
            Token::Equal => {
                let public = visibility == Visibility::Public
                    || previous
                        .is_some_and(|declaration| declaration.visibility() == Visibility::Public);
                return Some(self.recover_unprefixed_types(
                    visibility,
                    allow_member_visibility,
                    public,
                ));
            }
            Token::Colon | Token::Comma => match previous {
                Some(Decl::Const(_)) => BindingKeyword::Const,
                _ => BindingKeyword::Var,
            },
            _ => return None,
        };
        let public = visibility == Visibility::Public
            || previous.is_some_and(|declaration| declaration.visibility() == Visibility::Public);
        let start = self.current_span();
        let items = self.parse_binding_items(keyword, true);
        self.report_binding_habits(start, &items, BindingContext::Unprefixed, public);
        Some(
            items
                .into_iter()
                .flat_map(|item| Self::binding_declarations(item, visibility))
                .collect(),
        )
    }

    /// Recovers consecutive type definitions without their keyword and reports them once.
    fn recover_unprefixed_types(
        &mut self,
        visibility: Visibility,
        allow_member_visibility: bool,
        public: bool,
    ) -> Vec<Decl> {
        let start = self.current_span();
        let mut declarations = Vec::new();
        let mut texts = Vec::new();
        while matches!(self.current_token(), Token::Ident(_)) && self.peek_token() == &Token::Equal
        {
            let item_start = self.current_span();
            declarations.push(Decl::TypeDef(
                self.parse_type_def(visibility, allow_member_visibility),
            ));
            let modifier = if public { "public " } else { "" };
            texts.push(format!(
                "{modifier}type {}",
                self.source_text(self.span_from(item_start))
            ));
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = texts.join(" ");
        self.report_foreign_habit(
            PARSE_MISSING_DECLARATION_KEYWORD,
            "Each declaration requires its own `type` keyword",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "Every declaration repeats its keyword; repeat `public` for every exported declaration.",
            },
            written,
            Some(Applicability::MachineApplicable),
        );
        declarations
    }

    /// Recovers a local binding without keyword, using the preceding declaration keyword.
    pub(in crate::parser) fn recover_unprefixed_variable(
        &mut self,
        previous: Option<&Stmt>,
    ) -> Option<Stmt> {
        if !matches!(self.current_token(), Token::Ident(_))
            || !matches!(self.peek_token(), Token::Colon | Token::Comma)
        {
            return None;
        }
        let keyword = if matches!(previous, Some(Stmt::Const(_))) {
            BindingKeyword::Const
        } else {
            BindingKeyword::Var
        };
        let start = self.current_span();
        let item = self.parse_binding_item(keyword, false);
        self.expect_semi();
        self.report_binding_habits(
            start,
            std::slice::from_ref(&item),
            BindingContext::Unprefixed,
            false,
        );
        Self::binding_statements(item).into_iter().next()
    }
}
