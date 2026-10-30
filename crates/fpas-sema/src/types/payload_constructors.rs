//! The common signatures of the built-in Option and Result payload constructors.

use super::{FunctionTy, GenericParamDef, ParamTy, Ty};

/// A built-in payload constructor, usable as an ordinary noncapturing function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadConstructor {
    /// Construct a present Option.
    Some,
    /// Construct a successful Result.
    Ok,
    /// Construct a failed Result.
    Error,
}

impl PayloadConstructor {
    /// Resolve a reserved constructor name without case sensitivity.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("Some") {
            Some(Self::Some)
        } else if name.eq_ignore_ascii_case("Ok") {
            Some(Self::Ok)
        } else if name.eq_ignore_ascii_case("Error") {
            Some(Self::Error)
        } else {
            None
        }
    }

    /// Generic signature; the receiving callable supplies every payload type.
    pub fn signature(self) -> FunctionTy {
        let value = Ty::GenericParam("T".into(), None);
        let error = Ty::GenericParam("E".into(), None);
        let (names, argument, result) = match self {
            Self::Some => (vec!["T"], value.clone(), Ty::Option(Box::new(value))),
            Self::Ok => (
                vec!["T", "E"],
                value.clone(),
                Ty::Result(Box::new(value), Box::new(error)),
            ),
            Self::Error => (
                vec!["T", "E"],
                error.clone(),
                Ty::Result(Box::new(value), Box::new(error)),
            ),
        };
        FunctionTy {
            type_params: names
                .into_iter()
                .map(|name| GenericParamDef {
                    name: name.into(),
                    constraint: None,
                })
                .collect(),
            params: vec![ParamTy::value("Value", argument)],
            return_type: Box::new(result),
            variadic: false,
        }
    }
}
