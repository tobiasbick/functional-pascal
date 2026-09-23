//! Request-level disk I/O and freshness regression for closed project sources.
#![allow(clippy::expect_used, reason = "project I/O measurement fixture")]
use crate::{CancellationToken, LanguageService};

#[test]
fn diagnostics_read_closed_units_once_and_preserve_freshness() {
    let root = std::env::temp_dir().join(format!("fpas-diagnostic-reads-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("fixture directory");
    let manifest = root.join("app.fpasprj");
    std::fs::write(&manifest, "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n").expect("manifest");
    let main = root.join("main.fpas");
    let source = "program App; begin end.";
    std::fs::write(&main, source).expect("main source");
    let mut expected_bytes = 0;
    for index in 0..40 {
        let source = format!("unit U{index};\n");
        expected_bytes += source.len();
        std::fs::write(root.join(format!("u{index}.fpas")), source).expect("unit");
    }
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&main, 1, source)
        .expect("editor buffer");
    let first = service
        .analyze_document_diagnostics(&main)
        .expect("warm analysis");
    assert!(first.failure().is_none());
    service.documents().take_disk_reads();
    for _ in 0..3 {
        let result = service
            .analyze_document_diagnostics(&main)
            .expect("diagnostics");
        assert!(result.failure().is_none());
        assert!(std::sync::Arc::ptr_eq(first.document(), result.document()));
        let reads = service.documents().take_disk_reads();
        assert_eq!(reads, (40, expected_bytes));
        println!("diagnostic request: reads={}, bytes={}", reads.0, reads.1);
    }
    // Hover/analysis requests and concurrent forks retain the same freshness boundary.
    std::thread::scope(|scope| {
        let mut left = service.fork_for_queries();
        let mut right = service.fork_for_queries();
        let main = &main;
        let left = scope.spawn(move || left.analyze_document(main));
        let right = scope.spawn(move || right.analyze_document_diagnostics(main));
        assert!(left.join().expect("query thread").is_ok());
        assert!(
            right
                .join()
                .expect("query thread")
                .expect("diagnostic query")
                .failure()
                .is_none()
        );
    });
    assert_eq!(
        service.documents().take_disk_reads(),
        (80, expected_bytes * 2)
    );
    let changed = root.join("u0.fpas");
    std::fs::write(&changed, "unit U0; // changed without watcher\n").expect("disk mutation");
    let fresh = service
        .analyze_document_diagnostics(&main)
        .expect("fresh query");
    assert!(fresh.failure().is_none());
    assert!(!std::sync::Arc::ptr_eq(first.document(), fresh.document()));
    service.documents().take_disk_reads();
    service
        .refresh_paths(&[changed], &CancellationToken::new())
        .expect("watcher invalidation");
    assert!(
        service
            .analyze_document_diagnostics(&main)
            .expect("refreshed query")
            .failure()
            .is_none()
    );
    assert_eq!(service.documents().take_disk_reads().0, 40);
    std::fs::write(&main, "not valid Pascal").expect("disk under editor overlay");
    assert!(
        service
            .analyze_document_diagnostics(&main)
            .expect("overlay query")
            .document()
            .diagnostics()
            .is_empty()
    );
    std::fs::remove_dir_all(root).expect("cleanup fixture");
}
