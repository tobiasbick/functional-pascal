//! Complete semantic batches preserve distinct project interpretations of shared files.

mod facts;

use crate::{
    CancellationToken, DocumentAnalysis, LanguageService, ProjectContext, SourceQueryError,
};
use fpas_diagnostics::FileDiagnostic;
use fpas_parser::{CompilationUnit, Import};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

pub(crate) struct Batch {
    pub project: ProjectContext,
    pub documents: Vec<Arc<DocumentAnalysis>>,
    pub imported: BTreeSet<String>,
}
pub(crate) struct Discovery {
    pub batches: Vec<Batch>,
    pub diagnostics: Vec<FileDiagnostic>,
    pub reasons: BTreeSet<&'static str>,
}

pub(crate) fn checked(
    service: &mut LanguageService,
    cancellation: &CancellationToken,
) -> Result<Discovery, SourceQueryError> {
    let mut result = Discovery {
        batches: Vec::new(),
        diagnostics: Vec::new(),
        reasons: BTreeSet::new(),
    };
    let projects = service.workspace().projects().to_vec();
    for project in projects {
        cancellation.check()?;
        for warning in &project.loaded().warnings {
            if warning.diagnostic.is_error() {
                result.reasons.insert("analysis_errors");
            }
            if warning.diagnostic.code == fpas_diagnostics::codes::PROJECT_PROGRAM_SOURCE_SKIPPED {
                result.reasons.insert("ignored_source");
            }
            result.diagnostics.push(warning.clone());
        }
        let mut documents = match service.project_source_analyses(&project) {
            Ok(documents) => documents,
            Err(error) => {
                result.reasons.insert("analysis_unavailable");
                result.diagnostics.extend(error.diagnostics());
                service.project_syntax_analyses(&project)?
            }
        };
        documents.sort_by(|a, b| a.snapshot().path().cmp(b.snapshot().path()));
        let mut imported = BTreeSet::new();
        let mut relevant = project
            .all_source_paths()
            .into_iter()
            .collect::<BTreeSet<PathBuf>>();
        let units = documents
            .iter()
            .filter_map(|a| match a.snapshot().compilation_unit() {
                CompilationUnit::Unit(unit) => {
                    Some((unit.name.parts.join(".").to_ascii_lowercase(), a))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let mut pending = documents
            .iter()
            .filter(|a| project.owns_source(a.snapshot().path()))
            .collect::<Vec<_>>();
        while let Some(analysis) = pending.pop() {
            cancellation.check()?;
            for import in imports(analysis.snapshot().compilation_unit()) {
                let unit = import.unit.parts.join(".").to_ascii_lowercase();
                if imported.insert(unit.clone())
                    && let Some(dependency) = units.get(&unit)
                {
                    relevant.insert(dependency.snapshot().path().to_path_buf());
                    pending.push(dependency);
                }
            }
        }
        for analysis in &documents {
            if !relevant.contains(analysis.snapshot().path()) {
                continue;
            }
            if analysis.semantic().is_none() {
                result.reasons.insert("analysis_unavailable");
            }
            for diagnostic in analysis.diagnostics() {
                if diagnostic.is_error() {
                    result.reasons.insert("analysis_errors");
                }
                result.diagnostics.push(
                    FileDiagnostic::new(
                        diagnostic.clone(),
                        Some(analysis.snapshot().path().into()),
                    )
                    .with_source(analysis.snapshot().source()),
                );
            }
        }
        result.batches.push(Batch {
            project,
            documents,
            imported,
        });
    }
    result.diagnostics.sort_by(|a, b| {
        a.path.cmp(&b.path).then_with(|| {
            a.diagnostic
                .span
                .map(|s| s.offset())
                .cmp(&b.diagnostic.span.map(|s| s.offset()))
        })
    });
    result.diagnostics.dedup();
    let mut interpretations = BTreeMap::new();
    for batch in &result.batches {
        for analysis in &batch.documents {
            if !batch.project.contains_source(analysis.snapshot().path())
                && !batch
                    .imported
                    .contains(&analysis.symbols().owner().to_ascii_lowercase())
            {
                continue;
            }
            let imports = imports(analysis.snapshot().compilation_unit())
                .iter()
                .map(|import| {
                    let name = import.unit.parts.join(".").to_ascii_lowercase();
                    let path = batch
                        .documents
                        .iter()
                        .find(|a| a.symbols().owner().eq_ignore_ascii_case(&name))
                        .map(|a| a.snapshot().path());
                    (name, path)
                })
                .collect::<BTreeMap<_, _>>();
            if let Some((previous, previous_imports)) =
                interpretations.insert(analysis.snapshot().path(), (analysis, imports.clone()))
                && (!facts::same(previous, analysis) || previous_imports != imports)
            {
                result.reasons.insert("ambiguous_context");
            }
        }
    }
    Ok(result)
}

pub(super) fn imports(unit: &CompilationUnit) -> &[Import] {
    match unit {
        CompilationUnit::Unit(u) => &u.uses,
        CompilationUnit::Program(p) => &p.uses,
    }
}
