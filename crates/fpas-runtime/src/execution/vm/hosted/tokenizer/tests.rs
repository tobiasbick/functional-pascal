//! External reference vectors and native failure boundaries.

use super::*;

#[derive(serde::Deserialize)]
struct Reference {
    encoding: String,
    cases: Vec<Vector>,
}
#[derive(serde::Deserialize)]
struct Vector {
    text: String,
    tokens: Vec<Rank>,
}

#[test]
fn ordinary_token_ids_match_the_python_reference_including_unicode_and_literal_markers() {
    let reference: Reference = serde_json::from_str(include_str!("reference.json")).unwrap();
    let cache = Cache::default();
    let bpe = cache.get(encoding(&reference.encoding).unwrap()).unwrap();
    for case in reference.cases {
        assert_eq!(
            bpe.encode_ordinary(&case.text),
            case.tokens,
            "{:?}",
            case.text
        );
        assert_eq!(bpe.count_ordinary(&case.text), case.tokens.len());
        assert_eq!(decode(bpe, &case.tokens).unwrap(), case.text);
    }
}

#[test]
fn supported_encodings_roundtrip_and_keep_their_token_spaces_separate() {
    let cache = Cache::default();
    let text = "FPAS 🙂 中文 1234\r\n";
    for index in 0..ENCODINGS.len() {
        let bpe = cache.get(index).unwrap();
        assert_eq!(decode(bpe, &bpe.encode_ordinary(text)).unwrap(), text);
    }
    assert_ne!(
        cache.get(0).unwrap().encode_ordinary("hello world"),
        cache.get(1).unwrap().encode_ordinary("hello world")
    );
    assert!(matches!(encoding("unknown"), Err(Failure::Encoding(_))));
}

#[test]
fn invalid_tokens_utf8_and_native_limits_are_typed_and_do_not_drop_later_bytes() {
    let cache = Cache::default();
    let bpe = cache.get(0).unwrap();
    assert_eq!(token_ids(&[Value::Integer(-1)]), Err(Failure::Token(0, -1)));
    assert_eq!(
        token_ids(&[Value::Integer(i64::from(u32::MAX) + 1)]),
        Err(Failure::Token(0, i64::from(u32::MAX) + 1))
    );
    assert_eq!(
        decode(bpe, &[u32::MAX]),
        Err(Failure::Token(0, i64::from(u32::MAX)))
    );
    assert_eq!(decode(bpe, &[94]), Err(Failure::Utf8));
    assert_eq!(
        token_ids(&vec![Value::Integer(0); MAX_TOKENS + 1]),
        Err(Failure::Limit("Tokens", MAX_TOKENS))
    );
    assert_eq!(
        text_limit(&"x".repeat(MAX_BYTES + 1)),
        Err(Failure::Limit("TextBytes", MAX_BYTES))
    );
    let spaces = bpe.encode_ordinary(&" ".repeat(128));
    assert_eq!(spaces.len(), 1);
    let mut tokens = vec![spaces[0]; MAX_BYTES / 128];
    assert_eq!(decode(bpe, &tokens).unwrap().len(), MAX_BYTES);
    tokens.push(spaces[0]);
    assert_eq!(
        decode(bpe, &tokens),
        Err(Failure::Limit("DecodedBytes", MAX_BYTES))
    );
}

#[test]
fn tokenizer_callbacks_and_typed_errors_run_in_vm_and_seeded_debugger() {
    use crate::execution::vm::{
        Vm,
        debug::{DebugRunResult, DebugSession},
    };
    for source in [
        include_str!("../../../../../../../tests/stdlib/tokenizer/roundtrip_test.fpas"),
        include_str!("../../../../../../../tests/stdlib/tokenizer/errors_test.fpas"),
        include_str!("../../../../../../../tests/stdlib/tokenizer/tasks_test.fpas"),
    ] {
        let (program, errors) = fpas_parser::parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let image = fpas_compiler::compile(&program).unwrap();
        Vm::new(image.clone()).run().unwrap();
        for seed in [None, Some(17)] {
            let mut debug = DebugSession::with_limits_and_schedule(
                image.clone(),
                vec![],
                Default::default(),
                Default::default(),
                seed,
            )
            .unwrap();
            assert!(matches!(
                debug.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ));
        }
    }
}
