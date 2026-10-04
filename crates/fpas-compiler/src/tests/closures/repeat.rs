use super::super::assert_succeeds;

#[test]
fn repeat_condition_discovers_anonymous_closures_and_mutable_captures() {
    assert_succeeds(
        r#"
program RepeatClosures;
function Evaluate(Predicate: function(): boolean): boolean;
begin
  return Predicate();
end function;
procedure Check();
begin
   var Count: integer := 0;
  repeat
    Count := Count + 1;
  until Evaluate(function(): boolean begin return Count = 3; end function);
  if Count <> 3 then panic('repeat capture mismatch'); end if;
  repeat
    Count := Count + 1;
  until Evaluate(function(): boolean begin return true; end function);
  if Count <> 4 then panic('repeat closure mismatch'); end if;
end procedure;
begin
  Check();
end program;
"#,
    );
}

#[test]
fn repeat_condition_discovers_closures_with_record_captures() {
    assert_succeeds(
        r#"program RepeatRecord;

type Predicate = record
  Value: boolean;

end record;

function PredicateEvaluate(Receiver: Predicate): boolean;
begin
  return Receiver.Value;
end function;

function Invoke(Check: function(): boolean): boolean;
begin
  return Check();
end function;

begin
  const Check: Predicate := Predicate(Value := true);
  repeat
    begin
      null;
    end;
  until Invoke(function(): boolean begin return PredicateEvaluate(Check); end function);
end program;
"#,
    );
}
