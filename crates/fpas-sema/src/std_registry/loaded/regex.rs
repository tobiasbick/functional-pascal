//! The single public interface for compiled Unicode regular expressions.
use super::super::{define_func, p};
use super::type_registration::{register_opaque_type, register_record_type};
use crate::{check::Checker, types::Ty};

pub(super) fn register(checker: &mut Checker) {
    let error = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Regex,
    );
    let pattern = register_opaque_type(checker, "Std.Text.Regex.Pattern");
    let options = register_record_type(
        checker,
        "Std.Text.Regex.Options",
        [
            "CaseInsensitive",
            "MultiLine",
            "DotMatchesNewLine",
            "IgnoreWhitespace",
            "SwapGreed",
            "CrLf",
        ]
        .into_iter()
        .map(|name| (name.into(), Ty::Boolean))
        .collect(),
    );
    let span = register_record_type(
        checker,
        "Std.Text.Regex.Span",
        vec![
            ("Start".into(), Ty::Integer),
            ("Length".into(), Ty::Integer),
            ("Text".into(), Ty::String),
        ],
    );
    let capture = register_record_type(
        checker,
        "Std.Text.Regex.Capture",
        vec![
            ("Name".into(), Ty::Option(Box::new(Ty::String))),
            ("Value".into(), Ty::Option(Box::new(span.clone()))),
        ],
    );
    let found = register_record_type(
        checker,
        "Std.Text.Regex.Match",
        vec![
            ("Span".into(), span),
            ("Groups".into(), Ty::Array(Box::new(capture))),
        ],
    );
    define_func(
        checker,
        "Std.Text.Regex.DefaultOptions",
        vec![],
        options.clone(),
    );
    for (name, parameters, output) in [
        (
            "Compile",
            vec![p("Expression", Ty::String), p("Options", options)],
            pattern.clone(),
        ),
        (
            "IsMatch",
            vec![p("Pattern", pattern.clone()), p("Text", Ty::String)],
            Ty::Boolean,
        ),
        (
            "Find",
            vec![
                p("Pattern", pattern.clone()),
                p("Text", Ty::String),
                p("Start", Ty::Integer),
            ],
            Ty::Option(Box::new(found.clone())),
        ),
        (
            "FindAll",
            vec![
                p("Pattern", pattern.clone()),
                p("Text", Ty::String),
                p("Start", Ty::Integer),
            ],
            Ty::Array(Box::new(found)),
        ),
        (
            "Replace",
            vec![
                p("Pattern", pattern.clone()),
                p("Text", Ty::String),
                p("Replacement", Ty::String),
                p("Limit", Ty::Integer),
            ],
            Ty::String,
        ),
        (
            "Split",
            vec![
                p("Pattern", pattern.clone()),
                p("Text", Ty::String),
                p("Limit", Ty::Integer),
            ],
            Ty::Array(Box::new(Ty::String)),
        ),
        ("Escape", vec![p("Text", Ty::String)], Ty::String),
        (
            "GroupNames",
            vec![p("Pattern", pattern)],
            Ty::Array(Box::new(Ty::Option(Box::new(Ty::String)))),
        ),
    ] {
        define_func(
            checker,
            &format!("Std.Text.Regex.{name}"),
            parameters,
            Ty::Result(Box::new(output), Box::new(error.clone())),
        );
    }
}
