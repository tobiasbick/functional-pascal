//! Deterministic source names for protocol strings and JSON properties.

pub(super) fn identifier(wire: &str) -> String {
    let mut output = String::new();
    let mut upper = true;
    for character in wire.chars() {
        if !character.is_ascii_alphanumeric() {
            upper = true;
            continue;
        }
        output.push(if upper {
            character.to_ascii_uppercase()
        } else {
            character
        });
        upper = false;
    }
    if output.is_empty() {
        output.push_str("Value");
    }
    if output.as_bytes()[0].is_ascii_digit() {
        output.insert_str(0, "Value");
    }
    if [
        "and",
        "array",
        "as",
        "begin",
        "boolean",
        "case",
        "const",
        "discard",
        "distinct",
        "div",
        "do",
        "downto",
        "else",
        "elsif",
        "end",
        "enum",
        "error",
        "except",
        "exit",
        "export",
        "false",
        "finally",
        "for",
        "function",
        "go",
        "if",
        "implementation",
        "in",
        "integer",
        "interface",
        "is",
        "mod",
        "nil",
        "not",
        "null",
        "of",
        "or",
        "out",
        "private",
        "procedure",
        "program",
        "public",
        "raise",
        "real",
        "record",
        "repeat",
        "result",
        "return",
        "scope",
        "string",
        "then",
        "to",
        "true",
        "try",
        "type",
        "unit",
        "until",
        "uses",
        "var",
        "when",
        "while",
        "with",
        "xor",
    ]
    .contains(&output.to_ascii_lowercase().as_str())
    {
        output.push_str("Value");
    }
    output
}

pub(super) fn patch(version: crate::schema::Version, owner: &str, field: &str) -> bool {
    use crate::schema::Version;
    match owner {
        "SessionMessage" => matches!(field, "content" | "_meta"),
        "CompactionUpdate" => matches!(field, "summary" | "error" | "_meta"),
        "SubagentUpdate" => matches!(
            field,
            "title" | "description" | "capabilities" | "state" | "_meta"
        ),
        "SessionInfoUpdate" => {
            matches!(field, "title" | "updatedAt") || (version == Version::V2 && field == "_meta")
        }
        "McpError" => field == "data",
        "UserMessage" | "AgentMessage" | "AgentThought" if version == Version::V2 => {
            matches!(field, "content" | "_meta")
        }
        "TerminalUpdate" if version == Version::V2 => {
            matches!(field, "command" | "cwd" | "output" | "exitStatus" | "_meta")
        }
        "ToolCallUpdate" if version == Version::V2 => field != "toolCallId",
        _ => false,
    }
}

// These SDK v1 untagged fallbacks omit their semantic default tag in JSON Schema.
pub(super) fn default_variant(
    version: crate::schema::Version,
    owner: &str,
    index: usize,
) -> Option<(&'static str, &'static str)> {
    if version != crate::schema::Version::V1 {
        return None;
    }
    match (owner, index) {
        ("SetSessionConfigOptionRequestKind", 1) => Some(("Select", "select")),
        ("AuthMethod", 1) => Some(("Agent", "agent")),
        _ => None,
    }
}
