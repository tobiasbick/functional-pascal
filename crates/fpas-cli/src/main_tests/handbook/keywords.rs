//! The EBNF and handbook reserved-word tables must agree with lexer tokens.

use super::document;
use fpas_lexer::Token;
use std::collections::BTreeSet;

#[test]
fn grammar_and_handbook_keyword_tables_agree_with_the_lexer() {
    let grammar = document("docs/specs/grammar.ebnf");
    let production = grammar
        .split_once("keyword         =")
        .expect("keyword production")
        .1
        .split_once(';')
        .expect("keyword production terminator")
        .0;
    let keywords = production
        .split('\'')
        .skip(1)
        .step_by(2)
        .collect::<Vec<_>>();
    let grammar_words = keywords.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        keywords.len(),
        grammar_words.len(),
        "duplicate grammar keyword"
    );

    let handbook = document("docs/pascal/getting-started/keywords.md").replace("\r\n", "\n");
    let table = handbook.split_once("```\n").expect("keyword table").1;
    let table = table.split_once("```").expect("keyword table closer").0;
    let words = table.split_whitespace().collect::<Vec<_>>();
    let handbook_words = words.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        words.len(),
        handbook_words.len(),
        "duplicate handbook keyword"
    );
    assert_eq!(grammar_words, handbook_words);

    for keyword in grammar_words {
        let token = Token::from_ident(keyword);
        assert!(
            !matches!(token, Token::Ident(_)),
            "{keyword}: not a lexer keyword"
        );
        assert_eq!(
            Token::from_ident(&keyword.to_uppercase()),
            token,
            "{keyword}"
        );
    }
}

#[test]
fn retired_words_and_keyword_prefixes_remain_valid_identifiers() {
    for name in [
        "private",
        "shl",
        "shr",
        "asValue",
        "elsifValue",
        "whenValue",
        "NullValue",
    ] {
        assert_eq!(Token::from_ident(name), Token::Ident(name.to_owned()));
    }
}
