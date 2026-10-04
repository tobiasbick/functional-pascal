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
    // Keywords
    Program,
    Unit,
    Uses,
    As,
    Const,
    Var,
    Pure,
    Function,
    Procedure,
    Begin,
    End,
    Return,
    Discard,
    If,
    Then,
    Else,
    Elsif,
    When,
    Null,
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
    True,
    False,
    Type,
    Record,
    Enum,
    Array,
    /// Introduces a typed bounded channel: `channel of T`.
    Channel,
    /// A task handle type: bare `task` infers its result type, `task of T` declares it.
    Task,
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
    /// Restricts a generic type parameter to comparable types.
    Comparable,
    /// Equality-only generic constraint.
    Equatable,
    /// Restricts a generic type parameter to numeric types.
    Numeric,
    /// Restricts a generic type parameter to printable types.
    Printable,
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
