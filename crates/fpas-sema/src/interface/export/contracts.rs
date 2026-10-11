//! Export clauses by definition identity without changing callable compatibility.

use fpas_parser::{Decl, EnsuresClause, RecordMethod, RequiresClause, TypeBody, Unit, Visibility};
use fpas_unit::interface::RoutineContract;

pub(super) fn export_contracts(unit: &Unit) -> Vec<RoutineContract> {
    let mut contracts = Vec::new();
    let unit_name = unit.name.parts.join(".");
    for declaration in &unit.declarations {
        if declaration.visibility() != Visibility::Public {
            continue;
        }
        match declaration {
            Decl::Function(function) => append(
                &mut contracts,
                &format!("{unit_name}.{}", function.name),
                &function.requires,
                &function.ensures,
            ),
            Decl::Procedure(procedure) => append(
                &mut contracts,
                &format!("{unit_name}.{}", procedure.name),
                &procedure.requires,
                &procedure.ensures,
            ),
            Decl::TypeDef(definition) => {
                if let TypeBody::Record(record) = &definition.body {
                    for method in &record.methods {
                        let (visibility, name, requires, ensures) = match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => (
                                function.visibility,
                                &function.name,
                                &function.requires,
                                &function.ensures,
                            ),
                            RecordMethod::Procedure(procedure)
                            | RecordMethod::StaticProcedure(procedure) => (
                                procedure.visibility,
                                &procedure.name,
                                &procedure.requires,
                                &procedure.ensures,
                            ),
                        };
                        if visibility == Visibility::Public {
                            append(
                                &mut contracts,
                                &format!("{unit_name}.{}.{name}", definition.name),
                                requires,
                                ensures,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    contracts
}

fn append(
    contracts: &mut Vec<RoutineContract>,
    name: &str,
    clauses: &[RequiresClause],
    ensures: &[EnsuresClause],
) {
    if !clauses.is_empty() || !ensures.is_empty() {
        contracts.push(RoutineContract {
            qualified_name: name.to_owned(),
            requires: clauses.iter().map(|clause| clause.text.clone()).collect(),
            ensures: ensures.iter().map(|clause| clause.text.clone()).collect(),
        });
    }
}
