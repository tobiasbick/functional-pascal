//! Canonical Result parser repair forms using source-local import aliases.
//! See `docs/pascal/std/text/parse.md`.

use super::Checker;

impl Checker {
    /// Suggests the selected parser only for an unresolved retired parser spelling.
    pub(super) fn parser_migration_hint(&self, name: &str) -> Option<String> {
        let canonical = self.scopes.imports.canonical_name(name);
        let (owner, member) = canonical.rsplit_once('.').unwrap_or(("", &canonical));
        let (replacement, abort) = match (owner, member) {
            ("std.text.parse" | "", "tryint") => ("Int", false),
            ("std.text.parse" | "", "tryreal") => ("Real", false),
            ("std.text.parse" | "", "trybool") => ("Bool", false),
            ("std.text.format" | "", "strtoint") => ("Int", true),
            ("std.text.format" | "", "strtoreal") => ("Real", true),
            ("std.text.format" | "", "strtobool") => ("Bool", true),
            ("std.text.format" | "", "hextoint") => ("HexInt", true),
            _ => return None,
        };
        let qualified = format!("Std.Text.Parse.{replacement}");
        let (import, call) = if let Some(source) = self
            .scopes
            .imports
            .source_name_for("Std.Text.Parse", &qualified)
        {
            (String::new(), source)
        } else {
            let mut alias = "Parse".to_owned();
            let mut suffix = 2;
            while self.scopes.lookup(&alias).is_some() || self.scopes.imports.is_alias(&alias) {
                alias = format!("Parse{suffix}");
                suffix += 1;
            }
            (
                format!("Import `uses Std.Text.Parse as {alias};` and "),
                format!("{alias}.{replacement}"),
            )
        };
        let handling = if abort {
            format!(
                "use `{call}(Text).Unwrap()` to preserve deliberate abort, or handle the Result with case or try"
            )
        } else {
            format!("use `{call}(Text)` and handle its Result with case, try, or explicit Unwrap")
        };
        Some(format!("{import}{handling}. The parameter name is Text."))
    }
}
