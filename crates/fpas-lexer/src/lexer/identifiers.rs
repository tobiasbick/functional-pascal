use super::Lexer;
use crate::Token;

impl Lexer<'_> {
    pub(super) fn scan_ident_or_keyword(&mut self) {
        let (so, sl, sc) = self.span_here();
        let mut raw = String::new();

        while !self.at_end() && (self.current().is_ascii_alphanumeric() || self.current() == b'_') {
            raw.push(self.advance() as char);
        }

        let token = Token::from_ident(&raw);
        self.push_tok(token, so, sl, sc);
    }
}
