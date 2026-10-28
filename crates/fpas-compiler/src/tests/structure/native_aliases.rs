//! Native enum aliases retain field identity through compiled unit interfaces.

use super::super::{compiled_unit, parse_ok};

#[test]
fn native_enum_aliases_link_record_fields_without_a_native_import() {
    let keys = compiled_unit(
        "unit Keys; uses Std.Console as Console;
        public type Kind = Console.KeyKind;
        public type Key = Console.KeyEvent;
        end unit;",
        &[],
    );
    let interfaces = [keys.interface];
    let root = crate::compile_program_object_with_support(
        &parse_ok("program Consumer; uses Keys as Input;
          function IsEnter(Key: Input.Key): boolean;
          begin return Key.kind = Input.Kind.Enter; end function;
          begin
            const EnterKey := Input.Key(kind := Input.Kind.Enter, ch := '', shift := false, ctrl := false, alt := false, meta := false);
            const EscapeKey := EnterKey with kind := Input.Kind.Escape; end with;
            if not IsEnter(EnterKey) or IsEnter(EscapeKey) then panic('native enum alias field'); end if;
          end."),
        &interfaces, &interfaces,
    ).expect("native enum alias consumer");
    let linked = fpas_linker::link_objects(&[keys.object], &root).expect("link");
    fpas_runtime::execution::Vm::new(linked)
        .run()
        .expect("execution");
}
