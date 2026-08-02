#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler integration fixtures use direct assertions for diagnostic clarity"
)]

use fpas_compiler::compile_unit_object;

mod common;

use common::{parse_unit, run_zero_arity};

fn compile_visibility_unit() -> fpas_compiler::CompiledUnitObject {
    let unit = parse_unit(
        "unit Demo.Visibility;
         mutable var SharedHandler: Option of procedure() := None;
         public type Widget = record
           function ReadSecret(Self: Widget): integer;
           begin return 1 end;
           procedure WriteSecret(Self: Widget; Value: integer);
           begin end;
           function ReadPublic(Self: Widget): integer;
           begin return 2 end;
           procedure WritePublic(Self: Widget; Value: integer);
           begin end;
           function ReadSecretEvent(Self: Widget): Option of procedure();
           begin return SharedHandler end;
           procedure WriteSecretEvent(Self: Widget; Value: Option of procedure());
           begin SharedHandler := Value end;
           function ReadPublishedEvent(Self: Widget): Option of procedure();
           begin return SharedHandler end;
           procedure WritePublishedEvent(Self: Widget; Value: Option of procedure());
           begin SharedHandler := Value end;
           public function VisibleRead(Self: Widget): integer;
           begin return 3 end;
           public procedure VisibleWrite(Self: Widget; Value: integer);
           begin end;
           static function HiddenCreate(): Widget;
           begin return record end end;
           static procedure HiddenReset();
           begin end;
           public static function VisibleCreate(): Widget;
           begin return record end end;
           public static procedure VisibleReset();
           begin end;
           property Secret: integer read ReadSecret write WriteSecret;
           public property Published: integer read ReadPublic write WritePublic;
           event SecretEvent: procedure() read ReadSecretEvent write WriteSecretEvent;
           public event PublishedEvent: procedure() read ReadPublishedEvent write WritePublishedEvent;
         end;",
    );
    compile_unit_object(&unit, &[]).expect("visibility unit compilation")
}

#[test]
fn unit_object_records_effective_record_routine_visibility() {
    let compiled = compile_visibility_unit();
    let definitions: Vec<_> = compiled
        .object
        .definitions
        .iter()
        .filter(|definition| definition.name.starts_with("demo.visibility.widget."))
        .map(|definition| (definition.name.as_str(), definition.public))
        .collect();

    assert_eq!(
        definitions,
        [
            ("demo.visibility.widget.hiddencreate", false),
            ("demo.visibility.widget.hiddenreset", false),
            ("demo.visibility.widget.readpublic", true),
            ("demo.visibility.widget.readpublishedevent", true),
            ("demo.visibility.widget.readsecret", false),
            ("demo.visibility.widget.readsecretevent", false),
            ("demo.visibility.widget.visiblecreate", true),
            ("demo.visibility.widget.visibleread", true),
            ("demo.visibility.widget.visiblereset", true),
            ("demo.visibility.widget.visiblewrite", true),
            ("demo.visibility.widget.writepublic", true),
            ("demo.visibility.widget.writepublishedevent", true),
            ("demo.visibility.widget.writesecret", false),
            ("demo.visibility.widget.writesecretevent", false),
        ]
    );
}

#[test]
fn consumer_imports_only_effectively_public_record_routines() {
    let dependency = compile_visibility_unit();
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Visibility;
         public function Run(): integer;
         begin
           var Value: Widget := record end;
           return Value.Published
         end;",
    );
    let consumer = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .expect("consumer compilation");
    let imports: Vec<_> = consumer
        .object
        .imports
        .iter()
        .filter(|import| import.name.starts_with("demo.visibility.widget."))
        .map(|import| import.name.as_str())
        .collect();

    assert_eq!(
        imports,
        [
            "demo.visibility.widget.VisibleCreate",
            "demo.visibility.widget.VisibleRead",
            "demo.visibility.widget.VisibleReset",
            "demo.visibility.widget.VisibleWrite",
            "demo.visibility.widget.readpublic",
            "demo.visibility.widget.readpublishedevent",
            "demo.visibility.widget.writepublic",
            "demo.visibility.widget.writepublishedevent",
        ]
    );
    assert_eq!(
        run_zero_arity(
            vec![dependency.object, consumer.object],
            "demo.consumer.run"
        ),
        ["2"]
    );
}
