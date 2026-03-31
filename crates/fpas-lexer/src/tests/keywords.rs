use super::toks;
use crate::Token;

#[test]
fn all_62_keywords() {
    let input = "program unit uses const var mutable function procedure begin end return \
                 if then else case of for to downto in in do while \
                 repeat until and or not xor div mod shl shr \
                 true false type record enum array panic break continue \
                 public private result option ok error some none try \
                 go channel select default from dict ref new with interface implements extends";
    let tokens = toks(input);
    assert_eq!(
        tokens,
        vec![
            Token::Program,
            Token::Unit,
            Token::Uses,
            Token::Const,
            Token::Var,
            Token::Mutable,
            Token::Function,
            Token::Procedure,
            Token::Begin,
            Token::End,
            Token::Return,
            Token::If,
            Token::Then,
            Token::Else,
            Token::Case,
            Token::Of,
            Token::For,
            Token::To,
            Token::Downto,
            Token::In,
            Token::In,
            Token::Do,
            Token::While,
            Token::Repeat,
            Token::Until,
            Token::And,
            Token::Or,
            Token::Not,
            Token::Xor,
            Token::Div,
            Token::Mod,
            Token::Shl,
            Token::Shr,
            Token::True,
            Token::False,
            Token::Type,
            Token::Record,
            Token::Enum,
            Token::Array,
            Token::Panic,
            Token::Break,
            Token::Continue,
            Token::Public,
            Token::Private,
            Token::Result,
            Token::OptionKw,
            Token::Ok,
            Token::Error,
            Token::Some,
            Token::None,
            Token::Try,
            Token::Go,
            Token::Channel,
            Token::Select,
            Token::Default,
            Token::From,
            Token::Dict,
            Token::Ref,
            Token::New,
            Token::With,
            Token::Interface,
            Token::Implements,
            Token::Extends,
        ]
    );
}

#[test]
fn forward_is_not_a_keyword() {
    assert_eq!(toks("forward"), vec![Token::Ident("forward".into())]);
}

#[test]
fn default_is_reserved_keyword() {
    assert_eq!(toks("default"), vec![Token::Default]);
    assert_eq!(toks("DEFAULT"), vec![Token::Default]);
    assert_eq!(
        toks("defaultValue"),
        vec![Token::Ident("defaultValue".into())]
    );
}

#[test]
fn case_insensitive_lowercase() {
    assert_eq!(toks("program"), vec![Token::Program]);
    assert_eq!(toks("begin"), vec![Token::Begin]);
    assert_eq!(toks("true"), vec![Token::True]);
}

#[test]
fn case_insensitive_uppercase() {
    assert_eq!(toks("PROGRAM"), vec![Token::Program]);
    assert_eq!(toks("BEGIN"), vec![Token::Begin]);
    assert_eq!(toks("TRUE"), vec![Token::True]);
}

#[test]
fn case_insensitive_mixed() {
    assert_eq!(toks("Program"), vec![Token::Program]);
    assert_eq!(toks("pRoGrAm"), vec![Token::Program]);
    assert_eq!(toks("BeGiN"), vec![Token::Begin]);
    assert_eq!(toks("tRuE"), vec![Token::True]);
    assert_eq!(toks("FaLsE"), vec![Token::False]);
    assert_eq!(toks("ReTuRn"), vec![Token::Return]);
}

#[test]
fn keyword_prefix_is_identifier() {
    assert_eq!(toks("programs"), vec![Token::Ident("programs".into())]);
    assert_eq!(toks("iff"), vec![Token::Ident("iff".into())]);
    assert_eq!(toks("returns"), vec![Token::Ident("returns".into())]);
    assert_eq!(toks("truefalse"), vec![Token::Ident("truefalse".into())]);
    assert_eq!(toks("begins"), vec![Token::Ident("begins".into())]);
    assert_eq!(toks("ended"), vec![Token::Ident("ended".into())]);
}

#[test]
fn keyword_as_part_of_longer_word() {
    assert_eq!(toks("myfunction"), vec![Token::Ident("myfunction".into())]);
    assert_eq!(toks("endgame"), vec![Token::Ident("endgame".into())]);
    assert_eq!(toks("fortune"), vec![Token::Ident("fortune".into())]);
    assert_eq!(toks("divider"), vec![Token::Ident("divider".into())]);
}

#[test]
fn keywords_surrounded_by_symbols() {
    assert_eq!(
        toks("(begin)"),
        vec![Token::LParen, Token::Begin, Token::RParen]
    );
    assert_eq!(toks("not="), vec![Token::Not, Token::Equal]);
}
