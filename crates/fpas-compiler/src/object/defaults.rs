//! Link-visible default implementations remain absent from source symbol lookup.

use fpas_unit::interface::{FieldDefault, InterfaceType};
use std::collections::BTreeSet;

/// Expose initializer definitions to the linker through every exported type route.
pub(super) fn collect(ty: &InterfaceType, public: &mut BTreeSet<String>) {
    match ty {
        InterfaceType::Record(record) => {
            for field in &record.fields {
                if let Some(FieldDefault::Initializer { name, .. }) = &field.default_value {
                    public.insert(name.to_ascii_lowercase());
                }
            }
        }
        InterfaceType::Array(inner)
        | InterfaceType::Channel(inner)
        | InterfaceType::Option(inner)
        | InterfaceType::Task(inner) => collect(inner, public),
        InterfaceType::Dictionary(left, right) | InterfaceType::Result(left, right) => {
            collect(left, public);
            collect(right, public);
        }
        _ => {}
    }
}
