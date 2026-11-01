use std::path::PathBuf;
use std::time::Duration;

use amml_data::{BuildData, BuildId, EntryId, RecipeSet};
use amml_derive::cli::{DeriveArgs, DeriveCommand};
use amml_derive::{DeriveOutcome, Failure, FailureReason, StepError};
use tempfile::TempDir;

struct Run {
    dir: TempDir,
}

impl Run {
    fn new() -> Self {
        match TempDir::new() {
            Ok(dir) => Self { dir },
            Err(error) => panic!("{error}"),
        }
    }

    fn args(&self) -> DeriveArgs {
        DeriveArgs::new(
            PathBuf::from("unused.exe"),
            PathBuf::from("unused"),
            self.dir.path().join("data"),
            self.dir.path().join("reports"),
            None,
            None,
        )
    }

    fn outcome(failed: bool) -> DeriveOutcome {
        let failures = if failed {
            vec![Failure::new(EntryId("A".into()), FailureReason::AllRecipesFailed(vec![StepError::NoMatch]))]
        } else {
            Vec::new()
        };
        DeriveOutcome::new(BuildData::new(BuildId::new(1, 2)), failures, Vec::new())
    }

    fn finish(&self, first: &DeriveOutcome, second: &DeriveOutcome) -> amml_derive::Report {
        let Ok(set) = RecipeSet::new(Vec::new()) else { panic!("an empty set is valid") };
        match DeriveCommand::finish(&self.args(), &set, None, first, second, Duration::from_secs(1)) {
            Ok(report) => report,
            Err(error) => panic!("{error}"),
        }
    }
}

#[test]
fn equal_runs_write_the_data_and_the_report() {
    let run = Run::new();
    let report = run.finish(&Run::outcome(false), &Run::outcome(false));
    assert!(report.deterministic);
    assert_eq!(report.exit_code(), 0);
    assert!(run.dir.path().join("data/00000001-0000000000000002.hcl").is_file());
    assert!(run.dir.path().join("reports/00000001-0000000000000002.md").is_file());
}

#[test]
fn runs_that_differ_only_in_their_failures_write_the_report_but_not_the_data() {
    let run = Run::new();
    let report = run.finish(&Run::outcome(true), &Run::outcome(false));
    assert!(!report.deterministic);
    assert_eq!(report.exit_code(), 3);
    assert!(!run.dir.path().join("data").exists());
    let text = std::fs::read_to_string(run.dir.path().join("reports/00000001-0000000000000002.md")).unwrap();
    assert!(text.contains("deterministic: no"));
}

fn lines_without_elapsed(text: &str) -> Vec<&str> {
    text.lines().filter(|line| !line.starts_with("- elapsed:")).collect()
}

#[test]
fn jobs_do_not_change_data_or_report() {
    let small = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe/small.dll");
    let work = TempDir::new().unwrap();
    let recipes = work.path().join("recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    let entry = |name: &str, bytes: &str| {
        format!(
            "entry \"{name}\" {{\n  kind = \"function\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"{bytes}\"\n    }}\n  }}\n}}\n"
        )
    };
    std::fs::write(recipes.join("a.hcl"), entry("Lost", "DE AD BE EF 01 02 03 04 05 06 07 08")).unwrap();
    let derive = |name: &str, jobs: bool| {
        let root = work.path().join(name);
        let jobs_dir = root.join("jobs");
        let args = DeriveArgs::new(
            small.clone(),
            recipes.clone(),
            root.join("data"),
            root.join("reports"),
            None,
            jobs.then(|| jobs_dir.clone()),
        );
        DeriveCommand::run(&args).unwrap();
        root
    };
    let without = derive("without", false);
    let with = derive("with", true);
    let read = |root: &PathBuf, sub: &str| {
        let dir = root.join(sub);
        let file = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        std::fs::read_to_string(file).unwrap()
    };
    assert_eq!(read(&without, "data"), read(&with, "data"));
    let report_without = read(&without, "reports");
    let report_with = read(&with, "reports");
    assert_eq!(lines_without_elapsed(&report_without), lines_without_elapsed(&report_with));
    assert!(!without.join("jobs").exists());
    let jobs: Vec<_> = std::fs::read_dir(with.join("jobs")).unwrap().collect();
    assert_eq!(jobs.len(), 1);
    assert!(report_with.contains("Lost"));
}
