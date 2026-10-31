//! Pascal and Ada habits in `const` and `var` bindings: declaration sections, grouped names,
//! bindings without initializer, Pascal `=` initializers, bounded array types, Ada bindings
//! without keyword and Pascal declaration sections before a routine's `begin`.
//!
//! One binding item is parsed by [`Parser::parse_binding_item`]; canonical input yields the
//! canonical AST and no diagnostic. Deviations are recorded on the item and reported once for
//! the whole group by [`Parser::report_binding_habits`].
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/basics/variables.md`

use super::super::{ERROR_IDENT, Parser};
use crate::ast::{
    ConstDef, Decl, Expr, LocalBinding, QualifiedId, Stmt, TypeExpr, VarDef, Visibility,
};
use fpas_diagnostics::codes::{
    PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX, PARSE_MISSING_DECLARATION_KEYWORD,
};
use fpas_diagnostics::{Applicability, Fix, FixEdit, ForeignHabit, SourceSpan, fix_title};
use fpas_lexer::{Span, Token};

/// Declaration groups written with more than three bindings show only the first three.
const MAX_SHOWN_BINDINGS: usize = 3;

/// Binding keyword of a declaration group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parser) enum BindingKeyword {
    Const,
    Var,
}

impl BindingKeyword {
    fn text(self) -> &'static str {
        match self {
            Self::Const => "const",
            Self::Var => "var",
        }
    }
}

/// Where a group of binding items was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parser) enum BindingContext {
    /// After its own `const` or `var` keyword.
    Keyword,
    /// Without any keyword (Ada `X : integer := 1;`).
    Unprefixed,
    /// In a Pascal declaration section between a routine header and `begin`.
    RoutineSection,
}

/// One parsed binding item: `Name: Type := Value`, possibly in a foreign form.
pub(in crate::parser) struct BindingItem {
    names: Vec<String>,
    type_expr: Option<TypeExpr>,
    /// FPAS spelling of the type, when one is known.
    type_text: Option<String>,
    value: Expr,
    /// FPAS spelling of the initializer.
    value_text: String,
    span: Span,
    keyword: BindingKeyword,
    missing_initializer: bool,
    equals_initializer: bool,
    bounded_array: bool,
    /// Ada `Name : constant T := V`, recovered as `const`.
    ada_constant: bool,
    /// Ada aggregate `(X => 1, Y => 2)` or `(1, 2, 3)` as initializer.
    ada_aggregate: bool,
}

impl Parser {
    /// Parses one binding item without its terminator. `require_type` holds outside statement
    /// lists, where an omitted type is a foreign form.
    pub(in crate::parser) fn parse_binding_item(
        &mut self,
        keyword: BindingKeyword,
        require_type: bool,
    ) -> BindingItem {
        let start = self.current_span();
        let mut names = vec![self.expect_ident_or_error(start).0];
        while self.check(&Token::Comma) && matches!(self.peek_token(), Token::Ident(_)) {
            self.advance();
            names.push(self.expect_ident_or_error(start).0);
        }
        let mut bounded_array = false;
        let mut keyword = keyword;
        let mut ada_constant = false;
        let (type_expr, type_text) = if self.eat(&Token::Colon) {
            if matches!(self.current_token(), Token::Ident(word) if word.eq_ignore_ascii_case("constant"))
            {
                self.advance();
                keyword = BindingKeyword::Const;
                ada_constant = true;
            }
            let type_start = self.current_span();
            if self.check(&Token::Array) && self.peek_token() == &Token::LBracket {
                bounded_array = true;
                let (type_expr, text) = self.parse_bounded_array_type();
                (Some(type_expr), Some(text))
            } else {
                let type_expr = self.parse_type_expr();
                let text = self.source_text(self.span_from(type_start)).to_owned();
                (Some(type_expr), Some(text))
            }
        } else if self.check(&Token::Equal) {
            let type_text = require_type.then(|| Self::literal_type_name(self.peek_token()));
            (None, type_text)
        } else {
            if require_type || !self.check(&Token::ColonAssign) {
                self.expect(&Token::Colon);
            }
            (None, None)
        };
        let equals_initializer = self.eat(&Token::Equal);
        let mut missing_initializer = false;
        let (value, value_text, ada_aggregate) =
            if equals_initializer || self.eat(&Token::ColonAssign) {
                self.parse_binding_value(
                    type_expr.as_ref(),
                    type_text.as_deref(),
                    bounded_array || equals_initializer,
                )
            } else if matches!(self.current_token(), Token::Semicolon | Token::Eof) {
                missing_initializer = true;
                let default = Self::default_value_text(type_text.as_deref());
                (Expr::Error(self.current_span()), default, false)
            } else {
                self.expect(&Token::ColonAssign);
                (Expr::Error(self.current_span()), "…".to_owned(), false)
            };
        BindingItem {
            names,
            type_expr,
            type_text,
            value,
            value_text,
            span: self.span_from(start),
            keyword,
            missing_initializer,
            equals_initializer,
            bounded_array,
            ada_constant,
            ada_aggregate,
        }
    }

    /// Parses a binding item with its terminator, then any Pascal section continuation.
    pub(in crate::parser) fn parse_binding_items(
        &mut self,
        keyword: BindingKeyword,
        require_type: bool,
    ) -> Vec<BindingItem> {
        let mut items = Vec::new();
        loop {
            items.push(self.parse_binding_item(keyword, require_type));
            self.expect_semi();
            if !self.at_binding_continuation(keyword) {
                return items;
            }
        }
    }

    /// True when the next tokens continue a Pascal declaration section of `keyword`.
    pub(in crate::parser) fn at_binding_continuation(&self, keyword: BindingKeyword) -> bool {
        matches!(self.current_token(), Token::Ident(_))
            && match self.peek_token() {
                Token::Colon | Token::Comma => true,
                Token::Equal => keyword == BindingKeyword::Const,
                _ => false,
            }
    }

    /// Parses a Pascal declaration section between a routine header and `begin` as local
    /// bindings, reporting it once.
    pub(in crate::parser) fn parse_routine_declaration_section(&mut self) -> Vec<Stmt> {
        let start = self.current_span();
        let mut items = Vec::new();
        loop {
            let keyword = match self.current_token() {
                Token::Var => BindingKeyword::Var,
                Token::Const => BindingKeyword::Const,
                _ => break,
            };
            self.advance();
            items.extend(self.parse_binding_items(keyword, false));
        }
        let index =
            self.report_binding_habits(start, &items, BindingContext::RoutineSection, false);
        // The declarations move behind `begin`; a section followed by nested routines or
        // anything else keeps the diagnostic without a fix.
        if self.check(&Token::Begin) {
            let section = self.span_from(start);
            let begin = self.current_span();
            let insertion = Span {
                offset: begin.offset + begin.length,
                length: 0,
                ..begin
            };
            let declarations = Self::binding_declaration_texts(&items, false).join(" ");
            let fix = Self::bindings_fix_applicability(&items).and_then(|applicability| {
                Fix::new(
                    "Move the declarations behind `begin`",
                    applicability,
                    vec![
                        FixEdit {
                            span: SourceSpan::try_from(section).ok()?,
                            new_text: String::new(),
                        },
                        FixEdit {
                            span: SourceSpan::try_from(insertion).ok()?,
                            new_text: format!(" {declarations}"),
                        },
                    ],
                )
            });
            self.add_habit_fix(index, fix);
        }
        items
            .into_iter()
            .flat_map(Self::binding_statements)
            .collect()
    }

    /// Reports one diagnostic for a group of binding items when it contains a foreign form.
    pub(in crate::parser) fn report_binding_habits(
        &mut self,
        start: Span,
        items: &[BindingItem],
        context: BindingContext,
        public: bool,
    ) -> Option<usize> {
        let section = items.len() > 1;
        let grouped = items.iter().find(|item| item.names.len() > 1);
        let equals = items.iter().any(|item| item.equals_initializer);
        let bounded = items.iter().any(|item| item.bounded_array);
        let uninitialized = items.iter().find(|item| item.missing_initializer);
        let first = items.first()?;
        let keyword = first.keyword.text();
        let (code, message) = if context == BindingContext::RoutineSection {
            (
                PARSE_FOREIGN_SYNTAX,
                "Pascal declaration section before `begin` is not FPAS syntax".to_owned(),
            )
        } else if context == BindingContext::Unprefixed {
            (
                PARSE_MISSING_DECLARATION_KEYWORD,
                format!("Each declaration requires its own `{keyword}` keyword"),
            )
        } else if section {
            (
                PARSE_MISSING_DECLARATION_KEYWORD,
                format!(
                    "Pascal `{keyword}` section: each declaration requires its own `{keyword}` keyword"
                ),
            )
        } else if let Some(item) = grouped {
            (
                PARSE_MISSING_DECLARATION_KEYWORD,
                format!(
                    "Grouped names `{}` need one declaration each",
                    item.names.join(", ")
                ),
            )
        } else if bounded {
            (
                PARSE_FOREIGN_SYNTAX,
                "Pascal bounded array type is not FPAS syntax".to_owned(),
            )
        } else if equals {
            (
                PARSE_FOREIGN_SYNTAX,
                format!("Pascal `{keyword} Name = Value` is not FPAS syntax"),
            )
        } else if items.iter().any(|item| item.ada_aggregate) {
            (
                PARSE_FOREIGN_SYNTAX,
                "Ada aggregate is not FPAS syntax".to_owned(),
            )
        } else {
            let item = uninitialized?;
            (
                PARSE_EXPECTED_TOKEN,
                format!("`{}` has no initializer", item.names.join(", ")),
            )
        };
        let mut explanation = Vec::new();
        if context == BindingContext::RoutineSection {
            explanation.push("FPAS declares local bindings as statements after `begin`.");
        }
        if context == BindingContext::Unprefixed || section || grouped.is_some() {
            explanation.push("Every declaration repeats its keyword.");
            if public {
                explanation.push("Repeat `public` for every exported declaration.");
            }
        }
        if equals {
            explanation.push("Bindings use `:=`.");
            if items
                .iter()
                .any(|item| item.type_expr.is_none() && item.type_text.is_some())
            {
                explanation.push("Declarations outside statements name their type.");
            }
        }
        if bounded {
            explanation.push("Arrays have no bounds in the type and start at index 0.");
        }
        if items.iter().any(|item| item.ada_aggregate) {
            explanation.push(
                "Ada aggregates are array literals `[…]` or record constructors `Type(Field := …)`.",
            );
        }
        if items.iter().any(|item| item.ada_constant) {
            explanation.push("Ada `constant` is the `const` keyword.");
        }
        if uninitialized.is_some() {
            explanation.push("Every binding has an initializer.");
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = Self::bindings_fpas_form(items, public);
        let index = self.report_foreign_habit(
            code,
            &message,
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: &explanation.join(" "),
            },
            written,
            None,
        );
        if context != BindingContext::RoutineSection {
            let fix = Self::bindings_fix_applicability(items).and_then(|applicability| {
                Fix::replace(
                    fix_title(&replacement),
                    applicability,
                    SourceSpan::try_from(written).ok()?,
                    Self::binding_declaration_texts(items, public).join(" "),
                )
            });
            self.add_habit_fix(index, fix);
        }
        index
    }

    /// Converts an item to top-level declarations, one per name.
    pub(in crate::parser) fn binding_declarations(
        item: BindingItem,
        visibility: Visibility,
    ) -> Vec<Decl> {
        let type_expr = item.type_expr.unwrap_or_else(|| TypeExpr::Named {
            id: QualifiedId {
                parts: vec![ERROR_IDENT.to_owned()],
                span: item.span,
            },
            span: item.span,
        });
        item.names
            .into_iter()
            .map(|name| match item.keyword {
                BindingKeyword::Const => Decl::Const(ConstDef {
                    name,
                    type_expr: type_expr.clone(),
                    value: item.value.clone(),
                    visibility,
                    span: item.span,
                }),
                BindingKeyword::Var => Decl::Var(VarDef {
                    name,
                    type_expr: type_expr.clone(),
                    value: item.value.clone(),
                    visibility,
                    span: item.span,
                }),
            })
            .collect()
    }

    /// Converts an item to statement-local bindings, one per name.
    pub(in crate::parser) fn binding_statements(item: BindingItem) -> Vec<Stmt> {
        item.names
            .into_iter()
            .map(|name| {
                let binding = LocalBinding {
                    using: false,
                    name,
                    type_expr: item.type_expr.clone(),
                    value: item.value.clone(),
                    span: item.span,
                };
                match item.keyword {
                    BindingKeyword::Const => Stmt::Const(binding),
                    BindingKeyword::Var => Stmt::Var(binding),
                }
            })
            .collect()
    }

    /// Parses `array[Low..High] of T` as `array of T`; returns the type and its FPAS spelling.
    fn parse_bounded_array_type(&mut self) -> (TypeExpr, String) {
        let start = self.advance().span;
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::LBracket => depth += 1,
                Token::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                }
                Token::Semicolon => break,
                _ => {}
            }
            self.advance();
        }
        self.expect(&Token::Of);
        let element_start = self.current_span();
        let element = self.parse_type_expr();
        let element_text = self.source_text(self.span_from(element_start)).to_owned();
        (
            TypeExpr::Array(Box::new(element), self.span_from(start)),
            format!("array of {element_text}"),
        )
    }

    /// FPAS form of a binding group: one complete declaration per name.
    fn bindings_fpas_form(items: &[BindingItem], public: bool) -> String {
        let declarations = Self::binding_declaration_texts(items, public);
        let mut shown = declarations
            .iter()
            .take(MAX_SHOWN_BINDINGS)
            .cloned()
            .collect::<Vec<_>>();
        if declarations.len() > MAX_SHOWN_BINDINGS {
            shown.push("…".to_owned());
        }
        shown.join(" ")
    }

    /// Returns one FPAS declaration per bound name.
    fn binding_declaration_texts(items: &[BindingItem], public: bool) -> Vec<String> {
        let modifier = if public { "public " } else { "" };
        items
            .iter()
            .flat_map(|item| {
                item.names.iter().map(move |name| {
                    let keyword = item.keyword.text();
                    match &item.type_text {
                        Some(type_text) => {
                            format!(
                                "{modifier}{keyword} {name}: {type_text} := {};",
                                item.value_text
                            )
                        }
                        None => format!("{modifier}{keyword} {name} := {};", item.value_text),
                    }
                })
            })
            .collect()
    }

    /// Whether the FPAS form of `items` is a fix: guessed initializers and zero-based arrays
    /// need review, and placeholders rule a fix out.
    fn bindings_fix_applicability(items: &[BindingItem]) -> Option<Applicability> {
        let placeholder = items.iter().any(|item| {
            item.value_text.contains('…')
                || item
                    .type_text
                    .as_deref()
                    .is_some_and(|text| text == "TypeName")
        });
        if placeholder {
            None
        } else if items
            .iter()
            .any(|item| item.missing_initializer || item.bounded_array)
        {
            Some(Applicability::NeedsReview)
        } else {
            Some(Applicability::MachineApplicable)
        }
    }

    /// Type of a Pascal untyped constant, inferred from its literal.
    fn literal_type_name(literal: &Token) -> String {
        let name = match literal {
            Token::Integer(_) => "integer",
            Token::Real(_) => "real",
            Token::Str(_) => "string",
            Token::True | Token::False => "boolean",
            _ => "TypeName",
        };
        name.to_owned()
    }

    /// Initializer text suggested for a binding written without one.
    fn default_value_text(type_text: Option<&str>) -> String {
        let lower = type_text.unwrap_or_default().to_ascii_lowercase();
        let text = match lower.as_str() {
            "integer" => "0",
            "real" => "0.0",
            "string" => "''",
            "boolean" => "false",
            _ if lower.starts_with("array of") => "[]",
            _ if lower.starts_with("option of") => "None",
            _ => "…",
        };
        text.to_owned()
    }
}
