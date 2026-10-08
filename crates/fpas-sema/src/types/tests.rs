use std::sync::Arc;

use super::{EnumTy, FunctionTy, ParamMode, ParamTy, ProcedureTy, RecordTy, Ty};

#[test]
fn cloning_record_type_shares_immutable_descriptor() {
    let ty = Ty::Record(Arc::new(RecordTy {
        name: "Point".to_string(),
        owner_unit: None,
        private_members: Vec::new(),
        fields: vec![("X".to_string(), Ty::Integer)],
        methods: Vec::new(),
        static_functions: Vec::new(),
        static_procedures: Vec::new(),
        events: Vec::new(),
    }));

    let cloned = ty.clone();
    let (Ty::Record(original), Ty::Record(cloned)) = (&ty, &cloned) else {
        panic!("test values must remain record types");
    };

    assert!(Arc::ptr_eq(original, cloned));
}

#[test]
fn cloning_enum_type_shares_immutable_descriptor() {
    let ty = Ty::Enum(Arc::new(EnumTy {
        name: "Direction".to_string(),
        variants: Vec::new(),
    }));

    let cloned = ty.clone();
    let (Ty::Enum(original), Ty::Enum(cloned)) = (&ty, &cloned) else {
        panic!("test values must remain enum types");
    };

    assert!(Arc::ptr_eq(original, cloned));
}

#[test]
fn procedure_types_require_matching_variadic_flag_and_param_count() {
    let fixed = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        variadic: false,
    });
    let variadic = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        variadic: true,
    });

    assert!(!fixed.compatible_with(&variadic));
    assert!(!variadic.compatible_with(&fixed));
}

#[test]
fn function_types_require_matching_variadic_flag() {
    let fixed = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        return_type: Box::new(Ty::Integer),
        variadic: false,
    });
    let variadic = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        return_type: Box::new(Ty::Integer),
        variadic: true,
    });

    assert!(!fixed.compatible_with(&variadic));
    assert!(!variadic.compatible_with(&fixed));
}

#[test]
fn callable_types_match_read_only_value_parameters() {
    let by_value = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        variadic: false,
    });
    let same_signature = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![ParamTy::value("x", Ty::Integer)],
        variadic: false,
    });

    assert!(by_value.compatible_with(&same_signature));
    assert!(same_signature.compatible_with(&by_value));
}

#[test]
fn parameter_modes_are_part_of_callable_compatibility() {
    let callable = |mode| {
        Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params: vec![ParamTy {
                name: "Value".to_string(),
                ty: Ty::Integer,
                mode,
            }],
            variadic: false,
        })
    };
    let read_only = callable(ParamMode::Value);
    let writable = callable(ParamMode::Var);
    assert!(writable.compatible_with(&callable(ParamMode::Var)));
    assert!(!read_only.compatible_with(&writable));
    assert!(!writable.compatible_with(&read_only));
    assert_eq!(writable.to_string(), "procedure(var Value: integer)");
}
