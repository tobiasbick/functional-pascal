//! Persistent constant resolution and semantic bit-identity merging.

use std::collections::HashMap;

use fpas_code::{Constant, ConstantId, FunctionId};
use fpas_unit::object::{DefinitionTarget, ObjectConstant, RelocatableObject, SymbolKind};

use crate::LinkError;
use crate::plan::{LinkIds, SymbolTable};

use super::strings::StringInterner;

pub(super) struct ConstantIds {
    pub maps: Vec<Vec<ConstantId>>,
    pub values: Vec<Constant>,
}

pub(super) fn merge(
    objects: &[&RelocatableObject],
    symbols: &SymbolTable,
    ids: &LinkIds,
    strings: &mut StringInterner,
) -> Result<ConstantIds, LinkError> {
    let mut maps = objects
        .iter()
        .map(|object| Vec::with_capacity(object.constants.len()))
        .collect::<Vec<_>>();
    let mut values = Vec::new();
    let mut indices = HashMap::new();
    for (object_index, object) in objects.iter().enumerate() {
        for constant in &object.constants {
            let value = match constant {
                ObjectConstant::ErrorConstructor(request) => {
                    let name = fpas_unit::errors::constructor_symbol(request)
                        .map_err(crate::errors::invalid)?;
                    let resolved = symbols
                        .resolve_name(objects, &name, SymbolKind::Function)
                        .ok_or_else(|| {
                            crate::errors::invalid("unresolved error constructor factory")
                        })?;
                    let DefinitionTarget::Function(local) = resolved.target else {
                        return Err(crate::errors::invalid(
                            "error constructor is not a function",
                        ));
                    };
                    Constant::Function {
                        function: function_id(ids, resolved.object, local)?,
                        task_bound: false,
                    }
                }
                ObjectConstant::Integer(value) => Constant::Integer(*value),
                ObjectConstant::Real(bits) => Constant::Real(*bits),
                ObjectConstant::Boolean(value) => Constant::Boolean(*value),
                ObjectConstant::String(value) => Constant::String(strings.intern(value)?),
                ObjectConstant::Unit => Constant::Unit,
                ObjectConstant::Function {
                    function,
                    task_bound,
                } => {
                    let resolved =
                        symbols.resolve(object_index, *function, SymbolKind::Function)?;
                    let DefinitionTarget::Function(local) = resolved.target else {
                        return Err(LinkError::Overflow("constant function target"));
                    };
                    Constant::Function {
                        function: function_id(ids, resolved.object, local)?,
                        task_bound: *task_bound,
                    }
                }
            };
            let id = if let Some(id) = indices.get(&value) {
                *id
            } else {
                let id = ConstantId::try_from_index(values.len())
                    .map_err(|_| LinkError::Overflow("constant IDs"))?;
                values.push(value);
                indices.insert(value, id);
                id
            };
            maps[object_index].push(id);
        }
    }
    Ok(ConstantIds { maps, values })
}

fn function_id(ids: &LinkIds, object: usize, local: u32) -> Result<FunctionId, LinkError> {
    ids.functions
        .maps
        .get(object)
        .and_then(|map| map.get(local as usize))
        .and_then(|id| *id)
        .ok_or(LinkError::Overflow("function reference"))
}
