//! Analysis reuse across isolated request snapshots.

#![allow(clippy::expect_used, reason = "fixture failures carry local context")]

mod support;

use std::sync::Arc;

use fpas_language_service::{CancellationToken, LanguageService};
use support::{TempDirectory, write_program_project};

#[test]
fn query_results_are_reused_by_later_requests_and_the_parent() {
    let temp = TempDirectory::new("query-cache-reuse");
    let (manifest, main, _) = write_program_project(&temp);
    let mut service = LanguageService::load(&manifest);
    let first = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("first request");
    let next = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("next request");
    let parent = service.analyze_document(&main).expect("parent query");
    assert!(Arc::ptr_eq(&first, &next));
    assert!(Arc::ptr_eq(&first, &parent));
}

#[test]
fn concurrent_requests_publish_one_reusable_analysis() {
    let temp = TempDirectory::new("query-cache-concurrent");
    let (manifest, main, _) = write_program_project(&temp);
    let service = LanguageService::load(&manifest);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let requests = (0..2)
        .map(|_| {
            let mut query = service.fork_for_queries();
            let main = main.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                query.analyze_document(&main).expect("concurrent query")
            })
        })
        .collect::<Vec<_>>();
    let analyses = requests
        .into_iter()
        .map(|request| request.join().expect("query thread"))
        .collect::<Vec<_>>();
    assert!(Arc::ptr_eq(&analyses[0], &analyses[1]));
}

#[test]
fn older_request_completion_cannot_replace_new_editor_contents() {
    let temp = TempDirectory::new("query-cache-overlay");
    let (manifest, main, _) = write_program_project(&temp);
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&main, 1, "program Old; begin end.")
        .expect("old buffer");
    let mut old_request = service.fork_for_queries();
    service
        .documents_mut()
        .apply_full_text(&main, 2, "program New; begin end.")
        .expect("new buffer");
    let latest = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("new request");
    let older = old_request
        .analyze_document(&main)
        .expect("late old request");
    let next = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("next request");
    assert_ne!(older.snapshot().revision(), latest.snapshot().revision());
    assert_eq!(next.snapshot().revision(), latest.snapshot().revision());
    assert!(Arc::ptr_eq(
        older.snapshot(),
        &old_request.snapshot(&main).expect("isolated old buffer")
    ));
}

#[test]
fn late_request_from_an_old_project_cannot_supply_the_refreshed_project_analysis() {
    let temp = TempDirectory::new("query-cache-context");
    let (manifest, main, _) = write_program_project(&temp);
    let mut service = LanguageService::load(temp.path());
    let mut old_request = service.fork_for_queries();
    let source = std::fs::read_to_string(&manifest).expect("manifest");
    std::fs::write(
        &manifest,
        source.replace("name = \"demo\"", "name = \"renamed\""),
    )
    .expect("updated context");
    service
        .refresh_paths(&[manifest], &CancellationToken::new())
        .expect("refresh");
    let older = old_request
        .analyze_document(&main)
        .expect("late old project request");
    let latest = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("new project request");
    assert_eq!(older.snapshot().revision(), latest.snapshot().revision());
    assert!(!Arc::ptr_eq(&older, &latest));
    let repeated = service
        .fork_for_queries()
        .analyze_document(&main)
        .expect("cached new project request");
    assert!(Arc::ptr_eq(&latest, &repeated));
}
