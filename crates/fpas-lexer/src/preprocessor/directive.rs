//! Parsing of raw compiler directive content into typed [`DirectiveKind`] variants.
//!
//! **Documentation:** `docs/pascal/12-compiler-directives.md`

/// Parsed kind of a `{$...}` compiler directive.
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveKind {
    /// `{$DEFINE name}` — introduce a conditional symbol.
    Define(String),
    /// `{$UNDEF name}` — remove a previously defined symbol.
    Undef(String),
    /// `{$IFDEF name}` — begin a block that is active if `name` is defined.
    IfDef(String),
    /// `{$IFNDEF name}` — begin a block that is active if `name` is **not** defined.
    IfNDef(String),
    /// `{$ELSE}` — toggle the current conditional block.
    Else,
    /// `{$ENDIF}` — close the current conditional block.
    EndIf,
    /// `{$I filename}` or `{$INCLUDE filename}` — include a source file.
    Include(String),
    /// Any directive name not recognised by the preprocessor.
    ///
    /// Stored as the full trimmed content string so diagnostics can display it.
    Unknown(String),
}

/// Parses the raw trimmed content of a `{$...}` directive (the part after `$`).
///
/// The comparison is case-insensitive.  Unknown names produce
/// [`DirectiveKind::Unknown`] rather than an error so that the caller can
/// decide on the diagnostic severity.
#[must_use]
pub fn parse_directive_content(raw: &str) -> DirectiveKind {
    // Split on the first run of whitespace: keyword + optional argument.
    let (keyword, arg) = match raw.find(|c: char| c.is_ascii_whitespace()) {
        Some(i) => (&raw[..i], raw[i..].trim()),
        None => (raw, ""),
    };

    match keyword.to_ascii_uppercase().as_str() {
        "DEFINE" => {
            let name = first_token(arg);
            DirectiveKind::Define(name.to_owned())
        }
        "UNDEF" | "UNDEFINE" => {
            let name = first_token(arg);
            DirectiveKind::Undef(name.to_owned())
        }
        "IFDEF" => {
            let name = first_token(arg);
            DirectiveKind::IfDef(name.to_uppercase())
        }
        "IFNDEF" => {
            let name = first_token(arg);
            DirectiveKind::IfNDef(name.to_uppercase())
        }
        "ELSE" => DirectiveKind::Else,
        "ENDIF" => DirectiveKind::EndIf,
        // Both short form `{$I file}` and long form `{$INCLUDE file}`.
        "I" | "INCLUDE" => {
            let filename = arg.to_owned();
            DirectiveKind::Include(filename)
        }
        // Compiler switches like `{$R+}`, `{$O+}` fall here.
        _ => DirectiveKind::Unknown(raw.to_owned()),
    }
}

/// Returns the first whitespace-delimited token from `s`, or `""` if `s` is empty.
fn first_token(s: &str) -> &str {
    s.split_ascii_whitespace().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_define() {
        assert_eq!(
            parse_directive_content("DEFINE DEBUG"),
            DirectiveKind::Define("DEBUG".into())
        );
    }

    #[test]
    fn parse_define_case_insensitive() {
        assert_eq!(
            parse_directive_content("define Release"),
            DirectiveKind::Define("Release".into())
        );
    }

    #[test]
    fn parse_undef() {
        assert_eq!(
            parse_directive_content("UNDEF FOO"),
            DirectiveKind::Undef("FOO".into())
        );
    }

    #[test]
    fn parse_ifdef() {
        assert_eq!(
            parse_directive_content("IFDEF DEBUG"),
            DirectiveKind::IfDef("DEBUG".into())
        );
    }

    #[test]
    fn parse_ifdef_lowercase() {
        assert_eq!(
            parse_directive_content("ifdef debug"),
            DirectiveKind::IfDef("DEBUG".into())
        );
    }

    #[test]
    fn parse_ifndef() {
        assert_eq!(
            parse_directive_content("IFNDEF RELEASE"),
            DirectiveKind::IfNDef("RELEASE".into())
        );
    }

    #[test]
    fn parse_else() {
        assert_eq!(parse_directive_content("ELSE"), DirectiveKind::Else);
    }

    #[test]
    fn parse_endif() {
        assert_eq!(parse_directive_content("ENDIF"), DirectiveKind::EndIf);
    }

    #[test]
    fn parse_include_short() {
        assert_eq!(
            parse_directive_content("I config.fpas"),
            DirectiveKind::Include("config.fpas".into())
        );
    }

    #[test]
    fn parse_include_long() {
        assert_eq!(
            parse_directive_content("INCLUDE helpers.fpas"),
            DirectiveKind::Include("helpers.fpas".into())
        );
    }

    #[test]
    fn parse_unknown_switch() {
        assert_eq!(
            parse_directive_content("R+"),
            DirectiveKind::Unknown("R+".into())
        );
    }
}
