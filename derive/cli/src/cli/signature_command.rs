use std::path::Path;

use amml_data::{DataError, RecipeLoader, RecipeSet};

use super::SignatureArgs;
use crate::errors::CliError;
use crate::services::{Deriver, ImageLoader, SignatureCompare, SignatureProbe, SignatureStore};
use crate::structs::{EntrySignature, Image, Signature, SignatureFile, SignatureReport};

pub struct SignatureCommand;

impl SignatureCommand {
    /// Derives the build, probes every hooked entry, writes the signatures to `--out` (or standard output when
    /// neither `--out` nor `--compare` is given) and, with `--compare`, compares them with that file.
    pub fn run(args: &SignatureArgs) -> Result<(SignatureFile, Option<SignatureReport>), CliError> {
        let set = RecipeLoader::load_dir(&args.recipes)?;
        let image = ImageLoader::from_path(&args.exe)?;
        let file = Self::probe(&image, &set)?;
        match &args.out {
            Some(path) => {
                SignatureStore::write(path, &file).map_err(|source| CliError::Write { path: path.clone(), source })?
            }
            None if args.compare.is_none() => print!("{}", SignatureStore::render(&file)),
            None => {}
        }
        let report = args.compare.as_deref().map(|other| Self::compare(&file, other)).transpose()?;
        Ok((file, report))
    }

    /// The signatures of the hooked entries in recipe order; an entry that did not derive is unknown.
    pub fn probe(image: &Image, set: &RecipeSet) -> Result<SignatureFile, DataError> {
        let outcome = Deriver::new(image, set).derive()?;
        let probe = SignatureProbe::new(image);
        let mut entries = Vec::new();
        for entry in set.entries().iter().filter(|entry| entry.hooked) {
            let rva = outcome.data.entries.get(&entry.id).and_then(|resolved| u32::try_from(resolved.value).ok());
            let signature = match rva {
                Some(rva) => probe.probe(rva),
                None => Signature::unknown("the entry did not derive".to_string()),
            };
            entries.push(EntrySignature::new(entry.id.clone(), rva, signature));
        }
        Ok(SignatureFile::new(image.build(), entries))
    }

    /// Compares `current` against the signature file at `other`.
    pub fn compare(current: &SignatureFile, other: &Path) -> Result<SignatureReport, CliError> {
        let old = SignatureStore::read(other)?;
        Ok(SignatureCompare::compare(&old, current))
    }
}
