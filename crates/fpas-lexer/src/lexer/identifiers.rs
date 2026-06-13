use super::Lexer;
use crate::Token;

impl Lexer<'_> {
    pub(super) fn scan_ident_or_keyword(&mut self) {
        let (so, sl, sc) = self.span_here();
        let mut raw = String::new();

        while !self.at_end() {
            let ch = self.current();
            if !ch.is_ascii_alphanumeric() && ch != b'_' {
                break;
            }
            raw.push(self.advance() as char);
        }

        let token = Token::from_ident_owned(raw);
        self.push_tok(token, so, sl, sc);
    }
}
