use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use amml_data::{BuildData, BuildDataStore, RecipeLoader, RecipeSet};

use super::DeriveArgs;
use crate::errors::CliError;
use crate::services::{Deriver, ImageLoader, JobWriter, ReportWriter};
use crate::structs::{DeriveOutcome, Image, Report};

pub struct DeriveCommand;

impl DeriveCommand {
    /// Loads the image once, reuses the entries of an existing data file in `--out` unless `--full` is set, derives twice with fresh state to prove the result is stable, and writes the report
    /// and, when both runs agree, the data. With `--jobs` it then writes the repair jobs, which never change the data
    /// or the report.
    pub fn run(args: &DeriveArgs) -> Result<Report, CliError> {
        let set = RecipeLoader::load_dir(&args.recipes)?;
        let previous = args.compare.as_deref().map(BuildDataStore::read).transpose()?;
        let started = Instant::now();
        let image = ImageLoader::from_path(&args.exe)?;
        let known = Self::known(args, &image);
        let outcome = Deriver::new(&image, &set).derive_with(&known, None)?;
        let again = Deriver::new(&image, &set).derive_with(&known, None)?;
        let report = Self::finish(args, &set, previous.as_ref(), &outcome, &again, started.elapsed())?;
        if let Some(dir) = &args.jobs {
            JobWriter::new(&image, &set, &outcome, previous.as_ref())
                .write(dir)
                .map_err(|source| CliError::Write { path: dir.clone(), source })?;
        }
        Ok(report)
    }

    // The data file an earlier run wrote, or an empty set when `--full` is given or the file is missing or unreadable.
    // With `--jobs` the stored failures are dropped, so they are derived again and carry their recipe trace.
    fn known(args: &DeriveArgs, image: &Image) -> BuildData {
        let empty = BuildData::new(image.build());
        if args.full {
            return empty;
        }
        let path = args.out.join(format!("{}.hcl", image.build().file_stem()));
        match BuildDataStore::read(&path) {
            Ok(mut data) => {
                if args.jobs.is_some() {
                    data.failed.clear();
                }
                data
            }
            Err(_) => empty,
        }
    }

    /// Compares the two outcomes (data, failures and degraded entries), always writes the report, and writes the
    /// data file only when they are equal.
    pub fn finish(
        args: &DeriveArgs,
        set: &RecipeSet,
        previous: Option<&BuildData>,
        outcome: &DeriveOutcome,
        again: &DeriveOutcome,
        elapsed: Duration,
    ) -> Result<Report, CliError> {
        let deterministic = outcome == again;
        let report = ReportWriter::new(outcome, set, elapsed, deterministic, previous);
        let stem = outcome.data.build.file_stem();
        Self::create_dir(&args.report)?;
        let report_path = args.report.join(format!("{stem}.md"));
        fs::write(&report_path, ReportWriter::render(&report))
            .map_err(|source| CliError::Write { path: report_path, source })?;
        if deterministic {
            Self::create_dir(&args.out)?;
            BuildDataStore::write(&args.out.join(format!("{stem}.hcl")), &outcome.data).map_err(CliError::Store)?;
        }
        Ok(report)
    }

    fn create_dir(path: &Path) -> Result<(), CliError> {
        fs::create_dir_all(path).map_err(|source| CliError::Write { path: path.to_path_buf(), source })
    }
}
