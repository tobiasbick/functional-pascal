#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Fixtures report failures at the public boundary"
)]
//! Registration failure leaves catalog ownership and admission unchanged.

use fpas_mcp::{
    Catalog, Fault, Limits,
    model::{JsonObject, Tool},
};
use serde_json::{Value, json};

fn tool(schema: Value) -> Tool {
    Tool::new(
        "probe",
        "A registered probe",
        serde_json::from_value::<JsonObject>(schema).unwrap(),
    )
}

#[test]
fn malformed_and_external_schemas_fail_before_registration() {
    let mut catalog = Catalog::new(Limits {
        tools: 1,
        ..Limits::default()
    })
    .unwrap();
    for schema in [
        json!({"type":"array"}),
        json!({"type":"object","properties":{"n":{"type":"not-a-type"}}}),
        json!({"type":"object","$ref":"https://example.invalid/private-schema"}),
        json!({"type":"object","$ref":"file:///not-an-allowed-schema.json"}),
    ] {
        assert_eq!(
            catalog.register(tool(schema), 1),
            Err(Fault::InvalidSchema("InputSchema"))
        );
        assert_eq!(catalog.tools().count(), 0);
    }
    catalog.register(tool(json!({"type":"object"})), 1).unwrap();
    assert_eq!(catalog.tools().count(), 1);
}

#[test]
fn duplicate_names_and_exact_catalog_byte_limit_preserve_the_first_definition() {
    let definition = tool(json!({"type":"object"}));
    let bytes = serde_json::to_vec(&vec![definition.clone()]).unwrap().len();
    let mut too_small = Catalog::new(Limits {
        catalog_bytes: bytes - 1,
        ..Limits::default()
    })
    .unwrap();
    assert_eq!(
        too_small.register(definition.clone(), 1),
        Err(Fault::LimitExceeded("CatalogBytes", bytes - 1))
    );
    let mut catalog = Catalog::new(Limits {
        catalog_bytes: bytes,
        ..Limits::default()
    })
    .unwrap();
    catalog.register(definition.clone(), 1).unwrap();
    let mut replacement = definition.clone();
    replacement.description = Some("replacement".into());
    assert_eq!(catalog.register(replacement, 2), Err(Fault::DuplicateTool));
    assert_eq!(catalog.tool("probe"), Some(&definition));
}

#[test]
fn registration_bounds_tools_names_and_limits() {
    assert!(matches!(
        Catalog::new(Limits {
            pending_calls: 0,
            ..Limits::default()
        }),
        Err(Fault::InvalidArgument("PendingCalls"))
    ));
    let mut catalog = Catalog::new(Limits {
        tools: 1,
        ..Limits::default()
    })
    .unwrap();
    for name in ["".to_owned(), "has space".into(), "x".repeat(129)] {
        let mut invalid = tool(json!({"type":"object"}));
        invalid.name = name.into();
        assert_eq!(
            catalog.register(invalid, 1),
            Err(Fault::InvalidArgument("ToolName"))
        );
    }
    catalog.register(tool(json!({"type":"object"})), 1).unwrap();
    let mut another = tool(json!({"type":"object"}));
    another.name = "another".into();
    assert_eq!(
        catalog.register(another, 2),
        Err(Fault::LimitExceeded("Tools", 1))
    );
}
