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
    Const,
    Var,
    Function,
    Procedure,
    Begin,
    End,
    Return,
    /// Explicitly ignores a value after evaluating it once.
    Discard,
    If,
    Then,
    Else,
    /// Reserved keyword `elsif`; unavailable as an identifier.
    Elsif,
    Case,
    /// Reserved keyword `when`; unavailable as an identifier.
    When,
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
    /// Marks a static record routine: `static function Create(...): T` or
    /// `static procedure Reset(...)`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    Static,
    /// Marks a record event: `event OnClick: Handler read Get write Set`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    Event,
    /// Introduces an event's read accessor.
    Read,
    /// Introduces an event's write accessor.
    Write,
    /// Restricts a generic type parameter to comparable types.
    Comparable,
    /// Restricts a generic type parameter to numeric types.
    Numeric,
    /// Restricts a generic type parameter to printable types.
    Printable,
    /// Names the receiver parameter and expression inside an instance record method.
    SelfKw,
    /// Reserved keyword `null`; unavailable as an identifier.
    Null,
    /// Pattern test `Value is Pattern` in `if`, `elsif`, and `while` conditions.
    ///
    /// **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`
    Is,
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
