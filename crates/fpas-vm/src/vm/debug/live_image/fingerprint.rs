//! Portable fingerprints for live-image compatibility comparison.

use fpas_bytecode::{
    DebugCaptureKind, DebugType, DebugTypeId, EnumTypeId, Executable, FunctionId, FunctionInfo,
    Instruction, RecordTypeId, ReturnConvention, StringId,
};

pub(super) fn string_at(image: &Executable, id: StringId) -> &str {
    image.strings.get(id).unwrap_or("")
}

pub(super) fn function_name(image: &Executable, function: &FunctionInfo) -> String {
    string_at(image, function.name).to_owned()
}

pub(super) fn entry_name(image: &Executable) -> String {
    image
        .functions
        .get(usize::from(image.entry.get()))
        .map(|function| function_name(image, function))
        .unwrap_or_default()
}

pub(super) fn function_names(image: &Executable) -> Vec<String> {
    let mut names: Vec<String> = image
        .functions
        .iter()
        .map(|function| function_name(image, function))
        .collect();
    names.sort();
    names
}

pub(super) fn named_functions(image: &Executable) -> Vec<(String, FunctionId, &FunctionInfo)> {
    image
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            (
                function_name(image, function),
                FunctionId::try_from_index(index).unwrap_or(FunctionId::new(0)),
                function,
            )
        })
        .collect()
}

pub(super) fn function_code<'a>(
    image: &'a Executable,
    function: &FunctionInfo,
) -> Option<&'a [Instruction]> {
    let start = usize::try_from(function.code.start.get()).ok()?;
    let end = usize::try_from(function.code.end.get()).ok()?;
    image.code.get(start..end)
}

pub(super) fn capture_identity(image: &Executable, function: &FunctionInfo) -> String {
    let owner = function
        .debug
        .lexical_owner
        .and_then(|id| image.functions.get(usize::from(id.get())))
        .map(|owner| function_name(image, owner))
        .unwrap_or_default();
    let sources: Vec<String> = function
        .debug
        .capture_sources
        .iter()
        .map(|source| {
            format!(
                "{}:{}:{}",
                source.binding.get(),
                type_key(image, source.ty),
                capture_kind_name(source.kind)
            )
        })
        .collect();
    format!(
        "{}:{}:{owner}:{}",
        function.capture_count,
        function.arity,
        sources.join(",")
    )
}

const fn capture_kind_name(kind: DebugCaptureKind) -> &'static str {
    match kind {
        DebugCaptureKind::Value => "value",
        DebugCaptureKind::Cell => "cell",
        DebugCaptureKind::EnclosingCell => "enclosing_cell",
    }
}

pub(super) fn signature_identity(function: &FunctionInfo) -> (u8, ReturnConvention) {
    (function.arity, function.return_convention)
}

pub(super) fn debug_identity(image: &Executable, function: &FunctionInfo) -> String {
    format!(
        "{:?}:{:?}:{:?}",
        function.debug.scopes,
        function.debug.sequence_points,
        function.debug.result_type.map(|ty| type_key(image, ty))
    )
}

pub(super) fn record_layouts(image: &Executable) -> Vec<String> {
    image
        .records
        .iter()
        .map(|record| {
            let fields: Vec<String> = record
                .fields
                .iter()
                .map(|field| {
                    format!(
                        "{}:{}",
                        string_at(image, field.name),
                        type_key(image, field.ty)
                    )
                })
                .collect();
            let properties: Vec<String> = record
                .properties
                .iter()
                .map(|property| {
                    format!(
                        "{}:{}",
                        string_at(image, property.name),
                        string_at(image, property.getter)
                    )
                })
                .collect();
            let methods: Vec<String> = record
                .methods
                .iter()
                .map(|method| {
                    format!(
                        "{}:{}",
                        string_at(image, method.name),
                        string_at(image, method.routine)
                    )
                })
                .collect();
            format!(
                "{}|{}|{}|{}",
                string_at(image, record.name),
                fields.join(","),
                properties.join(","),
                methods.join(",")
            )
        })
        .collect()
}

