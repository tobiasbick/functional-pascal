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
      const Offset: integer := 1;
      public type Point = record public X: integer; public Y: integer := Offset + 1; end record;
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

#[test]
fn evaluated_defaults_match_local_values_through_aliases_generics_and_reused_sidecars() {
    let root = std::env::temp_dir().join(format!("fpas-evaluated-defaults-{}", std::process::id()));
    let main = root.join("src/main.fpas");
    write(
        &root.join("demo.fpasprj"),
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write(
        &root.join("src/values.fpas"),
        "unit Demo.Values; public const Base: integer := 1; end unit;",
    );
    write(
        &root.join("src/model.fpas"),
        "unit Demo.Model;
      uses Demo.Values as V;
      const Base: integer := V.Base + 2;
      public type State = enum Ready; Waiting; end enum;
      public type Defaults = record
        public Literal: integer := 3;
        public Expression: integer := 1 + 2;
        public Reference: integer := Base;
        public Parenthesized: integer := (Base);
        public Negative: integer := -(1 + 2);
        public Ratio: real := 5 / 2;
        public Caption: string := 'a' + 'b';
        public Flag: boolean := not false and (Base = 3);
        public Wrapped: integer := 9223372036854775807 + 1;
        public ShortCircuit: boolean := false and ((1 div 0) = 0);
        public Current: State := State.Ready;
        public Missing: Option of integer := (None);
        public Handler: Option of procedure() := None;
      end record;
      public type Alias = Defaults;
      public type Box of T = record
        public Value: T;
        public Count: integer := Base + 1;
        public Optional: Option of T := None;
      end record;
      end unit;",
    );
    write(
        &root.join("src/facade.fpas"),
        "unit Demo.Facade;
      uses Demo.Model as M;
      public type Settings = M.Alias;
      public type IntegerBox = M.Box of integer;
      end unit;",
    );
    write(&main, "program App;
      uses Std.Console, Demo.Model as M, Demo.Facade as F;
      const Base: integer := 90;
      function ReadValue(): integer; begin return Base; end function;
      type Local = record
        Literal: integer := 3;
        Expression: integer := 1 + 2;
        Reference: integer := Base;
        Dynamic: integer := ReadValue();
      end record;
      begin
        const P: F.Settings := F.Settings();
        const Q: M.Defaults := M.Alias(Expression := 4);
        const L: Local := Local();
        const B: F.IntegerBox := F.IntegerBox(Value := 5);
        const G: M.Box of string := M.Box(Value := 'x');
        WriteLn(P.Literal, ':', P.Expression, ':', P.Reference, ':', P.Parenthesized, ':', P.Negative);
        WriteLn(P.Ratio, ':', P.Caption, ':', P.Flag, ':', P.Wrapped, ':', P.ShortCircuit);
        WriteLn(P.Current = M.State.Ready, ':', P.Missing.IsNone(), ':', P.Handler.IsNone());
        WriteLn(Q.Expression, ':', Q.Reference, ':', L.Literal, ':', L.Expression, ':', L.Reference, ':', L.Dynamic);
        WriteLn(B.Value, ':', B.Count, ':', B.Optional.IsNone(), ':', G.Value, ':', G.Count, ':', G.Optional.IsNone());
      end.");
    let expected = [
        "3:3:3:3:-3",
        "2.5:ab:true:-9223372036854775808:false",
        "true:true:true",
        "4:3:3:3:90:90",
        "5:4:true:x:4:true",
    ];
    assert_eq!(run(build(&root, &main)), expected);
    let warm = build(&root, &main);
    assert_eq!(warm.counters().sidecar_reused, 3);
    assert_eq!(run(warm), expected);
    std::fs::remove_dir_all(root).expect("fixture cleanup");
}
