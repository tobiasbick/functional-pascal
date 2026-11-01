//! Explicit framing keeps plan checksums independent of JSON order and escaping.

use super::*;
use crate::analysis::identity::encoding::CanonicalHasher;
use crate::{QueryPolicy, ScopeKind, SourceSelector, TargetReference};

impl RefactorPlan {
    /// Recomputes the canonical content checksum without trusting the stored digest.
    pub fn content_digest(&self) -> String {
        let mut hash = CanonicalHasher::new("fpas-refactor-plan-v1");
        hash.count(self.plan_version as usize);
        hash.text(&self.analysis_identity);
        context(&mut hash, &self.context);
        recipe(&mut hash, &self.recipe);
        hash.text(&self.before.digest());
        hash.text(&self.after.digest());
        hash.count(self.edits.len());
        for edit in &self.edits {
            hash.text(&edit.path);
            hash.count(edit.start);
            hash.count(edit.length);
            hash.text(&edit.old_text);
            hash.text(&edit.new_text);
        }
        hash.finish()
    }
}

fn context(hash: &mut CanonicalHasher, context: &ScopeContext) {
    hash.text(&context.input);
    hash.text(match context.kind {
        ScopeKind::Folder => "folder",
        ScopeKind::Loose => "loose",
        ScopeKind::Project => "project",
        ScopeKind::Workspace => "workspace",
    });
    hash.text(&context.root);
    optional(hash, context.standard_library.as_deref());
    hash.count(context.projects.len());
    for project in &context.projects {
        hash.text(project);
    }
    hash.text(match context.policy {
        QueryPolicy::CheckedContext => "checked_context",
        QueryPolicy::EditorCatalog => "editor_catalog",
    });
}

fn recipe(hash: &mut CanonicalHasher, recipe: &RefactorRecipe) {
    match recipe {
        RefactorRecipe::AddParameter {
            selector: target,
            name,
            r#type,
            mode,
            argument,
        } => {
            hash.text("add-parameter");
            selector(hash, target);
            hash.text(name);
            hash.text(r#type);
            hash.text(match mode {
                crate::ParameterMode::Value => "value",
                crate::ParameterMode::Var => "var",
            });
            hash.text(argument);
        }
        RefactorRecipe::Rename {
            selector: target,
            name,
        } => {
            hash.text("rename");
            selector(hash, target);
            hash.text(name);
        }
        RefactorRecipe::ExtractVariable {
            path,
            start,
            end,
            name,
        } => {
            hash.text("extract-variable");
            hash.text(path);
            hash.count(*start);
            hash.count(*end);
            optional(hash, name.as_deref());
        }
        RefactorRecipe::OrganizeImports { path } => {
            hash.text("organize-imports");
            hash.text(path);
        }
    }
}

fn selector(hash: &mut CanonicalHasher, selector: &SourceSelector) {
    match selector {
        SourceSelector::Position { path, offset } => {
            hash.text("position");
            hash.text(path);
            hash.count(*offset);
        }
        SourceSelector::Symbol { name } => {
            hash.text("symbol");
            hash.text(name);
        }
        SourceSelector::Reference { target_ref } => {
            hash.text("reference");
            reference(hash, target_ref);
        }
    }
}

fn reference(hash: &mut CanonicalHasher, reference: &TargetReference) {
    hash.count(reference.version as usize);
    context(hash, &reference.context);
    hash.text(&reference.snapshot_id);
    hash.text(&reference.analysis_identity);
    optional(hash, reference.owner.as_deref());
    hash.text(&format!("{:?}", reference.kind));
    hash.text(&reference.qualified_name);
    hash.text(&reference.file);
    hash.count(reference.start);
    hash.count(reference.length);
    hash.text(&reference.file_sha256);
}

fn optional(hash: &mut CanonicalHasher, value: Option<&str>) {
    hash.count(usize::from(value.is_some()));
    if let Some(value) = value {
        hash.text(value);
    }
}
