//! Source-exclusion filesystem work must grow linearly with the source set.
use super::*;

#[test]
fn exclusion_canonicalizations_scale_linearly() {
    let root = std::env::temp_dir().join(format!("fpas-exclusion-count-{}", std::process::id()));
    fs::create_dir_all(&root).expect("fixture directory");
    for count in [100, 200] {
        for index in 0..count {
            fs::write(root.join(format!("unit{index:03}.fpas")), "unit Example;")
                .expect("fixture unit");
        }
        let exclude = (0..count / 2)
            .map(|index| format!("unit{index:03}.fpas"))
            .collect::<Vec<_>>();
        CANONICALIZATIONS.with(|calls| calls.set(0));
        let (files, _) =
            resolve_source_files(&["*.fpas".into()], &exclude, &root).expect("resolve sources");
        let calls = CANONICALIZATIONS.with(std::cell::Cell::get);
        println!(
            "sources={count}, exclusions={}, canonicalizations={calls}",
            count / 2
        );
        assert_eq!(files.len(), count / 2);
        assert_eq!(calls, count * 2 + count / 2);
    }
    fs::remove_dir_all(root).expect("remove fixture");
}
