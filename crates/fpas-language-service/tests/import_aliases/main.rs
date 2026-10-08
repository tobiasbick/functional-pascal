//! Editor regressions for source-local namespaces and canonical import identities.

#![allow(
    clippy::expect_used,
    reason = "editor fixtures assert exact source positions"
)]

mod completion;
mod highlighting;
mod navigation;
mod rename;
#[path = "../support/mod.rs"]
mod support;

use std::path::PathBuf;

use fpas_language_service::LanguageService;
use support::TempDirectory;

const UNIT: &str = "unit Demo.Math;\n\npublic type Point = record\n  public X: integer;\n  Hidden: integer;\nend record;\n\n// Returns its argument.\npublic function Answer(Value: integer): integer;\nbegin\n  return Value;\nend function;\n\npublic function MakePoint(): Point;\nbegin\n  return Point( X := 1, Hidden := 0 );\nend function;\n\npublic var Origin: Point := Point( X := 1, Hidden := 0 );\n\nfunction Secret(): integer;\nbegin\n  return 0;\nend function;\nend unit;\n";

struct Fixture {
    temp: TempDirectory,
    service: LanguageService,
    main: PathBuf,
    unit: PathBuf,
}

fn fixture(source: &str) -> Fixture {
    let temp = TempDirectory::new("import-alias-tooling");
    let manifest = temp.write("demo.fpasprj", "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n");
    let unit = temp.write("src/math.fpas", UNIT);
    let main = temp.write("src/main.fpas", source);
    let service = LanguageService::load(&manifest);
    Fixture {
        temp,
        service,
        main,
        unit,
    }
}

fn apply_edits(
    source: &str,
    edits: &[fpas_language_service::RenameEdit],
    path: &std::path::Path,
) -> String {
    let mut result = source.to_owned();
    for edit in edits.iter().filter(|edit| edit.path == path) {
        result.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
    }
    result
}
