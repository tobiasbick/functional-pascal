use super::Lexer;
use crate::Token;

impl Lexer<'_> {
    pub(super) fn scan_colon(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        if !self.at_end() && self.current() == b'=' {
            self.advance();
            self.push_tok(Token::ColonAssign, so, sl, sc);
        } else {
            self.push_tok(Token::Colon, so, sl, sc);
        }
    }

    pub(super) fn scan_dot(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        if !self.at_end() && self.current() == b'.' {
            self.advance();
            self.push_tok(Token::DotDot, so, sl, sc);
        } else {
            self.push_tok(Token::Dot, so, sl, sc);
        }
    }

    pub(super) fn scan_less(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();

        if self.at_end() {
            self.push_tok(Token::Less, so, sl, sc);
            return;
        }

        match self.current() {
            b'>' => {
                self.advance();
                self.push_tok(Token::NotEqual, so, sl, sc);
            }
            b'=' => {
                self.advance();
                self.push_tok(Token::LessEqual, so, sl, sc);
            }
            _ => self.push_tok(Token::Less, so, sl, sc),
        }
    }

    pub(super) fn scan_greater(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        if !self.at_end() && self.current() == b'=' {
            self.advance();
            self.push_tok(Token::GreaterEqual, so, sl, sc);
        } else {
            self.push_tok(Token::Greater, so, sl, sc);
        }
    }
}
