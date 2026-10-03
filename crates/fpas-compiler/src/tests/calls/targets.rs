use super::super::assert_succeeds;

#[test]
fn returned_indexed_parenthesized_and_anonymous_targets_execute() {
    assert_succeeds(
        r#"
program Targets;
type Handler = function(Value: integer): integer;
type Holder = record Apply: Handler; Notify: procedure(); end record;
function Make(Base: integer): Handler;
begin
  return function(Value: integer): integer begin return Base + Value; end function;
end function;
procedure Notify(); begin null; end procedure;
function MakeHolder(): Holder;
begin return record Apply := Make(30); Notify := Notify; end record; end function;
begin
  var Functions: array of Handler := [Make(10), Make(20)];
  if Make(3)(5) <> 8 then panic('returned target'); end if;
  if Functions[1](2) <> 22 then panic('indexed target'); end if;
  var Lookup: dict of string to Handler := ['answer': Make(40)];
  if Lookup['answer'](2) <> 42 then panic('dictionary target'); end if;
  if ([Make(40)])[0](2) <> 42 then panic('literal target'); end if;
  if (Functions[0])(2) <> 12 then panic('parenthesized target'); end if;
  if (function(X: integer): integer begin return X * 2; end function)(21) <> 42 then panic('anonymous target'); end if;
  if MakeHolder().Apply(12) <> 42 then panic('field target'); end if;
  (MakeHolder().Notify)();
  (procedure() begin null; end procedure)();
  var Actions: array of procedure() := [Notify];
  Actions[0]();
  discard Make(1)(2);
end program;
"#,
    );
}

#[test]
fn target_arguments_and_body_execute_once_in_order() {
    assert_succeeds(
        r#"
program Order;
mutable var Trace: string := '';
type Handler = function(A: integer; B: integer): integer;
function Make(): Handler;
begin
  Trace := Trace + 'T';
  return function(A: integer; B: integer): integer
  begin Trace := Trace + 'C'; return A + B; end function;
end function;
function Argument(Marker: string): integer;
begin Trace := Trace + Marker; return 1; end function;
function MakeArray(): array of Handler; begin return [Make()]; end function;
function Pick(): integer; begin Trace := Trace + 'I'; return 0; end function;
begin
  discard Make()(Argument('A'), Argument('B'));
  if Trace <> 'TABC' then panic(Trace); end if;
  Trace := '';
  discard MakeArray()[Pick()](Argument('A'), Argument('B'));
  if Trace <> 'TIABC' then panic(Trace); end if;
end program;
"#,
    );
}

#[test]
fn try_arguments_preserve_target_and_earlier_arguments_across_blocks() {
    assert_succeeds(
        r#"
program TryOrder;
mutable var Trace: string := '';
type Handler = function(A: integer; B: integer): integer;
function Make(): Handler;
begin Trace := Trace + 'T';
  return function(A: integer; B: integer): integer
  begin Trace := Trace + 'C'; return A * 10 + B; end function;
end function;
function Argument(Marker: string; Fail: boolean): Result of integer, string;
begin
  Trace := Trace + Marker;
  if Fail then return Error('failed'); end if;
  return Ok(2);
end function;
function Invoke(Fail: boolean): Result of integer, string;
begin return Ok(Make()(try Argument('A', Fail), try Argument('B', false))); end function;
begin
  discard Invoke(false);
  if Trace <> 'TABC' then panic(Trace); end if;
  Trace := '';
  discard Invoke(true);
  if Trace <> 'TA' then panic(Trace); end if;
end program;
"#,
    );
}
