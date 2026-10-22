//! Large captured payloads create real cycles through the public VM API.
use crate::measurement::completed_threads;
use fpas_bytecode::{CollectionSchedule, FunctionId, VerifiedExecutable};
use fpas_vm::Vm;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub const DECLARATIONS: &str = r#"
function Make(): function(): integer;
begin
  var Text := 'abcdefgh';
  for I: integer := 1 to 12 do Text := Text + Text; end for;
  var Items: array of string := [];
  for I: integer := 1 to 128 do Items.PushInPlace(Text); end for;
  var Count := 0;
  var Current: function(): integer := function(): integer begin return 0; end function;
  Current := function(): integer begin
    Count := Count + 1;
    if Count < 0 then return Current(); end if;
    return Text.Length() + Items.Length() + Count;
  end function;
  return Current;
end function;
function Invoke(Next: function(): integer): integer;
begin return Next(); end function;
procedure Exercise();
begin
  const Keep := Make();
  const Alias := Keep;
  for I: integer := 1 to 12 do
    const Wrapped: Result of (Option of function(): integer, string) := Ok(Some(Make()));
    const Garbage := Wrapped.Unwrap().Unwrap();
    if Garbage() <> 32897 then panic('garbage payload'); end if;
    if Keep() <> 32896 + I * 2 - 1 then panic('live payload'); end if;
    if Alias() <> 32896 + I * 2 then panic('alias payload'); end if;
  end for;
end procedure;
"#;

pub fn image(source: &str) -> VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).unwrap()
}

pub fn schedule() -> CollectionSchedule {
    CollectionSchedule {
        allocations: NonZeroUsize::new(1).unwrap(),
        mutations: NonZeroUsize::new(1).unwrap(),
        safe_points: NonZeroUsize::new(1).unwrap(),
    }
}

pub fn function(image: &VerifiedExecutable, name: &str) -> FunctionId {
    let executable = image.executable();
    let index = executable
        .functions
        .iter()
        .position(|function| {
            executable
                .strings
                .get(function.name)
                .is_some_and(|found| found.eq_ignore_ascii_case(name))
        })
        .unwrap();
    FunctionId::try_from_index(index).unwrap()
}

pub fn vm() {
    for failing in [false, true] {
        let tail = if failing {
            "panic('expected memory failure');"
        } else {
            ""
        };
        let image = Arc::new(image(&format!(
            "program Memory; {DECLARATIONS} begin Exercise(); {tail} end."
        )));
        for config in [CollectionSchedule::default(), schedule()] {
            completed_threads(
                if failing {
                    "vm failure"
                } else {
                    "vm completion"
                },
                12,
                || {
                    let result = Vm::from_shared(image.clone())
                        .with_collection_schedule(config)
                        .run();
                    if failing {
                        assert!(
                            result
                                .unwrap_err()
                                .message
                                .contains("expected memory failure")
                        );
                    } else {
                        drop(result.unwrap());
                    }
                },
            );
        }
    }
}

pub fn tasks() {
    for failing in [false, true] {
        let tail = if failing {
            "panic('expected memory failure');"
        } else {
            ""
        };
        let image = Arc::new(image(&format!(
            r#"program MemoryTasks;
uses Std.Tasks as Tasks;
{DECLARATIONS}
function Work(): integer;
begin Exercise(); {tail} return 42; end function;
begin
  scope
    for I: integer := 1 to 4 do
      const Child := go Work();
      const Peer := go Work();
      if Tasks.Wait(Child) <> 42 then panic('task result'); end if;
      if Tasks.Wait(Peer) <> 42 then panic('peer result'); end if;
    end for;
  end scope;
end."#
        )));
        for config in [CollectionSchedule::default(), schedule()] {
            completed_threads(
                if failing {
                    "task failure and scope cancellation"
                } else {
                    "task completion"
                },
                8,
                || {
                    let result = Vm::from_shared(image.clone())
                        .with_collection_schedule(config)
                        .run();
                    if failing {
                        assert!(
                            result
                                .unwrap_err()
                                .message
                                .contains("expected memory failure")
                        );
                    } else {
                        drop(result.unwrap());
                    }
                },
            );
        }
    }
}
