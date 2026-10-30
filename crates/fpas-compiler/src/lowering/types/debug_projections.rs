//! Bounded source-type projections beside shared erased operational layouts.

use fpas_ir::{DebugProjection, DebugProjectionKind as Kind, IrType, TypeId};
use fpas_sema::Ty;

use super::TypeTable;
use crate::CompileError;

const MAX_DEPTH: usize = 32;
const MAX_NODES: usize = 4096;
const MAX_NAME_BYTES: usize = 65536;

struct Budget {
    nodes: usize,
    names: usize,
}

impl TypeTable {
    /// Build from the exact semantic declaration, never its formatted display string.
    pub(in crate::lowering) fn debug_projection(
        &mut self,
        declaration: fpas_ir::SourceSpan,
        name: &str,
        operational: TypeId,
    ) -> Result<Option<DebugProjection>, CompileError> {
        let Some(ty) = self
            .debug_declarations
            .get(&(
                declaration.source_id(),
                declaration.offset(),
                name.to_ascii_lowercase(),
            ))
            .cloned()
        else {
            return Ok(None);
        };
        let reference = matches!(self.kind(operational), Some(IrType::Reference(_)));
        let value_type = match self.kind(operational) {
            Some(IrType::Reference(inner)) => *inner,
            _ => operational,
        };
        let mut budget = Budget {
            nodes: MAX_NODES - usize::from(reference),
            names: MAX_NAME_BYTES,
        };
        let span: fpas_lexer::Span = declaration.into();
        let mut projection = self
            .project_semantic(&ty, value_type, usize::from(reference), &mut budget, span)?
            .unwrap_or_else(|| operational_tail(value_type));
        if reference {
            projection = DebugProjection {
                ty: operational,
                type_name: None,
                kind: Kind::Reference(Box::new(projection)),
            };
        }
        Ok(Some(projection))
    }

    fn project_semantic(
        &mut self,
        semantic: &Ty,
        operational: TypeId,
        depth: usize,
        budget: &mut Budget,
        span: fpas_lexer::Span,
    ) -> Result<Option<DebugProjection>, CompileError> {
        if budget.nodes == 0 {
            return Ok(None);
        }
        budget.nodes -= 1;
        if depth >= MAX_DEPTH {
            return Ok(Some(operational_tail(operational)));
        }
        // Stored recursive headers can be incomplete. Recover the same nominal
        // declaration and preserve its exact applied arguments before unfolding.
        let resolved = match semantic {
            Ty::Named(name) => self
                .debug_named_types
                .get(&name.to_ascii_lowercase())
                .cloned(),
            Ty::Record(record) => self
                .debug_named_types
                .get(&record.name.to_ascii_lowercase())
                .and_then(|ty| match ty {
                    Ty::Record(template)
                        if record.fields.is_empty() && !template.fields.is_empty() =>
                    {
                        Some(Ty::Record(if record.type_args.is_empty() {
                            template.clone()
                        } else {
                            std::sync::Arc::new(template.instantiate(record.type_args.clone()))
                        }))
                    }
                    _ => None,
                }),
            Ty::Enum(enumeration) if enumeration.variants.is_empty() => self
                .debug_named_types
                .get(&enumeration.name.to_ascii_lowercase())
                .and_then(|ty| match ty {
                    Ty::Enum(template) if !template.variants.is_empty() => {
                        Some(Ty::Enum(if enumeration.type_args.is_empty() {
                            template.clone()
                        } else {
                            std::sync::Arc::new(template.instantiate(enumeration.type_args.clone()))
                        }))
                    }
                    _ => None,
                }),
            _ => None,
        };
        if let Some(resolved) = resolved {
            // A named alias is not an additional stored path step.
            return Ok(Some(
                self.project_semantic(&resolved, operational, depth + 1, budget, span)?
                    .unwrap_or_else(|| operational_tail(operational)),
            ));
        }
        // Only erased storage needs a new concrete node type. Known operational
        // containers keep their ABI type; their children carry source refinements.
        let ty = if matches!(self.kind(operational), Some(IrType::Dynamic)) {
            self.intern(semantic, span.line, span.column)?
        } else {
            operational
        };
        let mut type_name = self.declared_semantic_scalar_name(semantic);
        let kind = match semantic {
            Ty::ByteBuffer => Kind::Operational,
            Ty::Integer
            | Ty::Real
            | Ty::Boolean
            | Ty::String
            | Ty::Subrange(_)
            | Ty::Distinct(_) => Kind::Scalar,
            Ty::Enum(enumeration) if !enumeration.has_data() => Kind::Scalar,
            Ty::Array(element) => {
                let Some(IrType::Array(element_type)) = self.kind(ty).cloned() else {
                    return Ok(Some(unsupported(ty)));
                };
                let Some(element) =
                    self.project_semantic(element, element_type, depth + 1, budget, span)?
                else {
                    return Ok(Some(operational_tail(operational)));
                };
                Kind::Array(Box::new(element))
            }
            Ty::Record(record) => {
                let Some(IrType::Record(layout)) = self.kind(ty).cloned() else {
                    return Ok(Some(unsupported(ty)));
                };
                if record.fields.len() > budget.nodes {
                    return Ok(Some(operational_tail(operational)));
                }
                let stored = &self.record_layouts[layout.get() as usize].fields;
                if stored.len() != record.fields.len()
                    || stored
                        .iter()
                        .zip(&record.fields)
                        .any(|(a, (name, _))| !a.name.eq_ignore_ascii_case(name))
                {
                    return Ok(Some(unsupported(ty)));
                }
                let stored = stored.iter().map(|field| field.ty).collect::<Vec<_>>();
                let mut fields = Vec::with_capacity(record.fields.len());
                for (index, ((_, field), stored)) in record.fields.iter().zip(stored).enumerate() {
                    // An earlier recursive subtree must not consume the one-node
                    // reservation needed for every later source field.
                    let reserved = record.fields.len() - index - 1;
                    let mut child_budget = Budget {
                        nodes: budget.nodes - reserved,
                        names: budget.names,
                    };
                    let Some(field) =
                        self.project_semantic(field, stored, depth + 1, &mut child_budget, span)?
                    else {
                        return Ok(Some(operational_tail(operational)));
                    };
                    budget.nodes = child_budget.nodes + reserved;
                    budget.names = child_budget.names;
                    fields.push(field);
                }
                Kind::Record(fields)
            }
            _ => Kind::Unsupported,
        };
        if let Some(name) = &type_name {
            if name.len() > 4096 || name.len() > budget.names {
                return Ok(Some(operational_tail(operational)));
            }
            budget.names -= name.len();
        }
        if !matches!(kind, Kind::Scalar) {
            type_name = None;
        }
        Ok(Some(DebugProjection {
            ty,
            type_name,
            kind,
        }))
    }
}

fn unsupported(ty: TypeId) -> DebugProjection {
    DebugProjection {
        ty,
        type_name: None,
        kind: Kind::Unsupported,
    }
}

fn operational_tail(ty: TypeId) -> DebugProjection {
    DebugProjection {
        ty,
        type_name: None,
        kind: Kind::Operational,
    }
}
