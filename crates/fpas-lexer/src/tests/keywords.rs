use super::toks;
use crate::Token;

#[test]
fn reserved_keywords_and_ordinary_mutable_identifier() {
    let input = "program unit uses const var mutable function procedure begin end return discard \
                 if then else elsif case when of for to downto in in do while \
                 repeat until and or not xor div mod \
                 true false type record enum array channel task panic break continue \
                 public result option ok error some none try \
                 go dict with static property event read write comparable numeric printable self nil null";
    let tokens = toks(input);
    assert_eq!(
        tokens,
        vec![
            Token::Program,
            Token::Unit,
            Token::Uses,
            Token::Const,
            Token::Var,
            Token::Ident("mutable".into()),
            Token::Function,
            Token::Procedure,
            Token::Begin,
            Token::End,
            Token::Return,
            Token::Discard,
            Token::If,
            Token::Then,
            Token::Else,
            Token::Elsif,
            Token::Case,
            Token::When,
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
            Token::True,
            Token::False,
            Token::Type,
            Token::Record,
            Token::Enum,
            Token::Array,
            Token::Channel,
            Token::Task,
            Token::Panic,
            Token::Break,
            Token::Continue,
            Token::Public,
            Token::Result,
            Token::OptionKw,
            Token::Ok,
            Token::Error,
            Token::Some,
            Token::None,
            Token::Try,
            Token::Go,
            Token::Dict,
            Token::With,
            Token::Static,
            Token::Property,
            Token::Event,
            Token::Read,
            Token::Write,
            Token::Comparable,
            Token::Numeric,
            Token::Printable,
            Token::SelfKw,
            Token::Nil,
            Token::Null,
        ]
    );
}

#[test]
fn block_keywords_are_reserved_in_every_ascii_letter_case() {
    for (keyword, expected) in [
        ("elsif", Token::Elsif),
        ("when", Token::When),
        ("null", Token::Null),
        ("discard", Token::Discard),
    ] {
        for uppercase_mask in 0..(1 << keyword.len()) {
            let spelling = keyword
                .bytes()
                .enumerate()
                .map(|(index, byte)| {
                    char::from(if uppercase_mask & (1 << index) == 0 {
                        byte
                    } else {
                        byte.to_ascii_uppercase()
                    })
                })
                .collect::<String>();
            assert_eq!(toks(&spelling), vec![expected.clone()], "{spelling}");
        }
    }
}

#[test]
fn block_keyword_prefixes_and_suffixes_are_identifiers() {
    for keyword in ["elsif", "when", "null"] {
        for identifier in [
            format!("{keyword}Value"),
            format!("{keyword}_value"),
            format!("value{keyword}"),
        ] {
            assert_eq!(toks(&identifier), vec![Token::Ident(identifier)]);
        }
    }
}

#[test]
fn block_keywords_inside_strings_and_comments_are_preserved() {
    let source = "'elsif WHEN NuLl' // elsif WHEN NuLl\n'null' // ELSIF when null\nelsif when null";
    assert_eq!(
        toks(source),
        vec![
            Token::Str("elsif WHEN NuLl".into()),
            Token::Str("null".into()),
            Token::Elsif,
            Token::When,
            Token::Null
        ]
    );
}

#[test]
fn block_keyword_spans_preserve_source_casing() {
    let source = "ElSiF WhEn NuLl";
    let (tokens, errors) = crate::lex(source);
    assert!(errors.is_empty(), "{errors:?}");
    for (token, spelling) in tokens.iter().zip(["ElSiF", "WhEn", "NuLl"]) {
        assert_eq!(token.span.text(source), Some(spelling));
    }
}

#[test]
fn private_is_an_identifier() {
    assert_eq!(toks("private"), vec![Token::Ident("private".into())]);
}

#[test]
fn forward_is_not_a_keyword() {
    assert_eq!(toks("forward"), vec![Token::Ident("forward".into())]);
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
    assert_eq!(toks("SeLf"), vec![Token::SelfKw]);
    assert_eq!(toks("CoMpArAbLe"), vec![Token::Comparable]);
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
fn keyword_and_identifier_scanning_preserves_exact_source_spans() {
    let source = "PROGRAM My_Name";
    let (tokens, errors) = crate::lex(source);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(tokens[0].token, Token::Program);
    assert_eq!(tokens[0].span.text(source), Some("PROGRAM"));
    assert_eq!(tokens[1].token, Token::Ident("My_Name".to_string()));
    assert_eq!(tokens[1].span.text(source), Some("My_Name"));
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

#[test]
fn retired_shift_names_are_identifiers_in_every_ascii_letter_case() {
    for name in ["shl", "shr"] {
        for mask in 0..8 {
            let spelling = name
                .bytes()
                .enumerate()
                .map(|(index, byte)| {
                    char::from(if mask & (1 << index) == 0 {
                        byte
                    } else {
                        byte.to_ascii_uppercase()
                    })
                })
                .collect::<String>();
            assert_eq!(
                toks(&spelling),
                vec![Token::Ident(spelling.clone())],
                "{spelling}"
            );
        }
    }
}