pub(super) fn enum_layouts(image: &Executable) -> Vec<String> {
    let types: Vec<String> = image
        .enums
        .iter()
        .map(|layout| string_at(image, layout.name).to_owned())
        .collect();
    let variants: Vec<String> = image
        .enum_variants
        .iter()
        .map(|variant| {
            let fields: Vec<String> = variant
                .fields
                .iter()
                .map(|name| string_at(image, *name).to_owned())
                .collect();
            let types: Vec<String> = variant
                .field_types
                .iter()
                .map(|ty| type_key(image, *ty))
                .collect();
            format!(
                "{}:{}:{}:{}",
                enum_name(image, variant.owner),
                string_at(image, variant.name),
                fields.join(","),
                types.join(",")
            )
        })
        .collect();
    let mut rows = types;
    rows.extend(variants);
    rows
}

pub(super) fn global_layouts(image: &Executable) -> Vec<String> {
    image
        .globals
        .iter()
        .map(|global| {
            format!(
                "{}:{}:{}",
                string_at(image, global.name),
                type_key(image, global.ty),
                global.mutable
            )
        })
        .collect()
}

pub(super) fn source_map_identity(image: &Executable) -> String {
    let sources: Vec<&str> = image
        .source_map
        .sources
        .iter()
        .map(|source| string_at(image, *source))
        .collect();
    format!("{:?}:{:?}", sources, image.source_map.runs)
}

fn enum_name(image: &Executable, id: EnumTypeId) -> &str {
    image
        .enums
        .get(usize::from(id.get()))
        .map(|layout| string_at(image, layout.name))
        .unwrap_or("")
}

fn record_name(image: &Executable, id: RecordTypeId) -> &str {
    image
        .records
        .get(usize::from(id.get()))
        .map(|layout| string_at(image, layout.name))
        .unwrap_or("")
}

fn type_key(image: &Executable, id: DebugTypeId) -> String {
    type_key_inner(image, id, 0)
}

fn type_key_inner(image: &Executable, id: DebugTypeId, depth: u8) -> String {
    if depth > 16 {
        return "cycle".to_owned();
    }
    match image
        .debug_types
        .get(usize::try_from(id.get()).unwrap_or(usize::MAX))
    {
        None => format!("missing:{}", id.get()),
        Some(DebugType::Unit) => "unit".to_owned(),
        Some(DebugType::Boolean) => "boolean".to_owned(),
        Some(DebugType::Integer) => "integer".to_owned(),
        Some(DebugType::Real) => "real".to_owned(),
        Some(DebugType::String) => "string".to_owned(),
        Some(DebugType::Dynamic) => "dynamic".to_owned(),
        Some(DebugType::Array(element)) => {
            format!("array<{}>", type_key_inner(image, *element, depth + 1))
        }
        Some(DebugType::Dictionary { key, value }) => format!(
            "dict<{},{}>",
            type_key_inner(image, *key, depth + 1),
            type_key_inner(image, *value, depth + 1)
        ),
        Some(DebugType::Result { ok, error }) => format!(
            "result<{},{}>",
            type_key_inner(image, *ok, depth + 1),
            type_key_inner(image, *error, depth + 1)
        ),
        Some(DebugType::Option(inner)) => {
            format!("option<{}>", type_key_inner(image, *inner, depth + 1))
        }
        Some(DebugType::Function { parameters, result }) => {
            let parameters: Vec<String> = parameters
                .iter()
                .map(|parameter| type_key_inner(image, *parameter, depth + 1))
                .collect();
            format!(
                "fn<{}>{}",
                parameters.join(","),
                type_key_inner(image, *result, depth + 1)
            )
        }
        Some(DebugType::Record(record)) => format!("record<{}>", record_name(image, *record)),
        Some(DebugType::Enum(ty)) => format!("enum<{}>", enum_name(image, *ty)),
        Some(DebugType::Cell(inner)) => {
            format!("cell<{}>", type_key_inner(image, *inner, depth + 1))
        }
        Some(DebugType::Task(inner)) => {
            format!("task<{}>", type_key_inner(image, *inner, depth + 1))
        }
    }
}
