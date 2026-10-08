//! Constructor lowering through canonical interfaces, facades, and artifact reuse.

use super::{build, run, write};

#[test]
fn imported_constructors_and_reexported_aliases_preserve_defaults_and_identity() {
    let root =
        std::env::temp_dir().join(format!("fpas-record-construction-{}", std::process::id()));
    let main = root.join("src/main.fpas");
    write(
        &root.join("demo.fpasprj"),
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(
        &root.join("src/model.fpas"),
        "unit Demo.Model;
      public type Point = record public X: integer; public Y: integer := 2; end record;
      public type Location = Point;
      public type Secret = record public X: integer; Hidden: integer := 7; end record;
      public type SecretAlias = Secret;
      public function CreateSecret(): Secret; begin return SecretAlias(X := 5); end function;
      end unit;",
    );
    write(
        &root.join("src/facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as M; public type Position = M.Location; end unit;",
    );
    write(&main, "program App; uses Std.Console, Demo.Model as M, Demo.Facade as F;
      function Move(P: M.Point): M.Point; begin return M.Point(Y := P.Y + 1, X := P.X + 2); end function;
      begin
        const P: F.Position := F.Position(X := 3);
        const Q: M.Point := Move(P);
        const S: M.Secret := M.CreateSecret();
        WriteLn(P.X, ':', P.Y, ':', Q.X, ':', Q.Y, ':', S.X);
      end.");
    assert_eq!(run(build(&root, &main)), ["3:2:5:3:5"]);
    let second = build(&root, &main);
    assert_eq!(second.counters().sidecar_reused, 2);
    assert_eq!(run(second), ["3:2:5:3:5"]);
    write(
        &main,
        "program App; uses Demo.Facade as F; type Point = record X: integer := 90; Y: integer := 80; end record; begin const P: F.Position := F.Position(X := 4); const Local: Point := Point(); if (P.Y <> 2) or (Local.X <> 90) or (Local.Y <> 80) then panic('nominal defaults'); end if; end.",
    );
    assert!(run(build(&root, &main)).is_empty());
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}
