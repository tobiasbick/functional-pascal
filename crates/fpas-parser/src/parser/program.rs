use super::Parser;
use crate::ast::*;
use crate::error::ParseError;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    pub fn parse_compilation_unit(mut self) -> (CompilationUnit, Vec<ParseError>) {
        let unit = match self.current_token() {
            Token::Program => CompilationUnit::Program(self.parse_program_ast()),
            Token::Unit => CompilationUnit::Unit(self.parse_unit_ast()),
            _ => {
                let span = self.current_span();
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    &format!(
                        "Expected `program` or `unit`, found `{}`",
                        super::token_display(self.current_token())
                    ),
                    "Start the file with `program <name>;` or `unit <name>;`.",
                    span,
                );
                CompilationUnit::Program(self.parse_program_ast())
            }
        };
        (unit, self.errors)
    }

    pub fn parse_program(mut self) -> (Program, Vec<ParseError>) {
        let program = self.parse_program_ast();
        (program, self.errors)
    }

    fn parse_program_ast(&mut self) -> Program {
        let start = self.current_span();

        self.expect(&Token::Program);
        let (name, name_span) = self
            .expect_ident()
            .unwrap_or_else(|| ("_error_".to_string(), self.current_span()));
        self.expect_semi();

        let uses = if self.check(&Token::Uses) {
            self.parse_uses_clause()
        } else {
            Vec::new()
        };

        let declarations = self.parse_declarations(false);

        self.expect(&Token::Begin);
        let body = self.parse_statement_list();
        self.expect(&Token::End);
        self.expect(&Token::Dot);

        let span = self.span_from(start);
        Program {
            name,
            name_span,
            uses,
            declarations,
            body,
            span,
        }
    }

    fn parse_unit_ast(&mut self) -> Unit {
        let start = self.current_span();

        self.expect(&Token::Unit);
        let name = self.parse_qualified_id();
        self.expect_semi();

        let uses = if self.check(&Token::Uses) {
            self.parse_uses_clause()
        } else {
            Vec::new()
        };

        let declarations = self.parse_declarations(true);

        if !self.check(&Token::Eof) {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                &format!(
                    "Expected end of file after unit declarations, found `{}`",
                    super::token_display(self.current_token())
                ),
                "Unit files contain declarations only. Remove trailing statements or blocks.",
                span,
            );
            while !self.at_end() {
                self.advance();
            }
        }

        let span = self.span_from(start);
        Unit {
            name,
            uses,
            declarations,
            span,
        }
    }

    fn parse_uses_clause(&mut self) -> Vec<QualifiedId> {
        self.advance();
        let mut units = Vec::new();
        units.push(self.parse_qualified_id());
        while self.eat(&Token::Comma) {
            units.push(self.parse_qualified_id());
        }
        self.expect_semi();
        units
    }

    /// Parse a dotted identifier path while preserving strong sync tokens on recovery.
    pub(crate) fn parse_qualified_id(&mut self) -> QualifiedId {
        let start = self.current_span();
        let mut parts = Vec::new();
        match self.expect_ident() {
            Some((name, _)) => parts.push(name),
            None => {
                if !self.at_end() && !self.is_qualified_id_recovery_boundary() {
                    self.advance();
                }
                parts.push("_error_".to_string());
            }
        }
        while self.eat(&Token::Dot) {
            if let Some((name, _)) = self.expect_ident_after_dot() {
                parts.push(name);
            } else {
                if !self.at_end() && !self.is_qualified_id_recovery_boundary() {
                    self.advance();
                }
                parts.push("_error_".to_string());
                break;
            }
        }
        QualifiedId {
            parts,
            span: self.span_from(start),
        }
    }

    fn is_qualified_id_recovery_boundary(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Dot
                | Token::Comma
                | Token::Semicolon
                | Token::Colon
                | Token::Equal
                | Token::RParen
                | Token::RBracket
                | Token::Then
                | Token::Else
                | Token::Of
                | Token::To
                | Token::Downto
                | Token::In
                | Token::Do
                | Token::Until
                | Token::Begin
                | Token::End
                | Token::Program
                | Token::Unit
                | Token::Uses
                | Token::Const
                | Token::Var
                | Token::Mutable
                | Token::Function
                | Token::Procedure
                | Token::If
                | Token::Case
                | Token::For
                | Token::While
                | Token::Repeat
                | Token::Return
                | Token::Panic
                | Token::Break
                | Token::Continue
                | Token::Type
                | Token::Public
                | Token::Private
                | Token::Go
                | Token::Eof
        )
    }
}
