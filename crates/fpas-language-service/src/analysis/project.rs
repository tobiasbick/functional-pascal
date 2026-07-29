//! In-memory semantic analysis for one loaded project.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_parser::{CompilationUnit, QualifiedId};
use fpas_project::{
    ProjectKind, build_unit_graph_for_program_from_parsed_sources,
    build_unit_graph_from_parsed_sources, resolve_library_units, resolve_program_units,
};
use fpas_unit::interface::UnitInterface;

use crate::document::normalized_path;
use crate::workspace::ProjectContext;
use crate::{DocumentSnapshot, LanguageServiceError};

use super::cache::AnalysisSet;
use super::{DocumentAnalysis, semantic_document};

pub(super) fn analyze_project(
    project: &ProjectContext,
    snapshots: &[Arc<DocumentSnapshot>],
) -> Result<AnalysisSet, LanguageServiceError> {
    let snapshots_by_path = snapshots
        .iter()
        .map(|snapshot| (snapshot.path().to_path_buf(), Arc::clone(snapshot)))
        .collect::<HashMap<_, _>>();
    let parsed_units = snapshots
        .iter()
        .filter(|snapshot| !snapshot.has_parse_errors())
        .filter_map(|snapshot| match snapshot.compilation_unit() {
            CompilationUnit::Unit(unit) => Some((snapshot.path().to_path_buf(), unit.clone())),
            CompilationUnit::Program(_) => None,
        })
        .collect::<Vec<_>>();

    let graph = if let Some(main) = project.main() {
        build_unit_graph_for_program_from_parsed_sources(
            main,
            parsed_units,
            &project.loaded().link_meta,
        )
    } else {
        build_unit_graph_from_parsed_sources(parsed_units, &project.loaded().link_meta)
    }
    .map_err(|message| LanguageServiceError::analysis(project.manifest_path(), message))?;

    let unit_order = resolve_library_units(&graph)
        .map_err(|message| LanguageServiceError::analysis(project.manifest_path(), message))?;
    let mut analyses = HashMap::<PathBuf, Arc<DocumentAnalysis>>::new();
    let mut interfaces = HashMap::<String, UnitInterface>::new();
    let mut supporting_interfaces = Vec::<UnitInterface>::new();

    for unit_name in unit_order.order() {
        let node = graph.get(unit_name).ok_or_else(|| {
            LanguageServiceError::analysis(
                project.manifest_path(),
                format!("Resolved unit `{unit_name}` disappeared from the project graph."),
            )
        })?;
        let path = normalized_path(node.path());
        let snapshot = snapshots_by_path.get(&path).ok_or_else(|| {
            LanguageServiceError::analysis(
                &path,
                "The parsed project source has no immutable document snapshot.",
            )
        })?;
        let CompilationUnit::Unit(unit) = snapshot.compilation_unit() else {
            return Err(LanguageServiceError::analysis(
                &path,
                "A project unit snapshot no longer declares a unit.",
            ));
        };
        let direct = direct_interfaces(&unit.uses, &interfaces);
        let unit_analysis =
            fpas_sema::analyze_unit_with_interface_support(unit, &direct, &supporting_interfaces)
                .map_err(|error| LanguageServiceError::analysis(&path, error.to_string()))?;
        let interface = unit_analysis.interface.clone();
        let analysis = semantic_document(Arc::clone(snapshot), unit_analysis.metadata);
        analyses.insert(path, Arc::new(analysis));
        if let Some(interface) = interface {
            interfaces.insert(unit_name.clone(), interface.clone());
            supporting_interfaces.push(interface);
        }
    }

    for snapshot in snapshots {
        if snapshot.has_parse_errors() {
            analyses.insert(
                snapshot.path().to_path_buf(),
                Arc::new(DocumentAnalysis::syntax_only(Arc::clone(snapshot))),
            );
            continue;
        }
        let CompilationUnit::Program(program) = snapshot.compilation_unit() else {
            continue;
        };
        if project.kind() == ProjectKind::Library {
            return Err(LanguageServiceError::analysis(
                snapshot.path(),
                "A library project source declares a program.",
            ));
        }
        resolve_program_units(&graph, &program.uses)
            .map_err(|message| LanguageServiceError::analysis(snapshot.path(), message))?;
        let direct = direct_interfaces(&program.uses, &interfaces);
        let metadata = fpas_sema::analyze_program_with_interface_support(
            program,
            &direct,
            &supporting_interfaces,
        )
        .map_err(|error| LanguageServiceError::analysis(snapshot.path(), error.to_string()))?;
        analyses.insert(
            snapshot.path().to_path_buf(),
            Arc::new(semantic_document(Arc::clone(snapshot), metadata)),
        );
    }

    Ok(AnalysisSet::new(analyses))
}

fn direct_interfaces(
    uses: &[QualifiedId],
    interfaces: &HashMap<String, UnitInterface>,
) -> Vec<UnitInterface> {
    uses.iter()
        .filter_map(|used| interfaces.get(&used.parts.join(".").to_ascii_lowercase()))
        .cloned()
        .collect()
}

pub(super) fn project_identity(project: &ProjectContext) -> &Path {
    project.manifest_path()
}
