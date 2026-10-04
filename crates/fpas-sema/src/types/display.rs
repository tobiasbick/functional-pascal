//! Canonical type text used by diagnostics and intrinsic declarations.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use std::fmt::{self, Display, Formatter};

use super::Ty;

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Integer => write!(f, "integer"),
            Ty::Real => write!(f, "real"),
            Ty::Boolean => write!(f, "boolean"),
            Ty::String => write!(f, "string"),
            Ty::Unit => write!(f, "unit"),
            Ty::Array(inner) => write!(f, "array of ({inner})"),
            Ty::Channel(inner) => write!(f, "channel of ({inner})"),
            Ty::Record(record) => write_application(f, &record.name, &record.type_args),
            Ty::Enum(enumeration) => {
                write_application(f, &enumeration.name, &enumeration.type_args)
            }
            Ty::Function(function) => {
                write!(f, "function(")?;
                for (index, parameter) in function.params.iter().enumerate() {
                    if index > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", parameter.name, parameter.ty)?;
                }
                write!(f, "): {}", function.return_type)
            }
            Ty::Procedure(procedure) => {
                write!(f, "procedure(")?;
                for (index, parameter) in procedure.params.iter().enumerate() {
                    if index > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", parameter.name, parameter.ty)?;
                }
                write!(f, ")")
            }
            Ty::Named(name) => write!(f, "{name}"),
            Ty::Applied(name, arguments) => write_application(f, name, arguments),
            Ty::Result(ok, error) => write!(f, "Result of ({ok}, {error})"),
            Ty::Option(inner) => write!(f, "Option of ({inner})"),
            Ty::GenericParam(name, _) => write!(f, "{name}"),
            Ty::Dict(key, value) => write!(f, "dict of ({key}, {value})"),
            Ty::Task(inner) => write!(f, "task of ({inner})"),
            Ty::Error => write!(f, "<error>"),
        }
    }
}

fn write_application(f: &mut Formatter<'_>, name: &str, arguments: &[Ty]) -> fmt::Result {
    write!(f, "{name}")?;
    if !arguments.is_empty() {
        write!(f, " of (")?;
        for (index, argument) in arguments.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{argument}")?;
        }
        write!(f, ")")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
