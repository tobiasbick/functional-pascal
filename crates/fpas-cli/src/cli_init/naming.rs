//! Deterministic conversion from scaffold names to FPAS identifiers.

/// Converts a validated kebab- or snake-case scaffold name to Pascal case.
pub(crate) fn pascal_identifier(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut uppercase_next = true;
    for character in name.chars() {
        if matches!(character, '-' | '_') {
            uppercase_next = true;
        } else if uppercase_next {
            result.push(character.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(character);
        }
    }
    result
}

/// Returns whether one name segment is a valid non-keyword FPAS identifier.
pub(crate) fn is_available_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(first) if first.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && matches!(
            fpas_lexer::Token::from_ident(value),
            fpas_lexer::Token::Ident(_)
        )
}

#[cfg(test)]
mod tests {
    use super::{is_available_identifier, pascal_identifier};

    #[test]
    fn converts_supported_scaffold_names() {
        assert_eq!(pascal_identifier("hello"), "Hello");
        assert_eq!(pascal_identifier("my-app"), "MyApp");
        assert_eq!(pascal_identifier("my_library"), "MyLibrary");
    }

    #[test]
    fn rejects_keywords_as_generated_identifiers() {
        assert!(is_available_identifier("MyProgram"));
        assert!(!is_available_identifier("program"));
        assert!(!is_available_identifier("Demo-Greet"));
    }
}
