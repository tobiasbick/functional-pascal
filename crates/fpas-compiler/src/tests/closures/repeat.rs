use super::super::assert_succeeds;

#[test]
fn repeat_condition_discovers_anonymous_closures_and_mutable_captures() {
    assert_succeeds(
        r#"
program RepeatClosures;
function Evaluate(Predicate: function(): boolean): boolean;
begin
  return Predicate()
end;
procedure Check();
begin
  mutable var Count: integer := 0;
  repeat
    Count := Count + 1
  until Evaluate(function(): boolean begin return Count = 3 end);
  if Count <> 3 then panic('repeat capture mismatch');
  repeat
    Count := Count + 1
  until Evaluate(function(): boolean begin return true end);
  if Count <> 4 then panic('repeat closure mismatch')
end;
begin
  Check()
end.
"#,
    );
}

#[test]
fn repeat_condition_discovers_bound_method_values() {
    assert_succeeds(
        r#"
program RepeatMethod;
type Predicate = record
  Value: boolean;
  function Evaluate(Self: Predicate): boolean;
  begin
    return Self.Value
  end;
end;
function Invoke(Check: function(): boolean): boolean;
begin
  return Check()
end;
begin
  var Check: Predicate := record Value := true; end;
  repeat
    begin end
  until Invoke(Check.Evaluate)
end.
"#,
    );
}
