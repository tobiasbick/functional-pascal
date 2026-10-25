//! Automatic output and stack-safe lifetime of stored native errors.

#![allow(
    clippy::expect_used,
    reason = "value fixtures assert valid runtime layouts"
)]

use fpas_bytecode::{
    EnumTypeId, EnumVariantId, ErrorCause, RecordTypeId, RuntimeEnumLayout, RuntimeErrorVariant,
    RuntimeRecordLayout, SharedEnum, SharedRecord, Value,
};
use std::sync::Arc;

fn layout(
    family: u16,
    variant: u16,
    name: &str,
    fields: &[&str],
    message: Option<&str>,
    cause: Option<ErrorCause>,
) -> Arc<RuntimeEnumLayout> {
    Arc::new(RuntimeEnumLayout {
        error: Some(RuntimeErrorVariant {
            hide_payload: false,
            message: message.map(str::to_owned),
            cause,
        }),
        enumeration: EnumTypeId::new(family),
        variant_id: EnumVariantId::new(variant),
        type_name: "Example.Error".to_owned(),
        variant: name.to_owned(),
        fields: fields.iter().map(|name| (*name).to_owned()).collect(),
    })
}

#[test]
fn automatic_frames_quote_nested_fields_and_keep_the_original_cause() {
    let leaf = Value::Enum(
        SharedEnum::try_new(
            layout(0, 0, "Missing", &["Path"], Some("File not found"), None),
            vec![Value::Str("settings.toml".into())],
        )
        .expect("leaf"),
    );
    let wrapper = Value::Enum(
        SharedEnum::try_new(
            layout(
                1,
                1,
                "Load",
                &["Context", "Cause", "Related"],
                Some("Cannot load\nconfiguration"),
                Some(ErrorCause {
                    field: 1,
                    family: EnumTypeId::new(0),
                }),
            ),
            vec![
                Value::Array(
                    vec![
                        Value::option_some(Value::Str("a\n\"b\"\\c".into())),
                        Value::Str("".into()),
                    ]
                    .into(),
                ),
                leaf.clone(),
                leaf,
            ],
        )
        .expect("wrapper"),
    );
    assert_eq!(
        wrapper.to_string(),
        concat!(
            "Example.Error.Load: Cannot load\\nconfiguration\n",
            "  Context = [Some(\"a\\n\\\"b\\\"\\\\c\"), \"\"]\n",
            "  Related = Example.Error.Missing(\"settings.toml\")\n",
            "caused by Example.Error.Missing: File not found\n",
            "  Path = \"settings.toml\"",
        )
    );
    let mut ordinary = (*layout(2, 2, "Text", &["Value"], None, None)).clone();
    ordinary.error = None;
    let ordinary = Value::Enum(SharedEnum::new(
        Arc::new(ordinary),
        vec![Value::Str("a\nb".into())],
    ));
    assert_eq!(ordinary.to_string(), "Example.Error.Text(a\nb)");
}

#[test]
fn deep_causes_and_payloads_render_and_drop_on_a_small_host_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let end = layout(0, 0, "End", &[], None, None);
            let link = layout(
                0,
                1,
                "Link",
                &["Cause"],
                None,
                Some(ErrorCause {
                    field: 0,
                    family: EnumTypeId::new(0),
                }),
            );
            let mut value = Value::Enum(SharedEnum::try_new(end, vec![]).expect("end"));
            for _ in 0..20_000 {
                value =
                    Value::Enum(SharedEnum::try_new(Arc::clone(&link), vec![value]).expect("link"));
            }
            let output = value.to_string();
            assert_eq!(output.lines().count(), 20_001);
            assert!(output.ends_with("caused by Example.Error.End"));
            drop(value);
            let record = Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "Example.Box".to_owned(),
                fields: vec!["Value".to_owned()],
            });
            let mut payload = Value::Integer(0);
            for _ in 0..20_000 {
                payload = Value::Record(SharedRecord::new(Arc::clone(&record), vec![payload]));
            }
            let error = Value::Enum(
                SharedEnum::try_new(
                    layout(0, 2, "Data", &["Payload"], None, None),
                    vec![payload],
                )
                .expect("deep ordinary payload inside an error"),
            );
            let output = error.to_string();
            assert!(output.starts_with("Example.Error.Data\n  Payload = Example.Box{Value: "));
            assert_eq!(output.matches("Example.Box{Value: ").count(), 20_000);
            drop(error);
        })
        .expect("small-stack thread")
        .join()
        .expect("stack-safe cause traversal");
}
