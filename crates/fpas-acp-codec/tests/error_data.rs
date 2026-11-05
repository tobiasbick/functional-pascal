#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Independent protocol fixtures fail at their observable boundary"
)]
//! Schema examples must not narrow the public arbitrary-JSON error payload.

use fpas_acp_codec::{
    RawJson,
    binding::{CatalogCodec as _, Shape, Value, catalog},
    schema::Version,
};

#[test]
fn error_data_remains_lossless_arbitrary_json_in_both_protocol_families() {
    for version in [Version::V1, Version::V2] {
        let catalog = catalog(version).unwrap();
        let Shape::Record(fields) = &catalog.declarations["Error"].shape else {
            panic!("error record");
        };
        let index = fields
            .iter()
            .position(|field| field.name == "Data")
            .unwrap();
        for data in [
            r#"{"n":18446744073709551616,"large":1e400,"negative":-0}"#,
            r#"[true,"peer",18446744073709551616]"#,
            r#"{"requested":2,"supported":[1]}"#,
            r#"{"sessionId":"missing"}"#,
            r#""peer-specific error""#,
            "false",
            "42",
        ] {
            let raw = RawJson::parse(&format!(
                r#"{{"code":-32022,"message":"peer","data":{data}}}"#
            ))
            .unwrap();
            let error = catalog.decode("Error", &raw).unwrap();
            let Value::Record { fields, .. } = &error else {
                panic!("error value");
            };
            assert!(
                matches!(&fields[index], Value::Some(value) if matches!(&**value, Value::Raw(raw) if raw.text() == data))
            );
            let encoded = catalog.encode("Error", &error).unwrap();
            assert_eq!(encoded.member("data").unwrap().unwrap().text(), data);
        }
        for text in [
            r#"{"code":-32022,"message":"peer"}"#,
            r#"{"code":-32022,"message":"peer","data":null}"#,
        ] {
            let error = catalog
                .decode("Error", &RawJson::parse(text).unwrap())
                .unwrap();
            let Value::Record { fields, .. } = error else {
                panic!("error value");
            };
            assert!(matches!(fields[index], Value::None));
        }
    }
}
