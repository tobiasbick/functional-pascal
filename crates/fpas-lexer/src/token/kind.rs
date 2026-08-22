macro_rules! documented_token_enum {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(($payload:ty))?,
            )*
        }
    ) => {
        $(#[$enum_meta])*
        pub enum $name {
            $(
                $(#[$variant_meta])*
                #[doc = concat!("Lexical token `", stringify!($variant), "`.")]
                $variant $(($payload))?,
            )*
        }
    };
}

documented_token_enum! {
/// Lexical token produced by the Functional Pascal lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords (56)
    Program,
    Unit,
    Uses,
    Const,
    Var,
    Mutable,
    Function,
    Procedure,
    Begin,
    End,
    Return,
    If,
    Then,
    Else,
    Case,
    Of,
    For,
    To,
    Downto,
    In,
    Do,
    While,
    Repeat,
    Until,
    And,
    Or,
    Not,
    Xor,
    Div,
    Mod,
    Shl,
    Shr,
    True,
    False,
    Type,
    Record,
    Enum,
    Array,
    Panic,
    Break,
    Continue,
    Public,
    Result,
    OptionKw,
    Ok,
    Error,
    Some,
    None,
    Try,
    Go,
    Dict,
    With,
    /// Marks a static record routine: `static function Create(...): T` or
    /// `static procedure Reset(...)`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    Static,
    /// Marks a computed record property: `property Text: string read GetText write SetText`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    Property,
    /// Marks a record event: `event OnClick: Handler read Get write Set`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    Event,
    /// Clears an event handler: `Button.OnClick := nil`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    Nil,

    // Literals
    Integer(i64),
    Real(f64),
    Str(String),

    // Identifier
    Ident(String),

    // Symbols
    ColonAssign,
    DotDot,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Colon,
    Semicolon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Less,
    Greater,

    // End of file
    Eof,
}
}
