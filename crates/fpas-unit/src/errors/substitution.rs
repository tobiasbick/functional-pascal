//! Checked finite substitution for source-less error declarations.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::{ErrorInstantiationBudget, ErrorSchema, validation};
use fpas_bytecode::{ErrorInstanceKey, IdentityParameter, TypeIdentity};

impl ErrorSchema {
    /// Instantiate all payload types, retaining the declaration's messages and Cause slot.
    ///
    /// # Errors
    /// Rejects wrong arity, violated bounds, unresolved parameters and graph limits.
    pub fn instantiate(&self, key: &ErrorInstanceKey) -> Result<Self, &'static str> {
        self.instantiate_with_budget(key, &mut ErrorInstantiationBudget::default())
    }

    /// Instantiate with the link operation's shared generated-metadata budget.
    ///
    /// # Errors
    /// Rejects invalid instances or excessive expansion before copying arguments.
    pub fn instantiate_with_budget(
        &self,
        key: &ErrorInstanceKey,
        budget: &mut ErrorInstantiationBudget,
    ) -> Result<Self, &'static str> {
        self.instantiate_with_declarations(
            key,
            budget,
            &crate::equality::EqualityDeclarations::default(),
        )
    }

    /// Instantiate with verified nominal fields for equality-constrained arguments.
    pub fn instantiate_with_declarations(
        &self,
        key: &ErrorInstanceKey,
        budget: &mut ErrorInstantiationBudget,
        declarations: &crate::equality::EqualityDeclarations,
    ) -> Result<Self, &'static str> {
        self.validate()?;
        key.validate(0)?;
        if self.declaration != key.declaration || self.parameters.len() != key.arguments.len() {
            return Err("error instance does not match its schema arity or declaration");
        }
        for (argument, bound) in key.arguments.iter().zip(&self.parameters) {
            if !validation::satisfies(argument, *bound, declarations) {
                return Err("error instance violates a generic constraint");
            }
        }
        let mut remaining = fpas_bytecode::limits::MAX_LAYOUT_FIELDS;
        budget.text(&self.declaration)?;
        budget.text(&self.display_name)?;
        budget.nodes(self.variants.len())?;
        for variant in &self.variants {
            budget.text(&variant.name)?;
            if let Some(message) = &variant.message {
                budget.text(message)?;
            }
            for field in &variant.fields {
                budget.text(&field.name)?;
                expansion(&field.ty, &key.arguments, 0, &mut remaining, budget)?;
            }
        }
        let mut result = self.clone();
        result.parameters.clear();
        for variant in &mut result.variants {
            for field in &mut variant.fields {
                field.ty = replace(&field.ty, &key.arguments)?;
            }
        }
        result.validate()?;
        Ok(result)
    }
}

fn expansion(
    ty: &TypeIdentity,
    arguments: &[TypeIdentity],
    depth: usize,
    remaining: &mut usize,
    budget: &mut ErrorInstantiationBudget,
) -> Result<(), &'static str> {
    use TypeIdentity as T;
    if depth >= 64 || *remaining == 0 {
        return Err("substituted error type exceeds its graph limit");
    }
    if let T::Parameter(index) = ty {
        return expansion(
            arguments
                .get(usize::from(*index))
                .ok_or("unknown error parameter")?,
            &[],
            depth,
            remaining,
            budget,
        );
    }
    *remaining -= 1;
    budget.nodes(1)?;
    match ty {
        T::Nominal {
            declaration,
            arguments: nested,
            ..
        } => {
            budget.text(declaration)?;
            for ty in nested {
                expansion(ty, arguments, depth + 1, remaining, budget)?;
            }
        }
        T::Array(ty) | T::Channel(ty) | T::Option(ty) | T::Task(ty) => {
            expansion(ty, arguments, depth + 1, remaining, budget)?
        }
        T::Dictionary(left, right) | T::Result(left, right) => {
            expansion(left, arguments, depth + 1, remaining, budget)?;
            expansion(right, arguments, depth + 1, remaining, budget)?;
        }
        T::Callable {
            parameters, result, ..
        } => {
            for field in parameters {
                budget.text(&field.name)?;
                expansion(&field.ty, arguments, depth + 1, remaining, budget)?;
            }
            if let Some(result) = result {
                expansion(result, arguments, depth + 1, remaining, budget)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn substitute_identity(
    ty: &TypeIdentity,
    arguments: &[TypeIdentity],
    budget: &mut ErrorInstantiationBudget,
) -> Result<TypeIdentity, &'static str> {
    let mut remaining = fpas_bytecode::limits::MAX_LAYOUT_FIELDS;
    expansion(ty, arguments, 0, &mut remaining, budget)?;
    replace(ty, arguments)
}

fn replace(ty: &TypeIdentity, arguments: &[TypeIdentity]) -> Result<TypeIdentity, &'static str> {
    use TypeIdentity as T;
    Ok(match ty {
        T::Parameter(index) => arguments
            .get(usize::from(*index))
            .ok_or("unknown error type parameter")?
            .clone(),
        T::Nominal {
            kind,
            declaration,
            arguments: nested,
        } => T::Nominal {
            kind: *kind,
            declaration: declaration.clone(),
            arguments: nested
                .iter()
                .map(|ty| replace(ty, arguments))
                .collect::<Result<_, _>>()?,
        },
        T::Array(ty) => T::Array(Box::new(replace(ty, arguments)?)),
        T::Channel(ty) => T::Channel(Box::new(replace(ty, arguments)?)),
        T::Option(ty) => T::Option(Box::new(replace(ty, arguments)?)),
        T::Task(ty) => T::Task(Box::new(replace(ty, arguments)?)),
        T::Dictionary(left, right) => T::Dictionary(
            Box::new(replace(left, arguments)?),
            Box::new(replace(right, arguments)?),
        ),
        T::Result(left, right) => T::Result(
            Box::new(replace(left, arguments)?),
            Box::new(replace(right, arguments)?),
        ),
        T::Callable {
            parameters,
            result,
            variadic,
        } => T::Callable {
            variadic: *variadic,
            parameters: parameters
                .iter()
                .map(|p| {
                    Ok(IdentityParameter {
                        name: p.name.clone(),
                        mutable: p.mutable,
                        ty: replace(&p.ty, arguments)?,
                    })
                })
                .collect::<Result<_, &'static str>>()?,
            result: result
                .as_ref()
                .map(|ty| replace(ty, arguments).map(Box::new))
                .transpose()?,
        },
        _ => ty.clone(),
    })
}
