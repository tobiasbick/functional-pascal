//! Bounded semantic type identities used for error-instance interning.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

mod wire;

/// A nominal type's source-level category, without expanding its fields.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum NominalKind {
    /// Ordinary record identity.
    Record,
    /// Ordinary enum identity, even when its runtime layout is erased.
    Enum,
    /// Fieldless ordinary enum with the existing Comparable capability.
    OrdinalEnum,
    /// Native error-family identity.
    Error,
    /// Distinct scalar identity.
    Distinct,
}

/// One parameter of a concrete callable type argument.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct IdentityParameter {
    /// Canonical source parameter label.
    pub name: String,
    /// Whether the parameter writes through a caller reference.
    pub mutable: bool,
    /// Checked concrete type, or a schema's enclosing parameter.
    pub ty: TypeIdentity,
}

/// Alias-free semantic identity; schemas may reference enclosing parameters.
///
/// Nominal nodes retain declaration identity and arguments, never recursive fields.
/// Values admitted for execution contain no Parameter nodes. Validation bounds the
/// owned tree before comparison, hashing, substitution or serialization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeIdentity {
    /// Procedure-like result.
    Unit,
    /// Boolean scalar.
    Boolean,
    /// Signed integer scalar.
    Integer,
    /// Finite binary64 scalar.
    Real,
    /// Unicode string scalar.
    String,
    /// Structural inclusive integer bounds.
    Subrange {
        /// Inclusive minimum.
        lower: i64,
        /// Inclusive maximum.
        upper: i64,
    },
    /// Named declaration with its complete ordered arguments.
    Nominal {
        /// Declaration category.
        kind: NominalKind,
        /// Canonical defining name, not an imported alias or displayed type.
        declaration: String,
        /// Fully applied arguments, including enclosing parameters when needed.
        arguments: Vec<Self>,
    },
    /// Array element.
    Array(Box<Self>),
    /// Channel element.
    Channel(Box<Self>),
    /// Dictionary key and value.
    Dictionary(Box<Self>, Box<Self>),
    /// Optional element.
    Option(Box<Self>),
    /// Success and error payload.
    Result(Box<Self>, Box<Self>),
    /// Task result, including Unit for a procedure task.
    Task(Box<Self>),
    /// Instantiated function or procedure signature.
    Callable {
        /// Parameters in declaration order.
        parameters: Vec<IdentityParameter>,
        /// None for a procedure.
        result: Option<Box<Self>>,
        /// Whether the final parameter accepts a variadic argument list.
        variadic: bool,
    },
    /// Zero-based enclosing schema parameter; forbidden in executable keys.
    Parameter(u16),
}

/// Canonical defining declaration and complete semantic arguments of one error.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ErrorInstanceKey {
    /// Canonical defining declaration identity, excluding aliases.
    pub declaration: String,
    /// Ordered concrete arguments; symbolic only in constructor schemas.
    #[serde(deserialize_with = "TypeIdentity::deserialize_list")]
    pub arguments: Vec<TypeIdentity>,
}

impl ErrorInstanceKey {
    /// Validate a bounded instance key, allowing only the declared schema parameters.
    ///
    /// # Errors
    /// Returns the violated identity invariant. Pass zero for executable instances.
    pub fn validate(&self, parameters: u16) -> Result<(), &'static str> {
        validate_name(&self.declaration)?;
        TypeIdentity::validate_all(&self.arguments, parameters)
    }

    /// Return the equivalent nominal type expression for a cause or schema field.
    #[must_use]
    pub fn as_type(&self) -> TypeIdentity {
        TypeIdentity::Nominal {
            kind: NominalKind::Error,
            declaration: self.declaration.clone(),
            arguments: self.arguments.clone(),
        }
    }
}

impl TypeIdentity {
    /// Whether a concrete type satisfies the existing Printable capability.
    ///
    /// Aggregate payloads retain the language's shallow Printable rules.
    #[must_use]
    pub fn is_printable(&self) -> bool {
        !matches!(
            self,
            Self::Parameter(_)
                | Self::Callable { .. }
                | Self::Nominal {
                    kind: NominalKind::Distinct,
                    ..
                }
        )
    }

    /// Decode a bounded list used by essential semantic artifact metadata.
    ///
    /// # Errors
    /// Rejects malformed identities and excessive list lengths during deserialization.
    pub fn deserialize_list<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Self>, D::Error> {
        wire::arguments(deserializer)
    }

    /// Recover an exact error-instance key from a checked nominal type.
    #[must_use]
    pub fn error_key(&self) -> Option<ErrorInstanceKey> {
        match self {
            Self::Nominal {
                kind: NominalKind::Error,
                declaration,
                arguments,
            } => Some(ErrorInstanceKey {
                declaration: declaration.clone(),
                arguments: arguments.clone(),
            }),
            _ => None,
        }
    }

    /// Validate semantic type arguments without recursive graph traversal.
    ///
    /// # Errors
    /// Returns an invalid name, bound, parameter or resource invariant.
    pub fn validate_all(types: &[Self], parameters: u16) -> Result<(), &'static str> {
        if types.len() > crate::limits::MAX_LAYOUT_FIELDS {
            return Err("error type identity exceeds its graph limit");
        }
        let mut pending = types.iter().map(|ty| (ty, 0_usize)).collect::<Vec<_>>();
        let mut visited = 0_usize;
        while let Some((ty, depth)) = pending.pop() {
            visited += 1;
            if depth >= 64 || visited > crate::limits::MAX_LAYOUT_FIELDS {
                return Err("error type identity exceeds its graph limit");
            }
            match ty {
                Self::Parameter(index) if *index >= parameters => {
                    return Err("unresolved error type parameter");
                }
                Self::Subrange { lower, upper } if lower > upper => {
                    return Err("reversed error type subrange");
                }
                Self::Nominal {
                    declaration,
                    arguments,
                    ..
                } => {
                    validate_name(declaration)?;
                    pending.extend(arguments.iter().map(|ty| (ty, depth + 1)));
                }
                Self::Array(ty) | Self::Channel(ty) | Self::Option(ty) | Self::Task(ty) => {
                    pending.push((ty, depth + 1))
                }
                Self::Dictionary(left, right) | Self::Result(left, right) => {
                    pending.push((left, depth + 1));
                    pending.push((right, depth + 1));
                }
                Self::Callable {
                    parameters: fields,
                    result,
                    ..
                } => {
                    if fields.len() > crate::limits::MAX_CALL_ARGUMENTS {
                        return Err("error callable identity exceeds argument limit");
                    }
                    for field in fields {
                        validate_name(&field.name)?;
                        pending.push((&field.ty, depth + 1));
                    }
                    if let Some(result) = result {
                        pending.push((result, depth + 1));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty()
        || name.len() > 4096
        || !name.is_ascii()
        || name.bytes().any(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_whitespace() || byte.is_ascii_control()
        })
    {
        Err("error type identity requires a bounded canonical defining name")
    } else {
        Ok(())
    }
}
