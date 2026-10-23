use crate::Vm;

const PROGRAM: &str = r"
program ProfileProbe;

uses Std.Tasks as Tasks;

function Hot(Limit: integer): integer;
begin
  var Total: integer := 0;
  for Index: integer := 1 to Limit do
    begin
      Total := Total + Index mod 7;
    end;
  end for;
  return Total;
end function;

function Caller(): integer;
begin
  return Hot(200000) + Hot(1);
end function;

function Spawned(): integer;
begin
  scope
    const First: task of integer := go Hot(100000);
    const Second: task of integer := go Hot(100000);
    return Tasks.Wait(First) + Tasks.Wait(Second);
  end scope;
end function;

begin
  const Value: integer := Caller() + Spawned();
end.
";

fn profile(source: &str) -> crate::ProfileReport {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile");
    let mut vm = Vm::new(executable);
    assert!(vm.profile_report().is_none());
    vm.enable_profile();
    vm.run().expect("run");
    vm.profile_report().expect("profile")
}

#[test]
fn batches_attribute_self_and_inclusive_samples_across_pool_tasks() {
    let report = profile(PROGRAM);
    let routine = |name: &str| {
        report
            .routines
            .iter()
            .find(|routine| routine.name == name)
            .unwrap_or_else(|| panic!("{name} missing from {:?}", report.routines))
    };
    let hot = routine("hot");
    let caller = routine("caller");
    assert_eq!(report.routines[0].name, "hot");
    assert!(!hot.entry);
    assert!(
        report
            .routines
            .iter()
            .any(|routine| routine.entry && routine.name.starts_with("$entry"))
    );
    assert!(hot.self_sample.instructions * 10 > report.total.instructions * 9);
    // Caller's 200000 iterations plus two spawned 100000-iteration tasks.
    assert!(hot.inclusive.instructions > caller.inclusive.instructions * 3 / 2);
    assert!(caller.inclusive.instructions * 5 / 2 > hot.inclusive.instructions);
    assert!(caller.self_sample.instructions < caller.inclusive.instructions / 10);
    assert!(hot.line.is_some_and(|line| (8..=12).contains(&line)));
    let hot_line = &report.lines[0];
    assert_eq!(hot_line.routine, "hot");
    assert!(hot_line.line.is_some_and(|line| (9..=12).contains(&line)));
    for routine in &report.routines {
        assert!(routine.self_sample.instructions <= routine.inclusive.instructions);
    }
    let sum: u64 = report
        .lines
        .iter()
        .map(|line| line.sample.instructions)
        .sum();
    assert_eq!(sum, report.total.instructions);
}
