//! Distinct type identity, conversions, and constants across compiled-unit interfaces.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use super::{build, run, write};

#[test]
fn imported_distinct_types_convert_explicitly_through_plain_and_aliased_imports() {
    let root = std::env::temp_dir().join(format!("fpas-distinct-types-{}", std::process::id()));
    let main = root.join("src/main.fpas");
    write(
        &root.join("demo.fpasprj"),
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(
        &root.join("src/ids.fpas"),
        "unit Demo.Ids;
      public type UserId = distinct integer;
      public type Label = distinct string;
      public const Admin: UserId := UserId(7);
      public function Next(Id: UserId): UserId; begin return UserId(integer(Id) + 1); end function;
      end unit;",
    );
    write(
        &root.join("src/accounts.fpas"),
        "unit Demo.Accounts;
      uses Demo.Ids as Ids;
      public function Promote(Id: Ids.UserId): Ids.UserId;
      begin return Ids.Next(Ids.UserId(integer(Id) + 1)); end function;
      public function Describe(Name: Ids.Label): string; begin return string(Name); end function;
      end unit;",
    );
    write(
        &main,
        "program App; uses Std.Console, Demo.Ids, Demo.Accounts;
      begin
        const First: UserId := Next(Admin);
        const Second: UserId := Promote(First);
        WriteLn(integer(Admin), ':', integer(First), ':', integer(Second), ':', Describe(Label('root')));
      end.",
    );
    assert_eq!(run(build(&root, &main)), ["7:8:10:root"]);
    let second = build(&root, &main);
    assert_eq!(second.counters().sidecar_reused, 2);
    assert_eq!(run(second), ["7:8:10:root"]);
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}
