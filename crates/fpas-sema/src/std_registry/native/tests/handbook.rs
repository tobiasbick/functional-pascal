//! Public handbook signatures must cover the catalog exactly once per owner.

use super::*;

#[test]
fn handbook_tables_match_every_native_signature_and_factory() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for entry in native_operations() {
        let page = entry.receiver.handbook();
        let source = std::fs::read_to_string(repository.join(page))
            .unwrap_or_else(|error| panic!("read handbook {page}: {error}"));
        let owner = match entry.receiver {
            NativeReceiver::ByteBuffer => "Buffer",
            NativeReceiver::String => "Text",
            NativeReceiver::Array | NativeReceiver::StringArray => "Items",
            NativeReceiver::Dictionary => "Values",
            NativeReceiver::Option | NativeReceiver::Result | NativeReceiver::Error => "Value",
            NativeReceiver::StringFactory => "string",
            NativeReceiver::ArrayFactory => "array",
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
        let row = format!("| `{owner}.{}({parameters}){result}` |", entry.name)
            .replace("Std.Bytes.", "Bytes.");
        assert_eq!(
            source.matches(&row).count(),
            1,
            "missing or duplicate {row} in {page}"
        );
    }
}
