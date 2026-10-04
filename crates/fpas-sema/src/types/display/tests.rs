use super::super::{FunctionTy, ParamTy, Ty};

#[test]
fn displayed_container_types_parse_as_canonical_annotations() {
    let nested = Ty::Array(Box::new(Ty::Result(
        Box::new(Ty::Option(Box::new(Ty::Integer))),
        Box::new(Ty::String),
    )));
    for ty in [
        nested,
        Ty::Dict(
            Box::new(Ty::String),
            Box::new(Ty::Array(Box::new(Ty::Real))),
        ),
        Ty::Channel(Box::new(Ty::Option(Box::new(Ty::Boolean)))),
        Ty::Task(Box::new(Ty::Integer)),
        Ty::Applied("Box".into(), vec![Ty::Array(Box::new(Ty::Integer))]),
    ] {
        let source = format!("unit Types; type Value = {ty}; end unit;");
        let (parsed, errors) = fpas_parser::parse_compilation_unit(&source);
        assert!(errors.is_empty(), "{source}: {errors:?}");
        let formatted = fpas_fmt::format_compilation_unit(&parsed);
        assert!(
            formatted.contains(&format!("type Value = {ty};")),
            "{formatted}"
        );
    }
}

#[test]
fn displayed_callable_types_preserve_nested_type_boundaries() {
    let ty = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![ParamTy {
            mutable: false,
            name: "Values".into(),
            ty: Ty::Array(Box::new(Ty::Option(Box::new(Ty::String)))),
        }],
        return_type: Box::new(Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String))),
        variadic: false,
    });
    let source = format!("unit Types; type Callback = {ty}; end unit;");
    let (parsed, errors) = fpas_parser::parse_compilation_unit(&source);
    assert!(errors.is_empty(), "{source}: {errors:?}");
    assert!(fpas_fmt::format_compilation_unit(&parsed).contains(&format!("type Callback = {ty};")));
}
