//! Anonymous closures resolve sibling routines through their enclosing declaration.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`.

use crate::tests::assert_succeeds;

#[test]
fn anonymous_closures_resolve_siblings_across_nested_lexical_owners() {
    assert_succeeds(
        "program Main;
        function Sibling(): integer; begin return 1; end function;
        function Make(Base: integer): function(): function(): integer;
          function Sibling(): integer; begin return Base; end function;
          function Increment(Value: integer): integer; begin return Value + 1; end function;
        begin return function(): function(): integer
          begin return function(): integer
            begin return Increment(Sibling()); end function;
          end function;
        end function;
        begin
          const First := Make(41); const Second := Make(9);
          if First()() <> 42 then panic('first sibling capture'); end if;
          if Second()() <> 10 then panic('second sibling capture'); end if;
          if Sibling() <> 1 then panic('global sibling'); end if;
        end program;",
    );
}

#[test]
fn captured_siblings_preserve_environments_with_shadowing_branches_and_cells() {
    assert_succeeds(
        "program Main;
        function Make(Base: integer; First: boolean): function(Base: integer): integer;
          function Sibling(): integer; begin return Base; end function;
        begin
          if First then
            return function(Base: integer): integer begin return Sibling() + Base; end function;
          else
            return function(Base: integer): integer begin return Sibling() - Base; end function;
          end if;
        end function;
        function Counter(): function(): integer;
          procedure Increment(); begin Count := Count + 1; end procedure;
        begin
          var Count := 0;
          return function(): integer begin Increment(); return Count; end function;
        end function;
        begin
          const Add := Make(42, true); const Subtract := Make(42, false);
          if Add(100) <> 142 then panic('shadowed addition'); end if;
          if Subtract(10) <> 32 then panic('shadowed subtraction'); end if;
          const Next := Counter(); const Copy := Next; const Other := Counter();
          if Next() <> 1 then panic('initial counter'); end if;
          if Copy() <> 2 then panic('shared counter cell'); end if;
          if Other() <> 1 then panic('separate activation'); end if;
        end program;",
    );
}
