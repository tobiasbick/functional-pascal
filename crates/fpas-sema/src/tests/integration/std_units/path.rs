//! Public lexical path helpers retain parameter labels and optional outcomes.

use super::*;

#[test]
fn lexical_path_helpers_have_boolean_and_optional_string_signatures() {
    check_ok(
        "program T; uses Std.Path as Path; begin const Absolute: boolean := Path.IsAbsolute(Path := 'file'); const Relative: option of string := Path.RelativeTo(Target := 'a/file', Base := 'a'); const Changed: option of string := Path.ChangeExtension(Path := 'file.txt', Extension := 'log'); end.",
    );
}
