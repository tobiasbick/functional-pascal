use super::super::assert_succeeds;

#[test]
fn local_send_function_shadows_imported_channel_intrinsic() {
    assert_succeeds(
        r#"program SendShadowRepro;

uses Std.Console as Console;
uses Std.Conv as Conv;
uses Std.Tasks as Tasks;

function Send(First: integer; Second: integer; Third: integer): integer;
begin
  return First + Second + Third;
end function;

begin
  if Send(1, 2, 3) <> 6 then
    panic('local Send was not selected');
  end if;

  Console.WriteLn(Conv.IntToStr(Send(1, 2, 3)));
  var Queue: channel of (integer) := Tasks.CreateChannel(1);
  discard Tasks.Send(Queue, 42);
  case Tasks.Receive(Queue) of
    when Result.Ok(const Value):
      if Value <> 42 then
        panic('qualified channel Send');
      end if;
    when Result.Error(const Message):
      panic(Message);
  end case;

  discard Tasks.CloseChannel(Queue);
end program;
"#,
    );
}

#[test]
fn local_send_procedure_shadows_intrinsic_with_several_std_units() {
    assert_succeeds(
        r#"
program SendProcedure;
uses Std.Tasks as Tasks; uses Std.Console as Console;
  mutable var Total: integer := 0;
procedure Send(First: integer; Second: integer; Third: integer);
begin
  Total := First + Second + Third;
end procedure;
begin
  Send(1, 2, 3);
  if Total <> 6 then panic('local procedure'); end if;
  Console.WriteLn('loaded another unit');
  sEnD(4, 5, 6);
  if Total <> 15 then panic('local procedure replaced by the intrinsic'); end if;
end program;
"#,
    );
}

#[test]
fn callable_parameter_and_local_variable_shadow_standard_names() {
    assert_succeeds(
        r#"
program CallableShadowing;
uses Std.Conv as Conv; uses Std.Tasks as Tasks;
function Apply(IntToStr: function(Value: integer): string): string;
begin
  return IntToStr(42);
end function;
begin
  var Send: function(Value: integer): integer := function(Value: integer): integer
  begin
    return Value + 1;
  end function;
  if Send(3) <> 4 then panic('local callable'); end if;
  if Apply(function(Value: integer): string
  begin
    return 'local';
  end function) <> 'local' then panic('callable parameter'); end if;
  if Conv.IntToStr(42) <> '42' then panic('qualified intrinsic'); end if;
end program;
"#,
    );
}

#[test]
fn local_function_with_matching_intrinsic_arity_keeps_its_behavior() {
    assert_succeeds(
        r#"
program SameArityShadowing;
uses Std.Math as Math;
function Abs(Value: integer): integer;
begin
  return Value - 100;
end function;
begin
  if Abs(-2) <> -102 then panic('silent intrinsic substitution'); end if;
  if Math.Abs(-2) <> 2 then panic('qualified intrinsic'); end if;
end program;
"#,
    );
}
