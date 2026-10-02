//! Structural AST comparison independent of source coordinates.

use fpas_parser::CompilationUnit;

/// Removes lexer span records from a debug representation while preserving quoted strings.
pub fn normalized_ast(unit: &CompilationUnit) -> String {
    let debug = format!("{unit:?}");
    let mut output = String::new();
    let mut position = 0;
    let mut quoted = false;
    let bytes = debug.as_bytes();
    while position < bytes.len() {
        if quoted && bytes[position] == b'\\' {
            output.push_str(&debug[position..position + 2]);
            position += 2;
            continue;
        }
        if bytes[position] == b'"' {
            quoted = !quoted;
        }
        if !quoted && debug[position..].starts_with("Span {") {
            position += debug[position..]
                .find('}')
                .expect("span debug record closes")
                + 1;
            output.push_str("Span {}");
        } else {
            let character = debug[position..].chars().next().expect("character exists");
            output.push(character);
            position += character.len_utf8();
        }
    }
    output
}
