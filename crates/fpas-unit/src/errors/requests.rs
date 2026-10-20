//! Bounded substitution of hidden constructor requirements through erased calls.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{ErrorConstructorRequest, TypeIdentity};

impl ErrorConstructorRequest {
    /// Apply one checked call's arguments, retaining the caller's symbolic parameters.
    ///
    /// # Errors
    /// Rejects undeclared parameters and excessive substituted identity graphs.
    pub fn substitute(
        &self,
        arguments: &[TypeIdentity],
        parameters: u16,
    ) -> Result<Self, &'static str> {
        let arity = u16::try_from(arguments.len()).map_err(|_| "too many error type arguments")?;
        self.instance.validate(arity)?;
        TypeIdentity::validate_all(arguments, parameters)?;
        let mut budget = fpas_bytecode::limits::MAX_LAYOUT_FIELDS;
        let arguments = self
            .instance
            .arguments
            .iter()
            .map(|ty| replace(ty, Some(arguments), 0, &mut budget))
            .collect::<Result<_, _>>()?;
        let request = Self {
            instance: super::ErrorInstanceKey {
                declaration: self.instance.declaration.clone(),
                arguments,
            },
            variant: self.variant,
        };
        request.instance.validate(parameters)?;
        Ok(request)
    }
}

fn replace(
    ty: &TypeIdentity,
    arguments: Option<&[TypeIdentity]>,
    depth: usize,
    budget: &mut usize,
) -> Result<TypeIdentity, &'static str> {
    use TypeIdentity as T;
    if depth >= 64 || *budget == 0 {
        return Err("substituted error identity exceeds its graph limit");
    }
    if let (T::Parameter(index), Some(arguments)) = (ty, arguments) {
        return replace(
            arguments
                .get(usize::from(*index))
                .ok_or("unknown error parameter")?,
            None,
            depth,
            budget,
        );
    }
    *budget -= 1;
    let mut child = |ty: &T| replace(ty, arguments, depth + 1, budget);
    Ok(match ty {
        T::Nominal {
            kind,
            declaration,
            arguments: nested,
        } => T::Nominal {
            kind: *kind,
            declaration: declaration.clone(),
            arguments: nested.iter().map(child).collect::<Result<_, _>>()?,
        },
        T::Array(ty) => T::Array(Box::new(child(ty)?)),
        T::Channel(ty) => T::Channel(Box::new(child(ty)?)),
        T::Task(ty) => T::Task(Box::new(child(ty)?)),
        T::Option(ty) => T::Option(Box::new(child(ty)?)),
        T::Dictionary(left, right) => {
            T::Dictionary(Box::new(child(left)?), Box::new(child(right)?))
        }
        T::Result(left, right) => T::Result(Box::new(child(left)?), Box::new(child(right)?)),
        T::Callable {
            parameters,
            result,
            variadic,
        } => T::Callable {
            parameters: parameters
                .iter()
                .map(|parameter| {
                    Ok(super::IdentityParameter {
                        name: parameter.name.clone(),
                        mutable: parameter.mutable,
                        ty: child(&parameter.ty)?,
                    })
                })
                .collect::<Result<_, &'static str>>()?,
            result: result
                .as_ref()
                .map(|ty| child(ty).map(Box::new))
                .transpose()?,
            variadic: *variadic,
        },
        other => other.clone(),
    })
}
