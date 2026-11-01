//! Compare compiler bindings at original coordinates and inside newly written arguments.

use super::{edits::Insertions, site};
use crate::{CancellationToken, LanguageService, RefactorBlocker, ScopeContext, SourceQueryError};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Position {
    Original(usize),
    Argument(usize, usize, usize),
    Added(usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    context: String,
    path: PathBuf,
    position: Position,
    category: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fact {
    local: Option<Position>,
    target: Option<(String, Position)>,
    meaning: String,
}

pub(super) struct Facts {
    values: BTreeMap<Key, Fact>,
    bytes: usize,
}

impl Facts {
    pub(super) fn collect(
        service: &mut LanguageService,
        scope: &ScopeContext,
        insertions: Option<&Insertions>,
        cancellation: &CancellationToken,
    ) -> Result<Self, SourceQueryError> {
        let mut facts = Self {
            values: BTreeMap::new(),
            bytes: 0,
        };
        let discovery = crate::navigation::checked_source_context(service, cancellation)?;
        if !discovery.reasons.is_empty() {
            return Err(SourceQueryError {
                code: "analysis_incomplete",
                message: "Bindings require complete consumer analysis.".into(),
                diagnostics: discovery.diagnostics,
            });
        }
        let batches = if discovery.batches.is_empty() {
            vec![(
                scope.input.clone(),
                service.source_analyses(Path::new(&scope.input))?,
            )]
        } else {
            discovery
                .batches
                .into_iter()
                .map(|b| {
                    Ok((
                        crate::identity_path(b.project.manifest_path()).map_err(|e| {
                            SourceQueryError::new("unsupported_path_identity", e.to_string())
                        })?,
                        b.documents,
                    ))
                })
                .collect::<Result<Vec<_>, SourceQueryError>>()?
        };
        for (context, documents) in batches {
            for document in documents {
                cancellation.check()?;
                let path = document.snapshot().path();
                if service.is_editor_api_source(path) {
                    continue;
                }
                let metadata = document
                    .semantic()
                    .ok_or_else(|| {
                        SourceQueryError::new(
                            "analysis_incomplete",
                            "Checked bindings are unavailable.",
                        )
                    })?
                    .metadata();
                for ((_, offset), binding) in &metadata.source_bindings {
                    let Some(position) = use_position(insertions, path, *offset) else {
                        continue;
                    };
                    facts.insert(
                        Key {
                            context: context.clone(),
                            path: path.into(),
                            position,
                            category: "binding".into(),
                        },
                        Fact {
                            local: binding
                                .declaration
                                .map(|(_, start)| declaration_position(insertions, path, start)),
                            target: None,
                            meaning: format!(
                                "{:?}:{}:{}",
                                binding.kind,
                                binding.parts,
                                binding.name.to_ascii_lowercase()
                            ),
                        },
                    )?;
                }
                for ((_, offset), binding) in &metadata.type_references {
                    let Some(position) = use_position(insertions, path, *offset) else {
                        continue;
                    };
                    facts.insert(
                        Key {
                            context: context.clone(),
                            path: path.into(),
                            position,
                            category: "type".into(),
                        },
                        Fact {
                            local: binding
                                .declaration
                                .map(|span| declaration_position(insertions, path, span.offset)),
                            target: None,
                            meaning: binding
                                .qualified_name
                                .as_ref()
                                .map(|s| s.to_ascii_lowercase())
                                .unwrap_or_else(|| binding.ty.to_string()),
                        },
                    )?;
                }
                for ((_, offset), release) in &metadata.resource_releases {
                    let Some(position) = use_position(insertions, path, *offset) else {
                        continue;
                    };
                    facts.insert(
                        Key {
                            context: context.clone(),
                            path: path.into(),
                            position,
                            category: "release".into(),
                        },
                        Fact {
                            local: None,
                            target: None,
                            meaning: release.name.to_ascii_lowercase(),
                        },
                    )?;
                }
            }
        }
        let coverage = crate::navigation::visit_call_inventory(
            service,
            scope,
            None,
            cancellation,
            &mut |relation| {
                let path = relation.snapshot.path();
                let Some(position) = use_position(insertions, path, relation.span.offset()) else {
                    return Ok(());
                };
                let target = relation.target.as_ref().and_then(|t| {
                    t.identity
                        .file
                        .as_ref()
                        .zip(t.identity.start)
                        .map(|(file, start)| {
                            // Identity paths and snapshot paths have different Windows prefix spelling.
                            let path = t.snapshot.as_ref().map_or(Path::new(file), |s| s.path());
                            (file.clone(), declaration_position(insertions, path, start))
                        })
                });
                let meaning = format!(
                    "{:?}:{:?}:{}:{}",
                    relation.invocation,
                    relation.target.as_ref().map(|t| t.identity.kind),
                    relation.implicit_receiver,
                    if target.is_none() {
                        relation
                            .target
                            .as_ref()
                            .map_or("", |t| t.identity.qualified_name.as_str())
                    } else {
                        ""
                    }
                );
                facts.insert(
                    Key {
                        context: relation.context,
                        path: path.into(),
                        position,
                        category: format!("call:{:?}", relation.kind),
                    },
                    Fact {
                        local: None,
                        target,
                        meaning,
                    },
                )
            },
            &mut |_| {},
        )?;
        if !coverage.reasons.is_empty() {
            return Err(SourceQueryError {
                code: "analysis_incomplete",
                message: "Call-target comparison requires complete consumer analysis.".into(),
                diagnostics: coverage.diagnostics,
            });
        }
        Ok(facts)
    }

    fn insert(&mut self, key: Key, fact: Fact) -> Result<(), SourceQueryError> {
        self.bytes = self.bytes.saturating_add(
            256 + key.context.len()
                + key.path.as_os_str().len() * 2
                + fact.meaning.len()
                + fact.target.as_ref().map_or(0, |(file, _)| file.len()),
        );
        if self.bytes > 32 * 1024 * 1024 {
            return Err(SourceQueryError::new(
                "plan_limit",
                "Checked binding comparisons exceed their 32 MiB fact budget.",
            ));
        }
        if let Some(previous) = self.values.insert(key, fact.clone())
            && previous != fact
        {
            return Err(SourceQueryError::new(
                "analysis_incomplete",
                "Several checked meanings occupy one physical source site.",
            ));
        }
        Ok(())
    }

    pub(super) fn compare_original(
        &self,
        after: &Self,
        blockers: &mut Vec<RefactorBlocker>,
    ) -> Result<(), SourceQueryError> {
        self.compare(after, false, blockers)
    }

    pub(super) fn compare_arguments(
        &self,
        after: &Self,
        blockers: &mut Vec<RefactorBlocker>,
    ) -> Result<(), SourceQueryError> {
        self.compare(after, true, blockers)
    }

    fn compare(
        &self,
        after: &Self,
        arguments: bool,
        blockers: &mut Vec<RefactorBlocker>,
    ) -> Result<(), SourceQueryError> {
        let selected = |key: &&Key| matches!(key.position, Position::Argument(..)) == arguments;
        for key in self
            .values
            .keys()
            .filter(selected)
            .chain(after.values.keys().filter(selected))
        {
            if self.values.get(key) != after.values.get(key) {
                let offset = match key.position {
                    Position::Original(offset) | Position::Argument(offset, ..) => Some(offset),
                    _ => None,
                };
                blockers.push(site(
                    "binding_changed",
                    "The new parameter would change an existing binding or capture a new argument.",
                    &key.path,
                    offset,
                    None,
                )?);
            }
        }
        if blockers.is_empty() {
            Ok(())
        } else {
            Err(SourceQueryError::new(
                "binding_changed",
                "Parameter insertion changes checked bindings or call targets.",
            ))
        }
    }

    pub(super) fn reject_self_argument(
        &self,
        file: &str,
        start: usize,
        qualified: &str,
        blockers: &mut Vec<RefactorBlocker>,
    ) -> Result<(), SourceQueryError> {
        for (key, fact) in &self.values {
            if !matches!(key.position, Position::Argument(..)) {
                continue;
            }
            let local = crate::identity_path(&key.path).is_ok_and(|path| path == file)
                && fact.local == Some(Position::Original(start));
            let target = fact.target.as_ref().is_some_and(|(path, position)| {
                path == file && *position == Position::Original(start)
            });
            let imported = key.category == "binding"
                && fact.meaning.starts_with("Routine:")
                && fact
                    .meaning
                    .ends_with(&format!(":{}", qualified.to_ascii_lowercase()));
            if local || target || imported {
                let offset = match key.position {
                    Position::Argument(offset, ..) => Some(offset),
                    _ => None,
                };
                blockers.push(site("recursive_argument", "The inserted argument cannot refer to the routine whose signature is changing.", &key.path, offset, Some(0))?);
            }
        }
        if blockers.is_empty() {
            Ok(())
        } else {
            Err(SourceQueryError::new(
                "recursive_argument",
                "Inserted arguments refer to the changed routine.",
            ))
        }
    }
}

fn use_position(edits: Option<&Insertions>, path: &Path, offset: usize) -> Option<Position> {
    let Some(edits) = edits else {
        return Some(Position::Original(offset));
    };
    if let Some(original) = edits.original(path, offset) {
        return Some(Position::Original(original));
    }
    let (index, relative) = edits.inserted(path, offset)?;
    let item = &edits.items[index];
    let range = item
        .argument
        .as_ref()
        .filter(|range| range.contains(&relative))?;
    Some(Position::Argument(
        item.edit.range.offset(),
        index,
        relative - range.start,
    ))
}

fn declaration_position(edits: Option<&Insertions>, path: &Path, offset: usize) -> Position {
    if let Some(position) = use_position(edits, path, offset) {
        return position;
    }
    if let Some((index, relative)) = edits.and_then(|edits| edits.inserted(path, offset)) {
        return Position::Added(index, relative);
    }
    Position::Original(offset)
}
