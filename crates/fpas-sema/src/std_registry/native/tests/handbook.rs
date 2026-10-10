//! Public handbook signatures must cover the catalog exactly once per owner.

use super::*;

#[test]
fn handbook_tables_match_every_native_signature_and_factory() {
    let strings = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/pascal/language/types/string/README.md"
    ));
    let arrays = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/pascal/language/types/array/README.md"
    ));
    let dictionaries = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/pascal/language/types/dictionary-operations.md"
    ));
    let options = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/pascal/language/types/option-operations.md"
    ));
    let results = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/pascal/language/types/result-operations.md"
    ));
    for entry in native_operations() {
        let (owner, source) = match entry.receiver {
            NativeReceiver::String => ("Text", strings),
            NativeReceiver::Array | NativeReceiver::StringArray => ("Items", arrays),
            NativeReceiver::Dictionary => ("Values", dictionaries),
            NativeReceiver::Option => ("Value", options),
            NativeReceiver::Result => ("Value", results),
            NativeReceiver::StringFactory => ("string", strings),
            NativeReceiver::ArrayFactory => ("array", arrays),
        };
        let signature = (entry.signature)();
        let parameters = if signature.variadic {
            "Arguments...".into()
        } else {
            signature
                .params
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        };
        let result = if *signature.return_type == Ty::Unit {
            String::new()
        } else {
            format!(": {}", signature.return_type)
        };
        let row = format!("| `{owner}.{}({parameters}){result}` |", entry.name);
        assert_eq!(
            source.matches(&row).count(),
            1,
            "missing or duplicate {row}"
        );
    }
}
