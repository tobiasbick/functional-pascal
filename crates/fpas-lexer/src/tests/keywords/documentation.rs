//! Keyword parity with `docs/specs/grammar.ebnf` and `docs/pascal/getting-started/keywords.md`.

use super::toks;
use crate::Token;
use std::collections::BTreeSet;

fn keyword_set<'a>(words: &[&'a str], source: &str) -> BTreeSet<&'a str> {
    assert!(
        !words.is_empty(),
        "{source}: keyword inventory must not be empty"
    );
    let set = words.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(set.len(), words.len(), "{source}: duplicate keywords");
    set
}

fn lexer_keywords() -> BTreeSet<&'static str> {
    // Read the existing mapping rather than maintain another keyword list in tests.
    let words = include_str!("../../token/keywords.rs")
        .split(".eq_ignore_ascii_case(\"")
        .skip(1)
        .map(|rest| rest.split_once('"').expect("keyword spelling").0)
        .collect::<Vec<_>>();
    keyword_set(&words, "lexer mapping")
}

fn grammar_keywords() -> BTreeSet<&'static str> {
    let alternatives = include_str!("../../../../../docs/specs/grammar.ebnf")
        .split(';')
        .find_map(|production| {
            let (name, body) = production.rsplit_once('=')?;
            (name.lines().last()?.trim() == "keyword").then_some(body)
        })
        .expect("keyword production");
    let words = alternatives
        .split('\'')
        .skip(1)
        .step_by(2)
        .collect::<Vec<_>>();
    keyword_set(&words, "formal keyword production")
}

fn handbook_keywords() -> BTreeSet<&'static str> {
    let table = include_str!("../../../../../docs/pascal/getting-started/keywords.md")
        .split("```")
        .nth(1)
        .expect("handbook keyword table");
    let words = table.split_whitespace().collect::<Vec<_>>();
    keyword_set(&words, "handbook keyword table")
}

fn assert_keyword_parity(actual: &BTreeSet<&str>, source: &str) {
    let expected = lexer_keywords();
    let missing = expected.difference(actual).copied().collect::<Vec<_>>();
    let unexpected = actual.difference(&expected).copied().collect::<Vec<_>>();
    assert!(
        missing.is_empty() && unexpected.is_empty(),
        "{source}: missing reserved keywords {missing:?}; unexpected keywords {unexpected:?}"
    );
}

#[test]
fn formal_keyword_production_matches_the_lexer_inventory() {
    assert_keyword_parity(&grammar_keywords(), "grammar.ebnf");
}

#[test]
fn handbook_keyword_table_matches_the_lexer_inventory() {
    assert_keyword_parity(&handbook_keywords(), "getting-started/keywords.md");
}

#[test]
fn documented_keywords_are_reserved_in_lower_upper_and_mixed_case() {
    for keyword in lexer_keywords() {
        let expected = Token::from_ident(keyword);
        assert!(
            !matches!(expected, Token::Ident(_)),
            "{keyword}: reserved token"
        );
        let mixed_case = keyword
            .bytes()
            .enumerate()
            .map(|(index, byte)| {
                char::from(if index % 2 == 0 {
                    byte.to_ascii_uppercase()
                } else {
                    byte
                })
            })
            .collect::<String>();
        for spelling in [keyword.to_owned(), keyword.to_ascii_uppercase(), mixed_case] {
            assert_eq!(toks(&spelling), vec![expected.clone()], "{spelling}");
        }
    }
}

#[test]
fn discard_boundaries_qualified_names_strings_and_comments_keep_their_token_kinds() {
    for identifier in ["DiscardValue", "discard_value", "valueDiscard", "discard2"] {
        assert_eq!(toks(identifier), vec![Token::Ident(identifier.to_owned())]);
    }
    assert_eq!(
        toks("Module.DiScArD 'discard' // DISCARD\ndiscard"),
        vec![
            Token::Ident("Module".to_owned()),
            Token::Dot,
            Token::Discard,
            Token::Str("discard".to_owned()),
            Token::Discard,
        ]
    );
}
