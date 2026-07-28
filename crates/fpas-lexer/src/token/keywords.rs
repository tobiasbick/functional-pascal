use super::Token;

/// Maps a lowercase-insensitive identifier to a keyword token when recognized.
fn keyword_token(raw: &str) -> Option<Token> {
    match raw {
        s if s.eq_ignore_ascii_case("program") => Some(Token::Program),
        s if s.eq_ignore_ascii_case("unit") => Some(Token::Unit),
        s if s.eq_ignore_ascii_case("uses") => Some(Token::Uses),
        s if s.eq_ignore_ascii_case("const") => Some(Token::Const),
        s if s.eq_ignore_ascii_case("var") => Some(Token::Var),
        s if s.eq_ignore_ascii_case("mutable") => Some(Token::Mutable),
        s if s.eq_ignore_ascii_case("function") => Some(Token::Function),
        s if s.eq_ignore_ascii_case("procedure") => Some(Token::Procedure),
        s if s.eq_ignore_ascii_case("begin") => Some(Token::Begin),
        s if s.eq_ignore_ascii_case("end") => Some(Token::End),
        s if s.eq_ignore_ascii_case("return") => Some(Token::Return),
        s if s.eq_ignore_ascii_case("if") => Some(Token::If),
        s if s.eq_ignore_ascii_case("then") => Some(Token::Then),
        s if s.eq_ignore_ascii_case("else") => Some(Token::Else),
        s if s.eq_ignore_ascii_case("case") => Some(Token::Case),
        s if s.eq_ignore_ascii_case("of") => Some(Token::Of),
        s if s.eq_ignore_ascii_case("for") => Some(Token::For),
        s if s.eq_ignore_ascii_case("to") => Some(Token::To),
        s if s.eq_ignore_ascii_case("downto") => Some(Token::Downto),
        s if s.eq_ignore_ascii_case("in") => Some(Token::In),
        s if s.eq_ignore_ascii_case("do") => Some(Token::Do),
        s if s.eq_ignore_ascii_case("while") => Some(Token::While),
        s if s.eq_ignore_ascii_case("repeat") => Some(Token::Repeat),
        s if s.eq_ignore_ascii_case("until") => Some(Token::Until),
        s if s.eq_ignore_ascii_case("and") => Some(Token::And),
        s if s.eq_ignore_ascii_case("or") => Some(Token::Or),
        s if s.eq_ignore_ascii_case("not") => Some(Token::Not),
        s if s.eq_ignore_ascii_case("xor") => Some(Token::Xor),
        s if s.eq_ignore_ascii_case("div") => Some(Token::Div),
        s if s.eq_ignore_ascii_case("mod") => Some(Token::Mod),
        s if s.eq_ignore_ascii_case("shl") => Some(Token::Shl),
        s if s.eq_ignore_ascii_case("shr") => Some(Token::Shr),
        s if s.eq_ignore_ascii_case("true") => Some(Token::True),
        s if s.eq_ignore_ascii_case("false") => Some(Token::False),
        s if s.eq_ignore_ascii_case("type") => Some(Token::Type),
        s if s.eq_ignore_ascii_case("record") => Some(Token::Record),
        s if s.eq_ignore_ascii_case("enum") => Some(Token::Enum),
        s if s.eq_ignore_ascii_case("array") => Some(Token::Array),
        s if s.eq_ignore_ascii_case("panic") => Some(Token::Panic),
        s if s.eq_ignore_ascii_case("break") => Some(Token::Break),
        s if s.eq_ignore_ascii_case("continue") => Some(Token::Continue),
        s if s.eq_ignore_ascii_case("public") => Some(Token::Public),
        s if s.eq_ignore_ascii_case("result") => Some(Token::Result),
        s if s.eq_ignore_ascii_case("option") => Some(Token::OptionKw),
        s if s.eq_ignore_ascii_case("ok") => Some(Token::Ok),
        s if s.eq_ignore_ascii_case("error") => Some(Token::Error),
        s if s.eq_ignore_ascii_case("some") => Some(Token::Some),
        s if s.eq_ignore_ascii_case("none") => Some(Token::None),
        s if s.eq_ignore_ascii_case("try") => Some(Token::Try),
        s if s.eq_ignore_ascii_case("go") => Some(Token::Go),
        s if s.eq_ignore_ascii_case("dict") => Some(Token::Dict),
        s if s.eq_ignore_ascii_case("with") => Some(Token::With),
        s if s.eq_ignore_ascii_case("static") => Some(Token::Static),
        s if s.eq_ignore_ascii_case("property") => Some(Token::Property),
        s if s.eq_ignore_ascii_case("event") => Some(Token::Event),
        s if s.eq_ignore_ascii_case("nil") => Some(Token::Nil),
        _ => None,
    }
}

impl Token {
    /// Creates a keyword token when `raw` matches a Pascal keyword.
    ///
    /// Returns [`Token::Ident`] for non-keyword identifiers.
    #[must_use]
    pub fn from_ident(raw: &str) -> Token {
        keyword_token(raw).unwrap_or_else(|| Token::Ident(raw.to_owned()))
    }

    /// Like [`from_ident`](Self::from_ident), but takes ownership of the scanned identifier text.
    #[must_use]
    pub fn from_ident_owned(raw: String) -> Token {
        keyword_token(&raw).unwrap_or(Token::Ident(raw))
    }
}
