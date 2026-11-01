//! Shared physical sources may merge only when their checked interpretations agree.

use crate::DocumentAnalysis;
use fpas_sema::AnalysisMetadata;
use std::collections::{BTreeMap, HashMap};

pub(super) fn same(left: &DocumentAnalysis, right: &DocumentAnalysis) -> bool {
    if left.snapshot().source() != right.snapshot().source() || left.symbols() != right.symbols() {
        return false;
    }
    match (left.semantic(), right.semantic()) {
        (Some(left), Some(right)) => {
            let left = left.metadata();
            let right = right.metadata();
            left.import_aliases == right.import_aliases
                && offsets(&left.declaration_types) == offsets(&right.declaration_types)
                && expressions(left) == expressions(right)
                && bindings(left) == bindings(right)
                && annotations(left) == annotations(right)
        }
        (None, None) => true,
        _ => false,
    }
}

fn offsets<T>(values: &HashMap<(u32, usize), T>) -> BTreeMap<usize, &T> {
    values
        .iter()
        .map(|((_, offset), value)| (*offset, value))
        .collect()
}

fn expressions(metadata: &AnalysisMetadata) -> BTreeMap<(usize, usize), &fpas_sema::Ty> {
    metadata
        .source_expr_types
        .iter()
        .map(|((_, start, length), ty)| ((*start, *length), ty))
        .collect()
}

type Binding<'a> = (
    usize,
    Option<usize>,
    &'a str,
    fpas_sema::SourceBindingKind,
    &'a fpas_sema::Ty,
);
fn bindings(metadata: &AnalysisMetadata) -> BTreeMap<usize, Binding<'_>> {
    metadata
        .source_bindings
        .iter()
        .map(|((_, offset), binding)| {
            (
                *offset,
                (
                    binding.parts,
                    binding.declaration.map(|(_, offset)| offset),
                    binding.name.as_str(),
                    binding.kind,
                    &binding.ty,
                ),
            )
        })
        .collect()
}

type Annotation<'a> = (Option<usize>, Option<&'a str>, &'a fpas_sema::Ty);
fn annotations(metadata: &AnalysisMetadata) -> BTreeMap<usize, Annotation<'_>> {
    metadata
        .type_references
        .iter()
        .map(|((_, offset), binding)| {
            (
                *offset,
                (
                    binding.declaration.map(|span| span.offset),
                    binding.qualified_name.as_deref(),
                    &binding.ty,
                ),
            )
        })
        .collect()
}
