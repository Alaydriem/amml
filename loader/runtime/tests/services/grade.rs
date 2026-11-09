use std::sync::OnceLock;
use std::sync::atomic::Ordering;

use amml_data::{EntryKind, RecipeLoader};
use amml_runtime::{CacheFile, Grade, GradeReport, LeviLaminaSource};

use crate::support::{BASE, Fixtures};

static PROBE_RVA: OnceLock<u32> = OnceLock::new();

// LeviLamina agrees on `probe_run`, is off by one on `probe_global` and knows nothing else
fn lookup(name: &str) -> Option<u64> {
    let rva = u64::from(*PROBE_RVA.get()?);
    match name {
        "probe_run" => Some(BASE + rva),
        "probe_global" => Some(BASE + rva + 1),
        _ => None,
    }
}

#[test]
fn grade_derives_fresh_and_writes_no_cache() {
    let root = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(root.path());
    PROBE_RVA.set(probe.rva).unwrap();
    let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
    // a cache file that holds a wrong value with a current fingerprint; grade must not read it
    let build = Fixtures::probe_build();
    Fixtures::write_cache(
        &probe.cache,
        build,
        build,
        &set,
        vec![("Probe.run", Fixtures::resolved(EntryKind::Function, 1))],
    );
    let cache_file = CacheFile::path(&probe.cache, build);
    let before = std::fs::read(&cache_file).unwrap();
    let (derive, runs) = Fixtures::probe_derive(&probe);
    let grade = Grade::new(derive, LeviLaminaSource::new(lookup, set.clone(), BASE, Fixtures::layout()), set);

    let (first, _) = grade.run();
    let (second, _) = grade.run();
    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(first, second);
    assert_eq!(std::fs::read(&cache_file).unwrap(), before);
    let verdicts: Vec<(&str, &str)> = first.rows.iter().map(|row| (row.id.0.as_str(), row.verdict().label())).collect();
    assert_eq!(verdicts, vec![("Probe.absent", "neither"), ("Probe.global", "mismatch"), ("Probe.run", "match")]);

    let logs = root.path().join("logs");
    let path = Grade::write(&first, &logs, "stem").unwrap();
    assert_eq!(path, logs.join(GradeReport::file_name("stem")));
    assert_eq!(std::fs::read_to_string(path).unwrap(), first.markdown("stem"));
}
