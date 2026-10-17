//! Built-in routine and value names from other languages and their FPAS forms.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::super::Checker;
use fpas_diagnostics::ForeignHabit;
use fpas_parser::{DesignatorPart, Expr};

/// One foreign built-in name and the FPAS text that replaces it.
struct ForeignBuiltin {
    /// Lowercase name; qualified names use the canonical `std.unit.member` spelling.
    name: &'static str,
    /// Replacement; `{0}` and `{1}` stand for the written arguments, `{unit}` for the alias.
    template: &'static str,
    /// Placeholder text for `{0}` and `{1}` when an argument is missing or not a plain name.
    defaults: [&'static str; 2],
    /// `Std` unit and default alias the replacement uses, if any.
    unit: Option<(&'static str, &'static str)>,
    explanation: &'static str,
}

/// Foreign built-ins; one entry per name.
const FOREIGN_BUILTINS: &[ForeignBuiltin] = &[
    ForeignBuiltin {
        name: "exit",
        template: "return {0};",
        defaults: ["Value", ""],
        unit: None,
        explanation: "Delphi `Exit` does not exist; leave a routine with `return`, without a value in a procedure.",
    },
    ForeignBuiltin {
        name: "inc",
        template: "{0} := {0} + {1};",
        defaults: ["X", "1"],
        unit: None,
        explanation: "Pascal `Inc` does not exist; change a variable with an assignment.",
    },
    ForeignBuiltin {
        name: "dec",
        template: "{0} := {0} - {1};",
        defaults: ["X", "1"],
        unit: None,
        explanation: "Pascal `Dec` does not exist; change a variable with an assignment.",
    },
    ForeignBuiltin {
        name: "length",
        template: "{0}.Length()",
        defaults: ["Value", ""],
        unit: None,
        explanation: "Built-in type operations are methods, available without imports.",
    },
    ForeignBuiltin {
        name: "low",
        template: "0",
        defaults: ["", ""],
        unit: None,
        explanation: "Pascal `Low` does not exist; arrays and strings start at index 0.",
    },
    ForeignBuiltin {
        name: "high",
        template: "{0}.Length() - 1",
        defaults: ["Items", ""],
        unit: None,
        explanation: "Pascal `High` does not exist; the last index is one less than the length.",
    },
    ForeignBuiltin {
        name: "trystrtoint",
        template: "{unit}.Int({0})",
        defaults: ["Text", ""],
        unit: Some(("Std.Parse", "Parse")),
        explanation: "Delphi `TryStrToInt` does not exist; `Std.Parse` returns a Result, so match `Ok(const Value)` instead of passing a variable.",
    },
    ForeignBuiltin {
        name: "inttostr",
        template: "{unit}.IntToStr({0})",
        defaults: ["N", ""],
        unit: Some(("Std.Conv", "Conv")),
        explanation: "Conversions live in `Std.Conv`.",
    },
    ForeignBuiltin {
        name: "write",
        template: "{unit}.WriteText({0})",
        defaults: ["Text", ""],
        unit: Some(("Std.Console", "Console")),
        explanation: "Pascal `Write` is `WriteText` in `Std.Console`; it writes without a line break.",
    },
    ForeignBuiltin {
        name: "std.console.write",
        template: "{unit}.WriteText({0})",
        defaults: ["Text", ""],
        unit: Some(("Std.Console", "Console")),
        explanation: "Pascal `Write` is `WriteText` in `Std.Console`; it writes without a line break.",
    },
    ForeignBuiltin {
        name: "nil",
        template: "None",
        defaults: ["", ""],
        unit: None,
        explanation: "FPAS has no pointers; absence is `None` of `Option of T`.",
    },
];

impl Checker {
    /// Returns the help for an unresolved name that is a known foreign built-in, using the
    /// written arguments of a call where they are plain names or literals.
    pub(crate) fn foreign_builtin_hint(&self, name: &str, args: &[Expr]) -> Option<String> {
        let canonical = self.scopes.imports.canonical_name(name);
        let builtin = FOREIGN_BUILTINS
            .iter()
            .find(|builtin| builtin.name == canonical || builtin.name.eq_ignore_ascii_case(name))?;
        let argument = |index: usize| {
            args.get(index)
                .and_then(argument_text)
                .unwrap_or_else(|| builtin.defaults[index].to_owned())
        };
        let mut replacement = builtin
            .template
            .replace("{0}", &argument(0))
            .replace("{1}", &argument(1));
        let mut explanation = builtin.explanation.to_owned();
        if let Some((unit, default_alias)) = builtin.unit {
            let alias = self
                .scopes
                .imports
                .source_name_for(unit, &format!("{unit}.Member"))
                .and_then(|qualified| qualified.strip_suffix(".Member").map(str::to_owned));
            let alias = alias.unwrap_or_else(|| {
                explanation.push_str(&format!(" Import `uses {unit} as {default_alias};`."));
                default_alias.to_owned()
            });
            replacement = replacement.replace("{unit}", &alias);
        }
        Some(
            ForeignHabit {
                found: name,
                replacement: &replacement,
                explanation: &explanation,
            }
            .help(),
        )
    }
}

/// Source-like text of a plain argument: a name, a dotted name or a literal.
pub(super) fn argument_text(argument: &Expr) -> Option<String> {
    match argument {
        Expr::Designator(designator) => designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(..) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join(".")),
        Expr::Integer(value, _) => Some(value.to_string()),
        Expr::Str(text, _) => Some(format!("'{}'", text.replace('\'', "''"))),
        _ => None,
    }
}
