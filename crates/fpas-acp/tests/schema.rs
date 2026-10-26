#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! The SDK schema graph preserves the information needed by native type registration.

use fpas_acp::schema::{self, Version};

#[test]
fn pinned_schema_keeps_nominal_ids_discriminators_and_deserialization_rules() {
    for version in [Version::V1, Version::V2] {
        let document = schema::document(version);
        let definitions = document["$defs"].as_object().expect("schema definitions");
        assert_eq!(definitions["SessionId"]["type"], "string");
        assert_eq!(
            definitions["InitializeRequest"]["properties"]["protocolVersion"]["$ref"],
            "#/$defs/ProtocolVersion"
        );
        if version == Version::V1 {
            assert_eq!(
                definitions["InitializeRequest"]["properties"]["clientInfo"]["x-deserialize-default-on-error"],
                true
            );
            assert_eq!(
                definitions["ContentBlock"]["discriminator"]["propertyName"],
                "type"
            );
        }
    }
}

#[test]
fn every_reachable_sdk_declaration_has_a_native_shape() {
    for version in [Version::V1, Version::V2] {
        let catalog = fpas_acp::binding::catalog(version).expect("SDK native catalog");
        for name in schema::document(version)["$defs"]
            .as_object()
            .unwrap()
            .keys()
        {
            assert!(
                catalog.declarations.contains_key(name),
                "{version:?} {name}"
            );
        }
    }
}
